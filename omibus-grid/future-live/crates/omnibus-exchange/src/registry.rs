use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use omnibus_core::*;
use crate::{lcx::LCXExchange, kraken::KrakenExchange, coinbase::CoinbaseExchange};

pub struct ExchangeRegistry {
    exchanges: RwLock<HashMap<String, Arc<dyn Exchange>>>,
    public_apis: RwLock<HashMap<String, Arc<dyn PublicAPI>>>,
    private_apis: RwLock<HashMap<String, Arc<dyn PrivateAPI>>>,
    #[allow(dead_code)]
    websocket_apis: RwLock<HashMap<String, Arc<dyn WebSocketAPI>>>,
    lcx_exchange: RwLock<Option<Arc<LCXExchange>>>,
}

impl ExchangeRegistry {
    pub fn new() -> Self {
        Self {
            exchanges: RwLock::new(HashMap::new()),
            public_apis: RwLock::new(HashMap::new()),
            private_apis: RwLock::new(HashMap::new()),
            websocket_apis: RwLock::new(HashMap::new()),
            lcx_exchange: RwLock::new(None),
        }
    }
    
    pub async fn register(
        &self,
        config: &ExchangeConfig,
        api_key: Option<String>,
        api_secret: Option<String>,
    ) -> Result<(), OmniBusError> {
        let name = config.name.clone();
        let name_lower = name.to_lowercase();

        let has_keys = api_key.is_some() && api_secret.is_some();

        match name_lower.as_str() {
            "lcx" => {
                let exchange = Arc::new(LCXExchange::new(config, api_key, api_secret));
                self.exchanges.write().insert(name.clone(), exchange.clone());
                self.public_apis.write().insert(name.clone(), exchange.clone());
                if has_keys { self.private_apis.write().insert(name.clone(), exchange.clone()); }
                *self.lcx_exchange.write() = Some(exchange);
            }
            "kraken" => {
                let exchange = Arc::new(KrakenExchange::new(config, api_key, api_secret));
                self.exchanges.write().insert(name.clone(), exchange.clone());
                self.public_apis.write().insert(name.clone(), exchange.clone());
                if has_keys { self.private_apis.write().insert(name.clone(), exchange.clone()); }
            }
            "coinbase" => {
                let exchange = Arc::new(CoinbaseExchange::new(config, api_key, api_secret, None));
                self.exchanges.write().insert(name.clone(), exchange.clone());
                self.public_apis.write().insert(name.clone(), exchange.clone());
                if has_keys { self.private_apis.write().insert(name.clone(), exchange.clone()); }
            }
            _ => {
                tracing::warn!("Unknown exchange: {}", name);
            }
        }
        
        Ok(())
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<dyn PublicAPI>> {
        self.public_apis.read().get(name).cloned()
    }
    
    pub fn get_all(&self) -> Vec<Arc<dyn PublicAPI>> {
        self.public_apis.read().values().cloned().collect()
    }
    
    pub fn names(&self) -> Vec<String> {
        self.exchanges.read().keys().cloned().collect()
    }

    pub fn get_private(&self, name: &str) -> Option<Arc<dyn PrivateAPI>> {
        self.private_apis.read().get(name).cloned()
    }

    pub fn get_all_private(&self) -> Vec<(String, Arc<dyn PrivateAPI>)> {
        self.private_apis.read().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Returns the typed LCX exchange instance — needed to build the private WS URL.
    pub fn lcx_exchange(&self) -> Option<Arc<LCXExchange>> {
        self.lcx_exchange.read().clone()
    }
}