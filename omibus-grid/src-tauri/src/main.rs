
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod module_config;
mod modules;
mod db;

use module_config::ModuleConfig;
use db::Database;
use parking_lot::Mutex;
use tauri::{Manager, State};
use std::sync::Arc;

use omnibus_exchange::ExchangeRegistry;
use omnibus_core::{ExchangeConfig, EndpointConfig, Side, OrderType};

struct SessionData {
    user_id:      i64,
    display_name: String,
    key:          [u8; 32],
}

struct AppState {
    db:       Database,
    session:  Mutex<Option<SessionData>>,
    exchange: Arc<ExchangeRegistry>,
}

// ── Auth commands ─────────────────────────────────────────────────────────────

#[tauri::command]
fn get_user_count(state: State<'_, AppState>) -> Result<i64, String> {
    state.db.auth.get_user_count()
}

#[tauri::command]
fn auth_register(
    state: State<'_, AppState>,
    username: String,
    password: String,
    display_name: String,
) -> Result<serde_json::Value, String> {
    let (user_id, key) = state.db.auth.register(&username, &password, &display_name)?;
    *state.session.lock() = Some(SessionData { user_id, display_name: display_name.clone(), key });
    Ok(serde_json::json!({ "user_id": user_id, "display_name": display_name }))
}

