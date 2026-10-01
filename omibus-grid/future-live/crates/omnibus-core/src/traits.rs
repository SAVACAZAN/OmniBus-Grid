use async_trait::async_trait;
use crate::{OmniBusError, Ticker, OrderBook, Trade, Candle, Balance, Order, Side, OrderType, PairInfo};

#[async_trait]
pub trait Exchange: Send + Sync {
    fn name(&self) -> &str;
    fn base_url(&self) -> &str;
    fn ws_url(&self) -> Option<&str>;
    fn supported_symbols(&self) -> Vec<String>;
    async fn health_check(&self) -> Result<bool, OmniBusError>;
}

#[async_trait]
pub trait PublicAPI: Exchange {
    async fn ticker(&self, symbol: &str) -> Result<Ticker, OmniBusError>;
    async fn tickers(&self, symbols: &[String]) -> Result<Vec<Ticker>, OmniBusError>;
    async fn orderbook(&self, symbol: &str, depth: usize) -> Result<OrderBook, OmniBusError>;
    async fn trades(&self, symbol: &str, limit: usize) -> Result<Vec<Trade>, OmniBusError>;
    async fn candles(&self, symbol: &str, interval: &str, limit: usize) -> Result<Vec<Candle>, OmniBusError>;
    async fn markets(&self) -> Result<Vec<String>, OmniBusError>;
    async fn pair_info(&self, symbol: &str) -> Result<PairInfo, OmniBusError> {
        let _ = symbol;
        Ok(PairInfo::default())
    }
}

#[async_trait]
pub trait PrivateAPI: Exchange {
    async fn balances(&self) -> Result<Vec<Balance>, OmniBusError>;
    async fn balance(&self, coin: &str) -> Result<Balance, OmniBusError>;
    async fn place_order(&self, symbol: &str, side: Side, order_type: OrderType, qty: f64, price: Option<f64>) -> Result<Order, OmniBusError>;
    async fn cancel_order(&self, order_id: &str) -> Result<bool, OmniBusError>;
    async fn open_orders(&self) -> Result<Vec<Order>, OmniBusError>;
    async fn order_history(&self, limit: usize) -> Result<Vec<Order>, OmniBusError>;
    async fn trade_history(&self, limit: usize) -> Result<Vec<Trade>, OmniBusError>;
}

pub type TickerCallback = Box<dyn Fn(Ticker) + Send + Sync>;
pub type OrderBookCallback = Box<dyn Fn(OrderBook) + Send + Sync>;
pub type TradeCallback = Box<dyn Fn(Trade) + Send + Sync>;

#[async_trait]
pub trait WebSocketAPI: Send + Sync {
    async fn connect(&mut self) -> Result<(), OmniBusError>;
    async fn disconnect(&mut self) -> Result<(), OmniBusError>;
    async fn subscribe_ticker(&mut self, symbol: &str, callback: TickerCallback) -> Result<(), OmniBusError>;
    async fn subscribe_orderbook(&mut self, symbol: &str, callback: OrderBookCallback) -> Result<(), OmniBusError>;
    async fn subscribe_trades(&mut self, symbol: &str, callback: TradeCallback) -> Result<(), OmniBusError>;
    async fn unsubscribe_ticker(&mut self, symbol: &str) -> Result<(), OmniBusError>;
    async fn unsubscribe_orderbook(&mut self, symbol: &str) -> Result<(), OmniBusError>;
    async fn unsubscribe_trades(&mut self, symbol: &str) -> Result<(), OmniBusError>;
}