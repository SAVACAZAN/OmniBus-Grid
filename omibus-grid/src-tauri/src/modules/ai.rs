//! Process supervisor only. Model training never runs on the UI/backend thread.
use std::{fs, io::Write, path::PathBuf, process::{Child, Command, Stdio}, sync::{Mutex, Arc, atomic::{AtomicBool, Ordering}}};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use crate::module_config::ModuleConfig;

fn input_catalog() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../ai-worker/catalog.json")).expect("Bundled AI input catalog is valid")
}
fn default_inputs() -> Vec<String> {
    input_catalog()["default_inputs"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect()
}
fn default_take_profit() -> f64 { 1.0 }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TriggerRule { left: String, op: String, right: String, value: f64 }
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Triggers { buy: Vec<TriggerRule>, sell: Vec<TriggerRule> }
impl Triggers {
    fn validate(&self, inputs: &[String]) -> Result<(), String> {
        let catalog = input_catalog();
        let series: Vec<&str> = catalog["inputs"].as_array().unwrap().iter()
            .filter(|item| inputs.iter().any(|id| item["id"].as_str() == Some(id.as_str())))
            .flat_map(|item| item["series"].as_array().unwrap().iter().filter_map(|v| v.as_str())).collect();
        for rules in [&self.buy, &self.sell] {
            if rules.len()>4 { return Err("Use at most four entry conditions per side".into()); }
            for rule in rules {
                if !series.contains(&rule.left.as_str()) || !(rule.right == "number" || series.contains(&rule.right.as_str()))
                    || !["above","below","cross_above","cross_below"].contains(&rule.op.as_str())
                    || !rule.value.is_finite() || rule.value.abs()>1e12 {
                    return Err("Invalid trigger: use selected indicators, a supported comparison and a finite level".into());
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResearchConfig {
    symbol: String, interval: String, horizon: u32, history_bars: u32,
    fee_bps: f64, slippage_bps: f64, threshold: f64,
    #[serde(default = "default_inputs")]
    inputs: Vec<String>,
    #[serde(default = "default_take_profit")]
    take_profit_pct: f64,
    #[serde(default)]
    triggers: Triggers,
}
impl Default for ResearchConfig {
    fn default() -> Self {
        Self { symbol: "BTCUSDT".into(), interval: "1h".into(), horizon: 4,
            history_bars: 3000, fee_bps: 10.0, slippage_bps: 5.0, threshold: 0.6,
            inputs: default_inputs(), take_profit_pct: default_take_profit(), triggers: Triggers::default() }
    }
}
impl ResearchConfig {
    fn validate(&self) -> Result<(), String> {
        if !(5..=20).contains(&self.symbol.len()) || !self.symbol.bytes().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
            return Err("Use an uppercase Binance symbol, such as BTCUSDT".into());
        }
        if !input_catalog()["intervals"].as_array().unwrap().iter().any(|i| i["id"].as_str()==Some(self.interval.as_str())) || !(1..=24).contains(&self.horizon) || !(1500..=10000).contains(&self.history_bars) {
            return Err("Invalid interval, horizon or history length".into());
        }
        if ![self.fee_bps, self.slippage_bps].iter().all(|v| v.is_finite() && (0.0..=100.0).contains(v)) || !self.threshold.is_finite() || !(0.5..=0.95).contains(&self.threshold) {
            return Err("Fees/slippage must be 0–100 basis points; threshold must be 0.50–0.95".into());
        }
        let catalog = input_catalog();
        let known = catalog["inputs"].as_array().unwrap();
        let unique: std::collections::HashSet<_> = self.inputs.iter().collect();
        if self.inputs.is_empty() || self.inputs.len() > known.len() || unique.len() != self.inputs.len()
            || self.inputs.iter().any(|id| !known.iter().any(|item| item["id"].as_str() == Some(id.as_str()))) {
            return Err("Select at least one valid learning input; duplicates are not allowed".into());
        }
        if !self.take_profit_pct.is_finite() || !(0.1..=50.0).contains(&self.take_profit_pct) {
            return Err("Take profit must be 0.1–50% after configured costs".into());
        }
        self.triggers.validate(&self.inputs)
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct Preferences { resume: bool, config: ResearchConfig }
struct Supervisor { child: Option<Child>, preferences: Preferences, error: Option<String> }
pub(crate) struct AiService { directory: PathBuf, worker: PathBuf, inner: Mutex<Supervisor>, chart_busy: Arc<AtomicBool> }

fn write_json(path: &PathBuf, value: &impl Serialize) -> Result<(), String> {
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(temporary, path).map_err(|e| e.to_string())
}

impl AiService {
    fn launch(&self, config: &ResearchConfig) -> Result<Child, String> {
        config.validate()?;
        let runtime = self.worker.join("runtime").join("python.exe");
        let script = self.worker.join("worker.py");
        if !runtime.is_file() || !script.is_file() {
            return Err("AI runtime is missing. Keep the ai-worker folder next to the executable.".into());
        }
        fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        let config_path = self.directory.join("config.json");
        write_json(&config_path, config)?;
        let log = fs::File::create(self.directory.join("worker.log")).map_err(|e| e.to_string())?;
        let mut command = Command::new(runtime);
        command.arg("-I").arg(script).arg("--data-dir").arg(&self.directory)
            .arg("--config").arg(config_path).arg("--parent-pid").arg(std::process::id().to_string())
            .current_dir(&self.worker).stdin(Stdio::null()).stdout(Stdio::null()).stderr(log);
        #[cfg(windows)] {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        command.spawn().map_err(|e| format!("Cannot start AI worker: {e}"))
    }
    fn save(&self, preferences: &Preferences) -> Result<(), String> {
        fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        write_json(&self.directory.join("preferences.json"), preferences)
    }
}

impl Drop for AiService {
    fn drop(&mut self) {
        if let Ok(inner) = self.inner.get_mut() {
            if let Some(mut child) = inner.child.take() { let _ = child.kill(); let _ = child.wait(); }
        }
    }
}

pub(crate) fn initialize(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let directory = app.path().app_data_dir()?.join("ai-research");
    let adjacent = std::env::current_exe()?.with_file_name("ai-worker");
    let bundled = app.path().resource_dir()?.join("ai-worker");
    let worker = if adjacent.join("worker.py").is_file() { adjacent }
        else if bundled.join("worker.py").is_file() { bundled }
        else { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ai-worker") };
    let mut error = None;
    let preferences = match fs::read(directory.join("preferences.json")) {
        Ok(bytes) => match serde_json::from_slice::<Preferences>(&bytes) {
            Ok(mut p) if p.config.validate().is_ok() => {
                let raw: serde_json::Value = serde_json::from_slice(&bytes)?;
                if raw["config"].get("inputs").is_none() || raw["config"].get("take_profit_pct").is_none() || raw["config"].get("triggers").is_none() {
                    p.resume = false;
                    error = Some("AI Research upgraded: select learning inputs and press Start. Previous research is preserved in its original profile.".into());
                }
                p.config.inputs.sort();
                p
            },
            _ => { error = Some("Invalid AI preferences; review settings before starting".into()); Preferences::default() }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Preferences::default(),
        Err(e) => { error = Some(format!("Cannot read AI preferences: {e}")); Preferences::default() }
    };
    let service = AiService { directory, worker, inner: Mutex::new(Supervisor { child: None, preferences: preferences.clone(), error }), chart_busy: Arc::new(AtomicBool::new(false)) };
    if preferences.resume {
        let mut inner = service.inner.lock().map_err(|_| "AI supervisor lock failed")?;
        match service.launch(&preferences.config) { Ok(child) => inner.child = Some(child), Err(e) => inner.error = Some(e) }
    }
    app.manage(service);
    Ok(())
}

fn enabled(modules: &ModuleConfig) -> Result<(), String> {
    if modules.ai { Ok(()) } else { Err("AI Research is disabled in modules.json".into()) }
}

#[tauri::command]
pub(crate) fn ai_start(modules: State<'_, ModuleConfig>, service: State<'_, AiService>, mut config: ResearchConfig) -> Result<(), String> {
    enabled(&modules)?;
    config.validate()?;
    config.inputs.sort();
    let mut inner = service.inner.lock().map_err(|_| "AI supervisor lock failed")?;
    if let Some(child) = inner.child.as_mut() {
        if child.try_wait().map_err(|e| e.to_string())?.is_none() { return Err("Stop the current AI worker before changing settings".into()); }
    }
    let preferences = Preferences { resume: true, config };
    let mut child = service.launch(&preferences.config)?;
    if let Err(error) = service.save(&preferences) { let _ = child.kill(); let _ = child.wait(); return Err(error); }
    inner.child = Some(child); inner.preferences = preferences; inner.error = None;
    Ok(())
}

#[tauri::command]
pub(crate) fn ai_stop(modules: State<'_, ModuleConfig>, service: State<'_, AiService>) -> Result<(), String> {
    enabled(&modules)?;
    let mut inner = service.inner.lock().map_err(|_| "AI supervisor lock failed")?;
    inner.preferences.resume = false;
    let saved = service.save(&inner.preferences);
    if let Some(mut child) = inner.child.take() {
        if child.try_wait().map_err(|e| e.to_string())?.is_none() { child.kill().map_err(|e| e.to_string())?; }
        child.wait().map_err(|e| e.to_string())?;
    }
    saved
}

#[tauri::command]
pub(crate) fn ai_status(modules: State<'_, ModuleConfig>, service: State<'_, AiService>) -> Result<serde_json::Value, String> {
    enabled(&modules)?;
    let mut inner = service.inner.lock().map_err(|_| "AI supervisor lock failed")?;
    let mut running = false;
    if let Some(child) = inner.child.as_mut() {
        match child.try_wait().map_err(|e| e.to_string())? {
            None => running = true,
            Some(code) => { inner.child = None; inner.error = Some(format!("AI worker exited ({code}). See worker.log, then press Start to retry.")); }
        }
    }
    let status: serde_json::Value = fs::read(service.directory.join("status.json")).ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or(serde_json::Value::Null);
    Ok(serde_json::json!({ "running": running, "resume": inner.preferences.resume, "config": inner.preferences.config,
        "error": inner.error, "worker": status, "data_directory": service.directory, "catalog": input_catalog(), "build": "AI Research 3 · live chart" }))
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChartRequest { symbol: String, interval: String, inputs: Vec<String>, triggers: Triggers }

#[tauri::command]
pub(crate) async fn ai_chart_snapshot(modules: State<'_, ModuleConfig>, service: State<'_, AiService>, config: ChartRequest) -> Result<serde_json::Value, String> {
    enabled(&modules)?;
    let check = ResearchConfig { symbol: config.symbol.clone(), interval: config.interval.clone(), inputs: config.inputs.clone(), triggers: config.triggers.clone(), ..ResearchConfig::default() };
    check.validate()?;
    let flag = service.chart_busy.clone();
    if flag.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst).is_err() { return Err("A chart update is already in progress".into()); }
    struct Guard(Arc<AtomicBool>);
    impl Drop for Guard { fn drop(&mut self) { self.0.store(false,Ordering::SeqCst); } }
    let guard=Guard(flag);
    let worker=service.worker.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard=guard;
        let mut command=Command::new(worker.join("runtime/python.exe"));
        command.arg("-I").arg(worker.join("chart_worker.py")).current_dir(&worker)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        #[cfg(windows)] {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child=command.spawn().map_err(|e| format!("Cannot start chart reader: {e}"))?;
        let bytes=serde_json::to_vec(&config).map_err(|e| e.to_string())?;
        let sent=child.stdin.take().ok_or("Chart input unavailable".to_string())?.write_all(&bytes);
        if let Err(error)=sent { let _=child.kill(); let _=child.wait(); return Err(error.to_string()); }
        let output=child.wait_with_output().map_err(|e| e.to_string())?;
        if !output.status.success() { return Err(format!("Chart reader failed: {}",String::from_utf8_lossy(&output.stderr).chars().take(1500).collect::<String>())); }
        serde_json::from_slice(&output.stdout).map_err(|e| format!("Invalid chart snapshot: {e}"))
    }).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_reject_injection_and_unbounded_work() {
        let mut config = ResearchConfig::default();
        assert!(config.validate().is_ok());
        config.symbol = "BTCUSDT;cmd".into(); assert!(config.validate().is_err());
        config = ResearchConfig::default(); config.horizon = 0; assert!(config.validate().is_err());
        config = ResearchConfig::default(); config.history_bars = 1000000; assert!(config.validate().is_err());
        config = ResearchConfig::default(); config.fee_bps = f64::NAN; assert!(config.validate().is_err());
    }
    #[test]
    fn selected_inputs_and_take_profit_are_validated() {
        let mut config = ResearchConfig::default();
        for item in input_catalog()["inputs"].as_array().unwrap() {
            config.inputs = vec![item["id"].as_str().unwrap().to_string()];
            assert!(config.validate().is_ok());
        }
        config.inputs = vec![]; assert!(config.validate().is_err());
        config.inputs = vec!["rsi".into(), "rsi".into()]; assert!(config.validate().is_err());
        config.inputs = vec!["unknown".into()]; assert!(config.validate().is_err());
        config.inputs = vec!["rsi".into()]; config.take_profit_pct = f64::NAN; assert!(config.validate().is_err());
        config.take_profit_pct = 0.0; assert!(config.validate().is_err());
        let legacy = r#"{"symbol":"BTCUSDT","interval":"1h","horizon":4,"history_bars":3000,"fee_bps":10,"slippage_bps":5,"threshold":0.6}"#;
        assert!(serde_json::from_str::<ResearchConfig>(legacy).unwrap().validate().is_ok());
    }
    #[test]
    fn catalog_and_custom_entry_filters_are_validated() {
        let catalog=input_catalog();
        assert_eq!(catalog["inputs"].as_array().unwrap().iter().filter(|i|i["kind"]=="pattern").count(),20);
        let mut config=ResearchConfig::default();
        config.inputs=vec!["rsi".into()];
        config.triggers.buy.push(TriggerRule{left:"rsi".into(),op:"cross_above".into(),right:"number".into(),value:30.0});
        for interval in catalog["intervals"].as_array().unwrap() { config.interval=interval["id"].as_str().unwrap().into(); assert!(config.validate().is_ok()); }
        config.inputs=vec!["ema".into()]; assert!(config.validate().is_err());
        config.inputs=vec!["rsi".into()];config.triggers.buy[0].value=f64::NAN;assert!(config.validate().is_err());
        config.triggers.buy[0].value=30.0;config.triggers.buy[0].op="unknown".into();assert!(config.validate().is_err());
    }
}
