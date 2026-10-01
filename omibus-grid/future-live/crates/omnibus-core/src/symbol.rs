use std::collections::HashMap;

pub struct SymbolNormalizer {
    aliases: HashMap<String, String>,
}

impl SymbolNormalizer {
    pub fn new() -> Self {
        let mut aliases = HashMap::new();
        aliases.insert("XBT".to_string(), "BTC".to_string());
        aliases.insert("XXBT".to_string(), "BTC".to_string());
        aliases.insert("XETH".to_string(), "ETH".to_string());
        aliases.insert("ZUSD".to_string(), "USD".to_string());
        aliases.insert("ZEUR".to_string(), "EUR".to_string());
        Self { aliases }
    }

    /// Normalize to canonical format: BTC/USDT
    pub fn normalize(&self, symbol: &str) -> String {
        let s = symbol.to_uppercase().replace('-', "/").replace('_', "/");
        // If no separator found, try to split at common quote currencies
        if !s.contains('/') {
            for quote in &["USDT", "USDC", "USD", "EUR", "BTC", "ETH"] {
                if s.ends_with(quote) {
                    let base = &s[..s.len() - quote.len()];
                    if !base.is_empty() {
                        let base_norm = self.aliases.get(base).cloned().unwrap_or(base.to_string());
                        return format!("{}/{}", base_norm, quote);
                    }
                }
            }
        }
        s
    }

    /// Convert canonical BTC/USDT to exchange-specific format
    pub fn to_exchange_format(&self, canonical: &str, format: &str) -> String {
        let parts: Vec<&str> = canonical.split('/').collect();
        if parts.len() != 2 { return canonical.to_string(); }
        let (base, quote) = (parts[0], parts[1]);
        match format {
            "concat" => format!("{}{}", base, quote),
            "hyphen" => format!("{}-{}", base, quote),
            "underscore" => format!("{}_{}", base, quote),
            "slash" => format!("{}/{}", base, quote),
            "lower" => format!("{}{}", base.to_lowercase(), quote.to_lowercase()),
            "lower_underscore" => format!("{}_{}", base.to_lowercase(), quote.to_lowercase()),
            _ => canonical.to_string(),
        }
    }
}
