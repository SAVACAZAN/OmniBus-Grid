use thiserror::Error;

#[derive(Debug, Error)]
pub enum OmniBusError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("JSON parsing error: {0}")]
    Json(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Rate limit exceeded for {exchange}")]
    RateLimit { exchange: String },

    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),

    #[error("Exchange error: {0}")]
    Exchange(String),

    #[error("Exchange error: {code} - {message}")]
    ExchangeError { code: String, message: String },

    #[error("Timeout error for {operation}")]
    Timeout { operation: String },

    #[error("IO error: {0}")]
    Io(String),

    #[error("Channel error: {0}")]
    Channel(String),

    #[error("Invalid order: {0}")]
    InvalidOrder(String),

    #[error("Insufficient balance: need {need} {coin}, have {have}")]
    InsufficientBalance { coin: String, need: f64, have: f64 },

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl OmniBusError {
    pub fn is_rate_limit(&self) -> bool {
        match self {
            OmniBusError::RateLimit { .. } => true,
            OmniBusError::ExchangeError { code, message } => {
                code.contains("429")
                    || message.to_lowercase().contains("rate limit")
                    || message.to_lowercase().contains("too many requests")
            }
            OmniBusError::Exchange(msg) => {
                msg.contains("429") || msg.to_lowercase().contains("rate limit")
            }
            _ => false,
        }
    }

    pub fn is_insufficient_balance(&self) -> bool {
        match self {
            OmniBusError::InsufficientBalance { .. } => true,
            OmniBusError::ExchangeError { message, .. } => {
                let m = message.to_lowercase();
                m.contains("insufficient") || m.contains("not enough") || m.contains("balance")
            }
            OmniBusError::Exchange(msg) => {
                let m = msg.to_lowercase();
                m.contains("insufficient") || m.contains("not enough") || m.contains("balance")
            }
            _ => false,
        }
    }
}

impl From<reqwest::Error> for OmniBusError {
    fn from(e: reqwest::Error) -> Self { OmniBusError::Http(e.to_string()) }
}
impl From<serde_json::Error> for OmniBusError {
    fn from(e: serde_json::Error) -> Self { OmniBusError::Json(e.to_string()) }
}
impl From<std::io::Error> for OmniBusError {
    fn from(e: std::io::Error) -> Self { OmniBusError::Io(e.to_string()) }
}

pub type Result<T> = std::result::Result<T, OmniBusError>;