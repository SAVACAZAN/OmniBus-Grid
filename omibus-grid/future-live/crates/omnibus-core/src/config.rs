use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use crate::OmniBusError;
type Result<T> = std::result::Result<T, OmniBusError>;

#[derive(Debug, Clone)]
pub struct Config {
    pub exchanges: HashMap<String, ExchangeConfig>,
    pub settings: SettingsConfig,
}

fn default_post_order_delay_ms() -> u64 { 1300 }
fn default_key_iteration_delay_ms() -> u64 { 1100 }
fn default_retry_429_delay_ms() -> u64 { 2000 }
fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    /// Set to false in exchanges.toml to disable this exchange everywhere without deleting its config.
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub name: String,
    pub base_url: String,
    pub ws_url: Option<String>,
    pub auth_method: String,
    pub symbol_format: String,
    pub api_version: Option<String>,
    pub rate_limit_per_second: usize,
    #[serde(default)]
    pub trading_pairs: Vec<String>,
    pub endpoints: EndpointConfig,
    pub orderbook_parse: Option<OrderBookParseConfig>,
    pub websocket_subscribe: Option<WebSocketSubscribeConfig>,
    /// Propagation delay after place_order / cancel_order before next private call.
    #[serde(default = "default_post_order_delay_ms")]
    pub post_order_delay_ms: u64,
    /// Delay between requests when iterating multiple API keys.
    #[serde(default = "default_key_iteration_delay_ms")]
    pub key_iteration_delay_ms: u64,
    /// Backoff delay when the exchange returns 429 Too Many Requests.
    #[serde(default = "default_retry_429_delay_ms")]
    pub retry_429_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub ticker: String,
    pub tickers: Option<String>,
    pub orderbook: String,
    pub trades: String,
    pub kline: Option<String>,
    pub pairs: String,
    pub order_create: String,
    pub order_modify: Option<String>,
    pub order_cancel: String,
    pub open_orders: String,
    pub balances: String,
    pub balance: Option<String>,
    pub order_history: Option<String>,
    pub trade_history: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookParseConfig {
    pub bid_path: String,
    pub ask_path: String,
    pub price_index: usize,
    pub size_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketSubscribeConfig {
    pub ticker_channel: Option<String>,
    pub orderbook_channel: Option<String>,
    pub trades_channel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    pub theme: ThemeConfig,
    pub default_pairs: Vec<String>,
    pub refresh_rate_ms: u64,
    pub orderbook_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub background: String,
    pub panel_bg: String,
    pub border: String,
    pub text: String,
    pub buy_green: String,
    pub sell_red: String,
    pub accent: String,
    pub warning: String,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            default_pairs: vec!["BTC/USD".to_string(), "ETH/USD".to_string()],
            refresh_rate_ms: 1000,
            orderbook_depth: 20,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: "#0a0e1a".to_string(),
            panel_bg: "#111827".to_string(),
            border: "#253348".to_string(),
            text: "#e0e0e0".to_string(),
            buy_green: "#00c853".to_string(),
            sell_red: "#ff1744".to_string(),
            accent: "#2196f3".to_string(),
            warning: "#ffb347".to_string(),
        }
    }
}

impl ExchangeConfig {
    pub fn rate_delay(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.key_iteration_delay_ms)
    }
    pub fn post_order_delay(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.post_order_delay_ms)
    }
    pub fn retry_429_delay(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.retry_429_delay_ms)
    }
}

/// Wrapper that serde can deserialize directly from the TOML file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawConfig {
    #[serde(default)]
    lcx: Option<ExchangeConfig>,
    #[serde(default)]
    kraken: Option<ExchangeConfig>,
    #[serde(default)]
    coinbase: Option<ExchangeConfig>,
    #[serde(default)]
    settings: SettingsConfig,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| OmniBusError::Config(format!("Failed to read config file: {}", e)))?;

        let raw: RawConfig = toml::from_str(&contents)
            .map_err(|e| OmniBusError::Config(format!("Failed to parse config: {}", e)))?;

        let mut exchanges = HashMap::new();

        if let Some(ex) = raw.lcx {
            if ex.enabled {
                tracing::info!("Loaded exchange: lcx ({} pairs)", ex.trading_pairs.len());
                exchanges.insert("lcx".to_string(), ex);
            } else {
                tracing::info!("Exchange lcx is disabled (enabled=false) — skipping");
            }
        }
        if let Some(ex) = raw.kraken {
            if ex.enabled {
                tracing::info!("Loaded exchange: kraken ({} pairs)", ex.trading_pairs.len());
                exchanges.insert("kraken".to_string(), ex);
            } else {
                tracing::info!("Exchange kraken is disabled (enabled=false) — skipping");
            }
        }
        if let Some(ex) = raw.coinbase {
            if ex.enabled {
                tracing::info!("Loaded exchange: coinbase ({} pairs)", ex.trading_pairs.len());
                exchanges.insert("coinbase".to_string(), ex);
            } else {
                tracing::info!("Exchange coinbase is disabled (enabled=false) — skipping");
            }
        }

        tracing::info!("Loaded {} exchanges from config", exchanges.len());
        Ok(Self { exchanges, settings: raw.settings })
    }

    pub fn get_exchange(&self, name: &str) -> Option<&ExchangeConfig> {
        self.exchanges.get(name)
    }
}