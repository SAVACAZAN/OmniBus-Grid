use serde::{Deserialize, Serialize};
use crate::{OrderBook, OrderStatus, Side};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceEvent {
    pub exchange: String,
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub mid: f64,
    pub last: f64,
    pub volume_24h: Option<f64>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdateEvent {
    pub exchange: String,
    pub order_id: String,
    pub pair: String,
    pub side: Side,
    pub status: OrderStatus,
    pub filled_qty: f64,
    pub price: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEvent {
    pub book: OrderBook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedStatusEvent {
    pub feed: String,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEvent {
    pub exchange: String,
    pub symbol: String,
    pub price: f64,
    pub amount: f64,
    pub side: String,
    pub timestamp: i64,
}