#[tauri::command]
fn auth_login(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<serde_json::Value, String> {
    let (user_id, display_name, key) = state.db.auth.login(&username, &password)?;
    *state.session.lock() = Some(SessionData { user_id, display_name: display_name.clone(), key });
    Ok(serde_json::json!({ "user_id": user_id, "display_name": display_name }))
}

#[tauri::command]
fn auth_logout(state: State<'_, AppState>) {
    *state.session.lock() = None;
}

#[tauri::command]
fn auth_whoami(state: State<'_, AppState>) -> Option<serde_json::Value> {
    let s = state.session.lock();
    s.as_ref().map(|sd| serde_json::json!({ "user_id": sd.user_id, "display_name": sd.display_name }))
}

// ── API-key commands ──────────────────────────────────────────────────────────

#[tauri::command]
fn save_api_key(
    state: State<'_, AppState>,
    exchange: String,
    label: String,
    api_key: String,
    api_secret: String,
) -> Result<i64, String> {
    let sess = state.session.lock();
    let sd = sess.as_ref().ok_or("Not logged in")?;
    state.db.api_keys.save(sd.user_id, &exchange, &label, &api_key, &api_secret, &sd.key)
}

#[tauri::command]
fn get_api_keys(
    state: State<'_, AppState>,
    exchange: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let sess = state.session.lock();
    let sd = sess.as_ref().ok_or("Not logged in")?;
    state.db.api_keys.get_all(sd.user_id, exchange.as_deref(), &sd.key)
}

#[tauri::command]
fn delete_api_key(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let sess = state.session.lock();
    let sd = sess.as_ref().ok_or("Not logged in")?;
    state.db.api_keys.delete(id, sd.user_id)
}

#[tauri::command]
fn rename_api_key(state: State<'_, AppState>, id: i64, label: String) -> Result<(), String> {
    let sess = state.session.lock();
    let sd = sess.as_ref().ok_or("Not logged in")?;
    state.db.api_keys.rename(id, sd.user_id, &label)
}

// ── Saved credentials ────────────────────────────────────────────────────────

/// Save credentials for one-click login. Call after a successful login when user opts in.
#[tauri::command]
fn auth_save_credentials(state: State<'_, AppState>, username: String, password: String) -> Result<(), String> {
    state.db.auth.save_credentials(&state.db.path, &username, &password)
}

/// Try to auto-login with saved credentials. Returns session payload if credentials exist and are valid.
#[tauri::command]
fn auth_autologin(state: State<'_, AppState>) -> Result<Option<serde_json::Value>, String> {
    let Some((username, password)) = state.db.auth.load_credentials(&state.db.path) else {
        return Ok(None);
    };
    match state.db.auth.login(&username, &password) {
        Ok((user_id, display_name, key)) => {
            *state.session.lock() = Some(SessionData { user_id, display_name: display_name.clone(), key });
            Ok(Some(serde_json::json!({ "user_id": user_id, "display_name": display_name })))
        }
        Err(_) => {
            // Credentials stale (password changed) — clear them
            let _ = state.db.auth.forget_credentials();
            Ok(None)
        }
    }
}

/// Remove saved credentials (user unchecks Remember Me or logs out).
#[tauri::command]
fn auth_forget_credentials(state: State<'_, AppState>) -> Result<(), String> {
    state.db.auth.forget_credentials()
}

// ── Misc ──────────────────────────────────────────────────────────────────────

#[tauri::command]
fn get_modules(config: State<'_, ModuleConfig>) -> ModuleConfig { config.inner().clone() }

// ── Exchange — setup ──────────────────────────────────────────────────────────

fn make_exchange_config(name: &str) -> ExchangeConfig {
    let (base_url, ws_url, auth_method, symbol_format, endpoints) = match name {
        "kraken" => (
            "https://api.kraken.com",
            Some("wss://ws.kraken.com".to_string()),
            "kraken-hmac",
            "XBT/USD",
            EndpointConfig {
                ticker:       "/0/public/Ticker".into(),
                tickers:      None,
                orderbook:    "/0/public/Depth".into(),
                trades:       "/0/public/Trades".into(),
                kline:        Some("/0/public/OHLC".into()),
                pairs:        "/0/public/AssetPairs".into(),
                order_create: "/0/private/AddOrder".into(),
                order_modify: None,
                order_cancel: "/0/private/CancelOrder".into(),
                open_orders:  "/0/private/OpenOrders".into(),
                balances:     "/0/private/Balance".into(),
                balance:      None,
                order_history: Some("/0/private/ClosedOrders".into()),
                trade_history: Some("/0/private/TradesHistory".into()),
            },
        ),
        "coinbase" => (
            "https://api.coinbase.com",
            Some("wss://ws-feed.exchange.coinbase.com".to_string()),
            "coinbase-jwt",
            "BTC-USD",
            EndpointConfig {
                ticker:       "/api/v3/brokerage/best_bid_ask".into(),
                tickers:      None,
                orderbook:    "/products/{pair}/book".into(),
                trades:       "/products/{pair}/trades".into(),
                kline:        Some("/api/v3/brokerage/products/{pair}/candles".into()),
                pairs:        "/api/v3/brokerage/products".into(),
                order_create: "/api/v3/brokerage/orders".into(),
                order_modify: None,
                order_cancel: "/api/v3/brokerage/orders/batch_cancel".into(),
                open_orders:  "/api/v3/brokerage/orders/historical/batch".into(),
                balances:     "/api/v3/brokerage/accounts".into(),
                balance:      None,
                order_history: Some("/api/v3/brokerage/orders/historical/batch".into()),
                trade_history: Some("/api/v3/brokerage/orders/historical/fills".into()),
            },
        ),
        _ => (
            "https://api.unknown.com",
            None,
            "none",
            "BTC/USD",
            EndpointConfig {
                ticker: "/ticker".into(), tickers: None,
                orderbook: "/orderbook".into(), trades: "/trades".into(),
                kline: None, pairs: "/pairs".into(),
                order_create: "/order".into(), order_modify: None,
                order_cancel: "/cancel".into(), open_orders: "/orders".into(),
                balances: "/balances".into(), balance: None,
                order_history: None, trade_history: None,
            },
        ),
    };
    ExchangeConfig {
        enabled: true,
        name: name.into(),
        base_url: base_url.into(),
        ws_url,
        auth_method: auth_method.into(),
        symbol_format: symbol_format.into(),
        api_version: None,
        rate_limit_per_second: 3,
        trading_pairs: vec![],
        endpoints,
        orderbook_parse: None,
        websocket_subscribe: None,
        post_order_delay_ms: 1300,
        key_iteration_delay_ms: 1100,
        retry_429_delay_ms: 2000,
    }
}

/// Register API keys for an exchange (stores in ExchangeRegistry for this session).
#[tauri::command]
async fn exchange_register(
    state: State<'_, AppState>,
    exchange: String,
    api_key: String,
    api_secret: String,
) -> Result<(), String> {
    let cfg = make_exchange_config(&exchange);
    state.exchange.register(&cfg, Some(api_key), Some(api_secret)).await
        .map_err(|e| e.to_string())
}

/// Register public-only (no keys) access to an exchange.
#[tauri::command]
async fn exchange_register_public(
    state: State<'_, AppState>,
    exchange: String,
) -> Result<(), String> {
    let cfg = make_exchange_config(&exchange);
    state.exchange.register(&cfg, None, None).await
        .map_err(|e| e.to_string())
}

// ── Exchange — public API ─────────────────────────────────────────────────────

#[tauri::command]
async fn exchange_ticker(
    state: State<'_, AppState>,
    exchange: String,
    symbol: String,
) -> Result<serde_json::Value, String> {
    let api = state.exchange.get(&exchange).ok_or_else(|| format!("{exchange} not registered"))?;

    let t = api.ticker(&symbol).await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(t).unwrap())
}

#[tauri::command]
async fn exchange_orderbook(
    state: State<'_, AppState>,
    exchange: String,
    symbol: String,
    depth: usize,
) -> Result<serde_json::Value, String> {
    let api = state.exchange.get(&exchange).ok_or_else(|| format!("{exchange} not registered"))?;

    let ob = api.orderbook(&symbol, depth).await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(ob).unwrap())
}

#[tauri::command]
async fn exchange_trades(
    state: State<'_, AppState>,
    exchange: String,
    symbol: String,
    limit: usize,
) -> Result<serde_json::Value, String> {
    let api = state.exchange.get(&exchange).ok_or_else(|| format!("{exchange} not registered"))?;

    let t = api.trades(&symbol, limit).await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(t).unwrap())
}

