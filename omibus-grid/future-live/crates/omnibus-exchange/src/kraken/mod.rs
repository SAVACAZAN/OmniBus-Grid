mod public;
mod private;
mod auth;
mod websocket;

pub use websocket::KrakenWebSocket;

use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use parking_lot::RwLock;
use omnibus_core::*;
use crate::rate_limiter::TokenBucket;

pub struct KrakenExchange {
    name: String,
    base_url: String,
    ws_url: Option<String>,
    client: Client,
    api_key: Option<String>,
    api_secret: Option<String>,
    trading_pairs: Vec<String>,
    rate_limiter: Arc<RwLock<TokenBucket>>,
}

impl KrakenExchange {
    pub fn new(config: &ExchangeConfig, api_key: Option<String>, api_secret: Option<String>) -> Self {
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
        }
    }
    
    fn sign_request(&self, endpoint: &str, data: &str, nonce: u64) -> Result<String, OmniBusError> {
        auth::sign_request(endpoint, data, nonce, self.api_secret.as_ref())
    }
    
    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        data: Option<serde_json::Value>,
        signed: bool,
    ) -> Result<T, OmniBusError> {
        self.rate_limiter.write().acquire();
        
        let url = format!("{}{}", self.base_url, endpoint);
        
        let mut request = self.client.post(&url);
        
        if signed {
            // Kraken nonce must always be higher than previous — use microseconds since UNIX_EPOCH
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64;
            let nonce_str = nonce.to_string();

            // Build form-encoded data like Python: urllib.parse.urlencode({"nonce": "...", ...})
            let mut params: Vec<(String, String)> = Vec::new();
            params.push(("nonce".to_string(), nonce_str.clone()));
            if let Some(ref d) = data {
                if let Some(obj) = d.as_object() {
                    for (k, v) in obj {
                        let val = match v {
                            serde_json::Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        };
                        params.push((k.clone(), val));
                    }
                }
            }
            let encoded: String = params.iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect::<Vec<_>>().join("&");

            let signature = self.sign_request(endpoint, &encoded, nonce)?;

            request = request
                .header("API-Key", self.api_key.as_ref().ok_or_else(|| OmniBusError::Auth("Missing API key".to_string()))?)
                .header("API-Sign", signature)
                .header("Content-Type", "application/x-www-form-urlencoded");

            request = request.body(encoded);
        } else if let Some(data) = data {
            request = request.json(&data);
        }
        
        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;
        
        #[derive(serde::Deserialize)]
        struct KrakenResponse<T> {
            #[serde(default)]
            error: Vec<String>,
            result: Option<T>,
        }

        let kraken_resp: KrakenResponse<T> = serde_json::from_str(&text)
            .map_err(|e| OmniBusError::Json(format!("{} | raw: {}", e, &text[..text.len().min(200)])))?;
        
        if !kraken_resp.error.is_empty() {
            return Err(OmniBusError::ExchangeError {
                code: "KRAKEN_ERROR".to_string(),
                message: kraken_resp.error.join(", "),
            });
        }
        
        kraken_resp.result.ok_or_else(|| OmniBusError::ExchangeError {
            code: status.to_string(),
            message: "No result in response".to_string(),
        })
    }
}


#[async_trait]
impl Exchange for KrakenExchange {
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
        match self.ticker("XBT/USD").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}