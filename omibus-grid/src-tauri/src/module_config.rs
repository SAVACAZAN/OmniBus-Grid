use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ModuleConfig {
    pub grid:             bool,
    pub charts:           bool,
    #[serde(default)]                  pub orderbook:       bool,
    #[serde(default)]                  pub trade:           bool,
    #[serde(default)]                  pub gridbotplus:     bool,
    #[serde(default)]                  pub ai:              bool,
    #[serde(default = "default_true")] pub auth:            bool,
    #[serde(default = "default_true")] pub profile:         bool,
    #[serde(default = "default_true")] pub market_selector: bool,
}

fn default_true() -> bool { true }

impl ModuleConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_from(&std::env::current_exe()?.with_file_name("modules.json"))
    }
    fn load_from(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => include_str!("../../modules.json").to_string(),
            Err(error) => return Err(error.into()),
        };
        serde_json::from_str(&text).map_err(|error| format!("Invalid module configuration at {}: {error}", path.display()).into())
    }
    pub fn require_grid(&self) -> Result<(), String> {
        if self.grid { Ok(()) } else { Err("The Grid module is disabled in modules.json".into()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configuration_combinations_gate_grid_commands() {
        for grid in [false, true] { for charts in [false, true] {
            let config: ModuleConfig = serde_json::from_value(serde_json::json!({"grid": grid, "charts": charts})).unwrap();
            assert_eq!(config.require_grid().is_ok(), grid);
            assert_eq!(config.charts, charts);
        } }
    }
    #[test]
    fn invalid_config_never_silently_enables_modules() {
        for json in [r#"{"grid":"false","charts":true}"#, r#"{"grid":true}"#,
            r#"{"grid":true,"charts":true,"unknown":true}"#, "bad json"] {
            assert!(serde_json::from_str::<ModuleConfig>(json).is_err());
        }
    }
    #[test]
    fn shipped_defaults_are_valid() {
        let _: ModuleConfig = serde_json::from_str(include_str!("../../modules.json")).unwrap();
    }
    #[test]
    fn disk_override_and_missing_file_fallback() {
        let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("omibus-module-test-{}-{unique}.json", std::process::id()));
        let defaults: ModuleConfig = serde_json::from_str(include_str!("../../modules.json")).unwrap();
        assert_eq!(ModuleConfig::load_from(&path).unwrap(), defaults);
        fs::write(&path, r#"{"grid":false,"charts":true}"#).unwrap();
        assert_eq!(ModuleConfig::load_from(&path).unwrap(), ModuleConfig {
            grid: false, charts: true, orderbook: false, trade: false, gridbotplus: false, ai: false, auth: true, profile: true, market_selector: true
        });
        fs::write(&path, "invalid").unwrap();
        assert!(ModuleConfig::load_from(&path).is_err());
        fs::remove_file(path).unwrap();
    }
}
