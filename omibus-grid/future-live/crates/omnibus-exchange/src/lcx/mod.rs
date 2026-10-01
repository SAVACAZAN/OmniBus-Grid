mod public;
mod private;
mod auth;
mod websocket;

pub use websocket::{LCXWebSocket, LCXPrivateWebSocket};

use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use omnibus_core::*;

pub struct LCXExchange {
    name: String,
    base_url: String,
    ws_url: Option<String>,
    client: Client,
    api_key: Option<String>,
    api_secret: Option<String>,
    trading_pairs: Vec<String>,
    rate_limiter: Arc<tokio::sync::Mutex<RateLimiter>>,
}

impl LCXExchange {
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
            rate_limiter: Arc::new(tokio::sync::Mutex::new(RateLimiter::new(config.rate_limit_per_second))),
        }
    }

    /// Make a diagnostic HTTP POST to /api/auth/ws and return the response.
    /// LCX docs show POST for this endpoint — if it returns a token, the flow is:
    ///   1. POST /api/auth/ws?... → get token
    ///   2. Connect wss://.../ws, send {"Topic":"auth","Token":token}
    pub async fn probe_post_auth_ws(&self) -> String {
        let api_key = match self.api_key.as_ref() {
            Some(k) => k.clone(),
            None => return "no api key".to_string(),
        };
        let timestamp = chrono::Utc::now().timestamp_millis().to_string();
        let base = self.ws_url.as_deref().unwrap_or("wss://exchange-api.lcx.com/ws");
        let base_host = if base.ends_with("/ws") {
            base[..base.len() - 3].trim_end_matches('/').to_string()
        } else {
            base.trim_end_matches('/').to_string()
        };
        let http_base = base_host.replace("wss://", "https://");
        let body_json = "{}";
        let signature = match auth::sign_request("POST", "/api/auth/ws", body_json, self.api_secret.as_ref()) {
            Ok(s) => s,
            Err(e) => return format!("sign error: {}", e),
        };
        let sig_enc = signature.replace('+', "%2B").replace('/', "%2F").replace('=', "%3D");
        let url = format!("{}/api/auth/ws?x-access-key={}&x-access-sign={}&x-access-timestamp={}", http_base, api_key, sig_enc, timestamp);
        // POST probe
        let post_result = match self.client.post(&url).json(&serde_json::json!({})).send().await {
            Ok(resp) => { let s = resp.status(); let b = resp.text().await.unwrap_or_default(); format!("POST {} {}", s.as_u16(), &b[..b.len().min(200)]) }
            Err(e) => format!("POST err:{}", e),
        };

        // GET with fake random base64 signature (to test if 500 is before or after HMAC verify)
        let fake_sign = "ZmFrZXNpZ25hdHVyZWZha2VzaWduYXR1cmU%3D"; // "fakesignaturefakesignature" base64
        let url_fake = format!("{}/api/auth/ws?x-access-key={}&x-access-sign={}&x-access-timestamp={}", http_base, api_key, fake_sign, timestamp);
        let fake_result = match self.client.get(&url_fake).send().await {
            Ok(resp) => { let s = resp.status(); let b = resp.text().await.unwrap_or_default(); format!("GET-fake {} {}", s.as_u16(), &b[..b.len().min(200)]) }
            Err(e) => format!("GET-fake err:{}", e),
        };

        format!("{} | {}", post_result, fake_result)
    }

    /// Returns the authenticated trading WS URL per LCX docs:
    /// wss://exchange-api.lcx.com/api/auth/ws?x-access-key=...&x-access-sign=...&x-access-timestamp=...
    /// Signature covers GET + /api/auth/ws + "{}" (docs: always pass empty object when no payload).
    pub fn build_private_ws_auth(&self) -> Result<String, OmniBusError> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| OmniBusError::Auth("Missing API key".to_string()))?.clone();
        let secret = self.api_secret.as_ref()
            .ok_or_else(|| OmniBusError::Auth("Missing API secret".to_string()))?;
        let timestamp = chrono::Utc::now().timestamp_millis().to_string();
        let signature = auth::sign_request("GET", "/api/auth/ws", "{}", Some(secret))?;
        let base = self.ws_url.as_deref().unwrap_or("wss://exchange-api.lcx.com/ws");
        let base_host = if base.ends_with("/ws") {
            base[..base.len() - 3].trim_end_matches('/').to_string()
        } else {
            base.trim_end_matches('/').to_string()
        };
        // Signature goes raw into the URL — LCX docs show no percent-encoding on the JS side.
        // URL-encoding + → %2B breaks the server's naive query-param comparison → 500.
        Ok(format!(
            "{}/api/auth/ws?x-access-key={}&x-access-sign={}&x-access-timestamp={}",
            base_host, api_key, signature, timestamp
        ))
    }

    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        endpoint: &str,
        body: Option<serde_json::Value>,
        signed: bool,
    ) -> Result<T, OmniBusError> {
        let url = format!("{}{}", self.base_url, endpoint);

        let response = if signed {
            // Hold the mutex for the entire HTTP call (wait + send + receive).
            // This prevents MM and Snipe from sending overlapping private requests
            // to LCX — overlapping requests hit the rate limit even when spaced >200ms.
            let body_str = body.as_ref().map(|b| b.to_string()).unwrap_or_else(|| "{}".to_string());

            let mut rl = self.rate_limiter.lock().await;
            let elapsed = rl.last_request.elapsed();
            if elapsed < rl.min_interval {
                tokio::time::sleep(rl.min_interval - elapsed).await;
            }

            // Compute auth after any wait so the timestamp is always fresh
            let timestamp = chrono::Utc::now().timestamp_millis().to_string();
            let sign_endpoint = endpoint.split('?').next().unwrap_or(endpoint);
            let signature = auth::sign_request_with_timestamp(method, sign_endpoint, &timestamp, &body_str, self.api_secret.as_ref())?;

            let mut req = self.client.request(
                reqwest::Method::from_bytes(method.as_bytes()).unwrap(),
                &url,
            )
            .header("x-access-key", self.api_key.as_ref().ok_or_else(|| OmniBusError::Auth("Missing API key".to_string()))?)
            .header("x-access-sign", signature)
            .header("x-access-timestamp", &timestamp)
            .header("API-VERSION", "1.1.0");

            if let Some(ref b) = body {
                req = req.json(b);
            }

            let result = req.send().await;
            rl.last_request = std::time::Instant::now();
            drop(rl);
            result?
        } else {
            // Public (unsigned) calls are not rate-limited here — they don't trigger 429s
            let mut req = self.client.request(
                reqwest::Method::from_bytes(method.as_bytes()).unwrap(),
                &url,
            );
            if let Some(b) = body {
                req = req.json(&b);
            }
            req.send().await?
        };

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(OmniBusError::ExchangeError {
                code: status.to_string(),
                message: text,
            });
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }
}

// Minimum-interval serializer: at 5 req/s → 200ms + 20ms margin = 220ms between calls.
// Holding the tokio Mutex guard across the sleep queues concurrent callers automatically.
struct RateLimiter {
    last_request: std::time::Instant,
    min_interval: std::time::Duration,
}

impl RateLimiter {
    fn new(_requests_per_second: usize) -> Self {
        let ms = 1000u64;
        Self {
            last_request: std::time::Instant::now() - std::time::Duration::from_secs(60),
            min_interval: std::time::Duration::from_millis(ms),
        }
    }
}

#[async_trait]
impl Exchange for LCXExchange {
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
        match self.ticker("BTC/EUR").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}