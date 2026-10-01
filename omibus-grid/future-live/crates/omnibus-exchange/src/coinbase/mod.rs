mod public;
mod private;
mod auth;
mod websocket;

pub use websocket::CoinbaseWebSocket;

use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use omnibus_core::*;
use crate::rate_limiter::TokenBucket;

#[derive(Clone, Debug, Default)]
pub(crate) struct ProductTick {
    pub quote_increment: f64,
    pub base_increment: f64,
    pub price_decimals: usize,
    pub qty_decimals: usize,
}

pub struct CoinbaseExchange {
    name: String,
    base_url: String,
    ws_url: Option<String>,
    client: Client,
    api_key: Option<String>,
    api_secret: Option<String>,
    trading_pairs: Vec<String>,
    rate_limiter: Arc<RwLock<TokenBucket>>,
    pub(crate) tick_cache: Arc<RwLock<HashMap<String, ProductTick>>>,
}

pub(crate) fn count_decimals(s: &str) -> usize {
    match s.split_once('.') {
        Some((_, dec)) => dec.trim_end_matches('0').len(),
        None => 0,
    }
}

impl CoinbaseExchange {
    pub fn new(config: &ExchangeConfig, api_key: Option<String>, api_secret: Option<String>, _passphrase: Option<String>) -> Self {
        Self {
            name: config.name.clone(),
            base_url: config.base_url.clone(),
            ws_url: config.ws_url.clone(),
            trading_pairs: config.trading_pairs.clone(),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
            api_secret,
            rate_limiter: Arc::new(RwLock::new(TokenBucket::new(config.rate_limit_per_second))),
            tick_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Fetch and cache (quote_increment, base_increment) for a product.
    /// Coinbase rejects orders whose price/qty don't align with these increments.
    pub(crate) async fn get_tick(&self, product_id: &str) -> Result<ProductTick, OmniBusError> {
        if let Some(t) = self.tick_cache.read().get(product_id) {
            return Ok(t.clone());
        }
        let path = format!("/api/v3/brokerage/market/products/{}", product_id);
        let resp: serde_json::Value = self.request("GET", &path, None, false).await?;
        let q_str = resp.get("quote_increment").and_then(|v| v.as_str()).unwrap_or("0.01").to_string();
        let b_str = resp.get("base_increment").and_then(|v| v.as_str()).unwrap_or("0.00000001").to_string();
        let tick = ProductTick {
            quote_increment: q_str.parse().unwrap_or(0.01),
            base_increment: b_str.parse().unwrap_or(0.00000001),
            price_decimals: count_decimals(&q_str),
            qty_decimals: count_decimals(&b_str),
        };
        tracing::info!("Coinbase tick {}: quote_inc={} ({} dec), base_inc={} ({} dec)",
            product_id, q_str, tick.price_decimals, b_str, tick.qty_decimals);
        self.tick_cache.write().insert(product_id.to_string(), tick.clone());
        Ok(tick)
    }
    
    /// Normalize symbol for Coinbase Advanced Trade:
    /// - replaces '/' with '-'
    /// - remaps pairs where fiat USD isn't available but USDC is
    pub fn normalize_symbol(symbol: &str) -> String {
        let s = symbol.replace('/', "-");
        match s.as_str() {
            "LCX-USD" => "LCX-USDC".to_string(),
            _ => s,
        }
    }

    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
        signed: bool,
    ) -> Result<T, OmniBusError> {
        self.rate_limiter.write().acquire();

        let url = format!("{}{}", self.base_url, path);
        let body_str = body.as_ref().map(|b| b.to_string()).unwrap_or_default();

        let mut request = self.client.request(
            reqwest::Method::from_bytes(method.as_bytes()).unwrap(),
            &url,
        );

        if signed {
            let api_key = self.api_key.as_ref().ok_or_else(|| OmniBusError::Auth("Missing API key".to_string()))?;
            let api_secret = self.api_secret.as_ref().ok_or_else(|| OmniBusError::Auth("Missing API secret".to_string()))?;

            if auth::is_cloud_api_key(api_key) {
                // Cloud API — JWT ES256 Bearer token
                let jwt = auth::create_jwt(api_key, method, path, api_secret)?;
                request = request.header("Authorization", format!("Bearer {}", jwt));
            } else {
                // Legacy HMAC
                let timestamp = chrono::Utc::now().timestamp();
                let signature = auth::sign_request_hmac(timestamp, method, path, &body_str, api_secret)?;
                request = request
                    .header("CB-ACCESS-KEY", api_key)
                    .header("CB-ACCESS-SIGN", signature)
                    .header("CB-ACCESS-TIMESTAMP", timestamp.to_string());
            }
        }
        
        if let Some(body) = body {
            request = request.json(&body);
        }
        
        let response = request.send().await?;
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OmniBusError::ExchangeError {
                code: status.to_string(),
                message: error_text,
            });
        }
        
        let data = response.json::<T>().await?;
        Ok(data)
    }
}


#[async_trait]
impl Exchange for CoinbaseExchange {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn base_url(&self) -> &str {
        &self.base_url
    }
    
    fn ws_url(&self) -> Option<&str> {
        self.ws_url.as_deref()
    }
    
    fn supported_symbols(&self) -> Vec<String> {
        self.trading_pairs.clone()
    }
    
    async fn health_check(&self) -> Result<bool, OmniBusError> {
        match self.ticker("BTC-USD").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}