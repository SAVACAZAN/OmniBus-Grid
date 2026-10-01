use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use omibus_core::{GridConfig, GridPreview, PaperBot};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use crate::module_config::ModuleConfig;

#[derive(Clone, Serialize, Deserialize)]
struct Store {
    schema_version: u32,
    next_id: u64,
    bots: Vec<PaperBot>,
}

impl Default for Store {
    fn default() -> Self { Self { schema_version: 1, next_id: 1, bots: Vec::new() } }
}

pub(crate) struct AppState { file: PathBuf, store: Mutex<Store> }

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

fn load_store(file: &PathBuf) -> Result<Store, Box<dyn std::error::Error>> {
    if !file.exists() { return Ok(Store::default()); }
    let store: Store = serde_json::from_slice(&fs::read(file)?)?;
    if store.schema_version != 1 { return Err("Unsupported paper data version".into()); }
    Ok(store)
}

fn save_store(file: &PathBuf, store: &Store) -> Result<(), String> {
    use std::io::Write;
    let parent = file.parent().ok_or("Invalid data path")?;
    fs::create_dir_all(parent).map_err(|e| format!("Cannot create data directory: {e}"))?;
    let temp = file.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(store).map_err(|e| e.to_string())?;
    let mut handle = fs::File::create(&temp).map_err(|e| e.to_string())?;
    handle.write_all(&bytes).and_then(|_| handle.sync_all()).map_err(|e| e.to_string())?;
    fs::rename(&temp, file).map_err(|e| format!("Cannot save paper bots: {e}"))
}

// Persist a complete copy before replacing in-memory state, so a failed write
// cannot show a successful command that disappears after the app restarts.
fn transact<T>(state: State<'_, AppState>, change: impl FnOnce(&mut Store) -> Result<T, String>) -> Result<T, String> {
    let mut guard = state.store.lock().map_err(|_| "Paper store lock failed".to_string())?;
    let mut next = guard.clone();
    let result = change(&mut next)?;
    save_store(&state.file, &next)?;
    *guard = next;
    Ok(result)
}

#[tauri::command]
pub(crate) fn preview_grid(modules: State<'_, ModuleConfig>, config: GridConfig, current_price: f64) -> Result<GridPreview, String> {
    modules.require_grid()?;
    config.preview(current_price)
}

#[tauri::command]
pub(crate) fn list_bots(modules: State<'_, ModuleConfig>, state: State<'_, AppState>) -> Result<Vec<PaperBot>, String> {
    modules.require_grid()?;
    let guard = state.store.lock().map_err(|_| "Paper store lock failed".to_string())?;
    Ok(guard.bots.clone())
}

#[tauri::command]
pub(crate) fn create_paper_bot(modules: State<'_, ModuleConfig>, state: State<'_, AppState>, name: String, config: GridConfig,
                    current_price: f64, initial_base: f64, initial_quote: f64) -> Result<PaperBot, String> {
    modules.require_grid()?;
    transact(state, |store| {
        let bot = PaperBot::new(store.next_id, name, config, current_price,
                                initial_base, initial_quote, now_ms())?;
        store.next_id = store.next_id.checked_add(1).ok_or("Bot ID limit reached")?;
        store.bots.push(bot.clone());
        Ok(bot)
    })
}

#[tauri::command]
pub(crate) fn tick_bot(modules: State<'_, ModuleConfig>, state: State<'_, AppState>, id: u64, price: f64) -> Result<PaperBot, String> {
    modules.require_grid()?;
    transact(state, |store| {
        let bot = store.bots.iter_mut().find(|b| b.id == id).ok_or("Bot not found")?;
        bot.tick(price, now_ms())?;
        Ok(bot.clone())
    })
}

#[tauri::command]
pub(crate) fn bot_action(modules: State<'_, ModuleConfig>, state: State<'_, AppState>, id: u64, action: String) -> Result<PaperBot, String> {
    modules.require_grid()?;
    transact(state, |store| {
        let bot = store.bots.iter_mut().find(|b| b.id == id).ok_or("Bot not found")?;
        match action.as_str() {
            "pause" => bot.pause()?,
            "resume" => bot.resume()?,
            "stop" => bot.stop()?,
            _ => return Err("Unknown bot action".into()),
        }
        Ok(bot.clone())
    })
}

#[tauri::command]
pub(crate) fn delete_bot(modules: State<'_, ModuleConfig>, state: State<'_, AppState>, id: u64) -> Result<(), String> {
    modules.require_grid()?;
    transact(state, |store| {
        let index = store.bots.iter().position(|b| b.id == id).ok_or("Bot not found")?;
        if store.bots[index].status != omibus_core::BotStatus::Stopped {
            return Err("Stop the bot before deleting it".into());
        }
        store.bots.remove(index);
        Ok(())
    })
}

pub(crate) fn initialize(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let file = app.path().app_data_dir()?.join("paper-bots.json");
    let store = load_store(&file)?;
    app.manage(AppState { file, store: Mutex::new(store) });
    Ok(())
}
