use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub exchange: String,
    pub symbol: String,
    pub last: f64,
    pub bid: f64,
    pub ask: f64,
    pub high_24h: Option<f64>,
    pub low_24h: Option<f64>,
    pub volume_24h: Option<f64>,
    pub change_24h_pct: Option<f64>,
    pub timestamp: i64,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub exchange: String,
    pub symbol: String,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub spread: f64,
    pub spread_pct: f64,
    pub mid_price: f64,
    pub timestamp: i64,
    pub latency_ms: f64,
}

impl OrderBook {
    pub fn new(exchange: String, symbol: String) -> Self {
        Self {
            exchange,
            symbol,
            bids: Vec::new(),
            asks: Vec::new(),
            spread: 0.0,
            spread_pct: 0.0,
            mid_price: 0.0,
            timestamp: chrono::Utc::now().timestamp_millis(),
            latency_ms: 0.0,
        }
    }
    
    pub fn calculate_metrics(&mut self) {
        if let (Some(best_bid), Some(best_ask)) = (self.bids.first(), self.asks.first()) {
            self.spread = best_ask.price - best_bid.price;
            self.spread_pct = (self.spread / best_ask.price) * 100.0;
            self.mid_price = (best_bid.price + best_ask.price) / 2.0;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub exchange: String,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub filled_qty: f64,
    pub status: OrderStatus,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Side::Buy => "buy",
            Side::Sell => "sell",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub exchange: String,
    pub coin: String,
    pub available: f64,
    pub locked: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub exchange: String,
    pub symbol: String,
    pub side: Side,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_coin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairInfo {
    pub price_decimals: i64,
    pub amount_decimals: i64,
    pub min_base: f64,
    pub min_quote: f64,
}

impl Default for PairInfo {
    fn default() -> Self {
        Self { price_decimals: 5, amount_decimals: 2, min_base: 1.0, min_quote: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub symbol: String,
    pub buy_exchange: String,
    pub buy_price: f64,
    pub sell_exchange: String,
    pub sell_price: f64,
    pub spread_pct: f64,
    pub estimated_profit_usd: f64,
}