#[tauri::command]
async fn exchange_candles(
    state: State<'_, AppState>,
    exchange: String,
    symbol: String,
    interval: String,
    limit: usize,
) -> Result<serde_json::Value, String> {
    let api = state.exchange.get(&exchange).ok_or_else(|| format!("{exchange} not registered"))?;

    let c = api.candles(&symbol, &interval, limit).await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(c).unwrap())
}

#[tauri::command]
async fn exchange_markets(
    state: State<'_, AppState>,
    exchange: String,
) -> Result<Vec<String>, String> {
    let api = state.exchange.get(&exchange).ok_or_else(|| format!("{exchange} not registered"))?;

    api.markets().await.map_err(|e| e.to_string())
}

// ── Exchange — private API ────────────────────────────────────────────────────

#[tauri::command]
async fn exchange_balances(
    state: State<'_, AppState>,
    exchange: String,
) -> Result<serde_json::Value, String> {
    let api = state.exchange.get_private(&exchange).ok_or_else(|| format!("{exchange} has no private API (add API keys first)"))?;

    let b = api.balances().await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(b).unwrap())
}

#[tauri::command]
async fn exchange_place_order(
    state: State<'_, AppState>,
    exchange: String,
    symbol: String,
    side: String,
    order_type: String,
    qty: f64,
    price: Option<f64>,
) -> Result<serde_json::Value, String> {
    let api = state.exchange.get_private(&exchange).ok_or_else(|| format!("{exchange} has no private API (add API keys first)"))?;
    let side = match side.to_lowercase().as_str() {
        "buy"  => Side::Buy,
        "sell" => Side::Sell,
        other  => return Err(format!("Unknown side: {other}")),
    };
    let order_type = match order_type.to_lowercase().as_str() {
        "limit"  => OrderType::Limit,
        "market" => OrderType::Market,
        other    => return Err(format!("Unknown order type: {other}")),
    };

    let order = api.place_order(&symbol, side, order_type, qty, price).await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(order).unwrap())
}

#[tauri::command]
async fn exchange_cancel_order(
    state: State<'_, AppState>,
    exchange: String,
    order_id: String,
) -> Result<bool, String> {
    let api = state.exchange.get_private(&exchange).ok_or_else(|| format!("{exchange} has no private API"))?;

    api.cancel_order(&order_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn exchange_open_orders(
    state: State<'_, AppState>,
    exchange: String,
) -> Result<serde_json::Value, String> {
    let api = state.exchange.get_private(&exchange).ok_or_else(|| format!("{exchange} has no private API"))?;

    let orders = api.open_orders().await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(orders).unwrap())
}

#[tauri::command]
async fn exchange_order_history(
    state: State<'_, AppState>,
    exchange: String,
    limit: usize,
) -> Result<serde_json::Value, String> {
    let api = state.exchange.get_private(&exchange).ok_or_else(|| format!("{exchange} has no private API"))?;

    let orders = api.order_history(limit).await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(orders).unwrap())
}

#[tauri::command]
async fn exchange_trade_history(
    state: State<'_, AppState>,
    exchange: String,
    limit: usize,
) -> Result<serde_json::Value, String> {
    let api = state.exchange.get_private(&exchange).ok_or_else(|| format!("{exchange} has no private API"))?;

    let trades = api.trade_history(limit).await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(trades).unwrap())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // resolve DB path in app data dir
            let data_dir = app.path().app_data_dir()
                .map_err(|e| format!("No app data dir: {e}"))?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("omibus.db");
            let db = Database::open(db_path.to_str().unwrap())?;

            let exchange = Arc::new(ExchangeRegistry::new());
            // Pre-register public access so the UI can query orderbook/ticker without login
            let reg_clone = exchange.clone();
            tauri::async_runtime::block_on(async move {
                for ex in &["kraken", "coinbase"] {
                    let cfg = make_exchange_config(ex);
                    let _ = reg_clone.register(&cfg, None, None).await;
                }
            });
            app.manage(AppState { db, session: Mutex::new(None), exchange });

            // module setup
            let config = ModuleConfig::load()?;
            if config.grid { modules::grid::initialize(app)?; }
            if config.ai   { modules::ai::initialize(app)?; }
            app.manage(config);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // modules
            get_modules,
            modules::grid::preview_grid, modules::grid::list_bots,
            modules::grid::create_paper_bot, modules::grid::tick_bot,
            modules::grid::bot_action, modules::grid::delete_bot,
            modules::ai::ai_start, modules::ai::ai_stop,
            modules::ai::ai_status, modules::ai::ai_chart_snapshot,
            // auth
            get_user_count, auth_register, auth_login, auth_logout, auth_whoami,
            auth_save_credentials, auth_autologin, auth_forget_credentials,
            // api keys
            save_api_key, get_api_keys, delete_api_key, rename_api_key,
            // exchange — setup
            exchange_register, exchange_register_public,
            // exchange — public
            exchange_ticker, exchange_orderbook, exchange_trades, exchange_candles, exchange_markets,
            // exchange — private
            exchange_balances, exchange_place_order, exchange_cancel_order,
            exchange_open_orders, exchange_order_history, exchange_trade_history,
        ])
        .run(tauri::generate_context!())
        .expect("failed to launch Omibus Grid");
}
