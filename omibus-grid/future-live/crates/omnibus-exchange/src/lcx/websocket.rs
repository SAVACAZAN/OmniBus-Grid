use async_trait::async_trait;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use omnibus_core::*;
use super::LCXExchange;
use tokio::sync::broadcast;

type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
type WsWrite = futures_util::stream::SplitSink<WsStream, Message>;

pub struct LCXWebSocket {
    exchange: Arc<LCXExchange>,
    connected: bool,
    write: Option<WsWrite>,
    ticker_callbacks: CallbackStore<Ticker>,
    orderbook_callbacks: CallbackStore<OrderBook>,
    trade_callbacks: CallbackStore<Trade>,
}

impl LCXWebSocket {
    pub fn new(exchange: Arc<LCXExchange>) -> Self {
        Self {
            exchange,
            connected: false,
            write: None,
            ticker_callbacks: CallbackStore::new(),
            orderbook_callbacks: CallbackStore::new(),
            trade_callbacks: CallbackStore::new(),
        }
    }
}

fn parse_ticker(exchange_name: &str, symbol: &str, data: &serde_json::Value) -> Result<Ticker, OmniBusError> {
    let start = std::time::Instant::now();
    Ok(Ticker {
        exchange: exchange_name.to_string(),
        symbol: symbol.to_string(),
        last: data["last"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
        bid: data["buy"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
        ask: data["sell"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
        high_24h: data["high"].as_str().and_then(|s| s.parse().ok()),
        low_24h: data["low"].as_str().and_then(|s| s.parse().ok()),
        volume_24h: data["vol"].as_str().and_then(|s| s.parse().ok()),
        change_24h_pct: None,
        timestamp: chrono::Utc::now().timestamp_millis(),
        latency_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

fn parse_orderbook(exchange_name: &str, symbol: &str, data: &serde_json::Value) -> Result<OrderBook, OmniBusError> {
    let start = std::time::Instant::now();
    let mut orderbook = OrderBook::new(exchange_name.to_string(), symbol.to_string());

    if let Some(bids) = data["buy"].as_array() {
        for bid in bids.iter().take(20) {
            if let Some(ps) = bid.as_array() {
                orderbook.bids.push(OrderBookLevel {
                    price: ps[0].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    size:  ps[1].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                });
            }
        }
    }

    if let Some(asks) = data["sell"].as_array() {
        for ask in asks.iter().take(20) {
            if let Some(ps) = ask.as_array() {
                orderbook.asks.push(OrderBookLevel {
                    price: ps[0].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    size:  ps[1].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                });
            }
        }
    }

    orderbook.calculate_metrics();
    orderbook.latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    Ok(orderbook)
}

fn parse_trade(exchange_name: &str, symbol: &str, data: &serde_json::Value) -> Result<Trade, OmniBusError> {
    Ok(Trade {
        id: data["id"].as_str().unwrap_or("").to_string(),
        exchange: exchange_name.to_string(),
        symbol: symbol.to_string(),
        side: if data["type"].as_str() == Some("buy") { Side::Buy } else { Side::Sell },
        price: data["price"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
        quantity: data["volume"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
        timestamp: data["timestamp"].as_i64().unwrap_or(0),
        fee: None,
        fee_coin: None,
    })
}

fn dispatch_message(
    exchange_name: &str,
    text: &str,
    ticker_cbs: &CallbackStore<Ticker>,
    orderbook_cbs: &CallbackStore<OrderBook>,
    trade_cbs: &CallbackStore<Trade>,
) {
    #[derive(serde::Deserialize)]
    struct WSMessage {
        #[allow(dead_code)]
        event: Option<String>,
        channel: String,
        pair: String,
        data: serde_json::Value,
    }

    if let Ok(msg) = serde_json::from_str::<WSMessage>(text) {
        match msg.channel.as_str() {
            "ticker" => {
                if let Ok(ticker) = parse_ticker(exchange_name, &msg.pair, &msg.data) {
                    ticker_cbs.invoke(&msg.pair, ticker);
                }
            }
            "orderbook" => {
                if let Ok(orderbook) = parse_orderbook(exchange_name, &msg.pair, &msg.data) {
                    orderbook_cbs.invoke(&msg.pair, orderbook);
                }
            }
            "trades" => {
                if let Ok(trade) = parse_trade(exchange_name, &msg.pair, &msg.data) {
                    trade_cbs.invoke(&msg.pair, trade);
                }
            }
            _ => {}
        }
    }
}

#[async_trait]
impl WebSocketAPI for LCXWebSocket {
    async fn connect(&mut self) -> Result<(), OmniBusError> {
        let ws_url = self.exchange.ws_url()
            .ok_or_else(|| OmniBusError::WebSocket("No WebSocket URL configured".to_string()))?
            .to_string();

        let (stream, _) = connect_async(&ws_url).await
            .map_err(|e| OmniBusError::WebSocket(format!("Connection failed: {}", e)))?;

        let (write, mut read) = stream.split();
        self.write = Some(write);
        self.connected = true;

        let ticker_cbs    = self.ticker_callbacks.clone();
        let orderbook_cbs = self.orderbook_callbacks.clone();
        let trade_cbs     = self.trade_callbacks.clone();
        let exchange_name = self.exchange.name().to_string();

        tokio::spawn(async move {
            tracing::info!("LCX WebSocket receive loop started");
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        dispatch_message(&exchange_name, &text, &ticker_cbs, &orderbook_cbs, &trade_cbs);
                    }
                    Ok(Message::Ping(payload)) => {
                        tracing::debug!("LCX WS ping received");
                        let _ = payload; // pong is handled automatically by tungstenite
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("LCX WebSocket closed by server");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("LCX WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            tracing::info!("LCX WebSocket receive loop ended");
        });

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), OmniBusError> {
        if let Some(mut write) = self.write.take() {
            let _ = write.close().await;
        }
        self.connected = false;
        Ok(())
    }

    async fn subscribe_ticker(&mut self, symbol: &str, callback: TickerCallback) -> Result<(), OmniBusError> {
        let msg = serde_json::json!({"event": "subscribe", "channel": "ticker", "pair": symbol});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Subscribe failed: {}", e)))?;
        }
        self.ticker_callbacks.push(symbol, callback);
        Ok(())
    }

    async fn subscribe_orderbook(&mut self, symbol: &str, callback: OrderBookCallback) -> Result<(), OmniBusError> {
        let msg = serde_json::json!({"event": "subscribe", "channel": "orderbook", "pair": symbol});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Subscribe failed: {}", e)))?;
        }
        self.orderbook_callbacks.push(symbol, callback);
        Ok(())
    }

    async fn subscribe_trades(&mut self, symbol: &str, callback: TradeCallback) -> Result<(), OmniBusError> {
        let msg = serde_json::json!({"event": "subscribe", "channel": "trades", "pair": symbol});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Subscribe failed: {}", e)))?;
        }
        self.trade_callbacks.push(symbol, callback);
        Ok(())
    }

    async fn unsubscribe_ticker(&mut self, symbol: &str) -> Result<(), OmniBusError> {
        let msg = serde_json::json!({"event": "unsubscribe", "channel": "ticker", "pair": symbol});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Unsubscribe failed: {}", e)))?;
        }
        self.ticker_callbacks.remove(symbol);
        Ok(())
    }

    async fn unsubscribe_orderbook(&mut self, symbol: &str) -> Result<(), OmniBusError> {
        let msg = serde_json::json!({"event": "unsubscribe", "channel": "orderbook", "pair": symbol});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Unsubscribe failed: {}", e)))?;
        }
        self.orderbook_callbacks.remove(symbol);
        Ok(())
    }

    async fn unsubscribe_trades(&mut self, symbol: &str) -> Result<(), OmniBusError> {
        let msg = serde_json::json!({"event": "unsubscribe", "channel": "trades", "pair": symbol});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Unsubscribe failed: {}", e)))?;
        }
        self.trade_callbacks.remove(symbol);
        Ok(())
    }
}

// ── LCX Private WebSocket ────────────────────────────────────────────────────
//
// Connects to wss://exchange-api.lcx.com/api/auth/ws?x-access-key=…&…
// Subscribes to `user_orders` and `user_trades` channels.
// Emits OrderUpdateEvent on `order_tx` for every status change.

pub struct LCXPrivateWebSocket {
    exchange: Arc<LCXExchange>,
    pub order_tx: broadcast::Sender<OrderUpdateEvent>,
}

impl LCXPrivateWebSocket {
    pub fn new(exchange: Arc<LCXExchange>) -> Self {
        let (order_tx, _) = broadcast::channel(256);
        Self { exchange, order_tx }
    }

    /// Connect, subscribe, and spawn the receive loop.
    /// Returns immediately — events flow through `order_tx`.
    pub async fn connect(&self) -> Result<(), OmniBusError> {
        let ws_url = self.exchange.build_private_ws_auth()?;
        let (stream, _) = connect_async(&ws_url).await
            .map_err(|e| OmniBusError::WebSocket(format!("LCX private WS connect failed: {}", e)))?;

        let (mut write, mut read) = stream.split();

        // Auth is in URL query params — subscribe to private channels
        for (topic, ch_type) in [("subscribe", "user_wallets"), ("subscribe", "user_orders"), ("update", "user_trades")] {
            let sub = serde_json::json!({"Topic": topic, "Type": ch_type});
            write.send(Message::Text(sub.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("LCX private WS subscribe failed: {}", e)))?;
        }

        let order_tx = self.order_tx.clone();
        let exchange_name = self.exchange.name().to_string();

        tokio::spawn(async move {
            tracing::info!("LCX private WebSocket receive loop started");
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        dispatch_private_message(&exchange_name, &text, &order_tx);
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("LCX private WebSocket closed by server");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("LCX private WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            tracing::info!("LCX private WebSocket receive loop ended");
        });

        Ok(())
    }
}

fn dispatch_private_message(
    exchange_name: &str,
    text: &str,
    order_tx: &broadcast::Sender<OrderUpdateEvent>,
) {
    let data: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return,
    };

    let msg_type = match data["Type"].as_str() {
        Some(t) => t,
        None => return,
    };

    match msg_type {
        "user_orders" => {
            if let Some(evt) = parse_order_update(exchange_name, &data["Data"]) {
                let _ = order_tx.send(evt);
            }
        }
        "user_trades" => {
            // user_trades confirms a fill — treat as Filled status on the matched order
            if let Some(evt) = parse_trade_fill(exchange_name, &data["Data"]) {
                let _ = order_tx.send(evt);
            }
        }
        _ => tracing::debug!("LCX private WS unhandled type: {}", msg_type),
    }
}

fn parse_order_update(exchange_name: &str, data: &serde_json::Value) -> Option<OrderUpdateEvent> {
    let order_id = data["OrderId"].as_str().or_else(|| data["orderId"].as_str())?.to_string();
    let pair = data["Pair"].as_str().or_else(|| data["pair"].as_str()).unwrap_or("").to_string();
    let side_str = data["Side"].as_str().or_else(|| data["side"].as_str()).unwrap_or("buy");
    let side = if side_str.eq_ignore_ascii_case("sell") { Side::Sell } else { Side::Buy };

    let status_str = data["Status"].as_str().or_else(|| data["status"].as_str()).unwrap_or("OPEN");
    let status = match status_str.to_uppercase().as_str() {
        "CLOSED" | "FILLED" | "COMPLETED" => OrderStatus::Filled,
        "PARTIALLY_FILLED" | "PARTIALLYMATCHED" => OrderStatus::PartiallyFilled,
        "CANCELLED" | "CANCELED" => OrderStatus::Cancelled,
        _ => OrderStatus::Open,
    };

    let filled_qty: f64 = data["FilledQuantity"].as_str()
        .or_else(|| data["filledQuantity"].as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    let price: f64 = data["Price"].as_str()
        .or_else(|| data["price"].as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    Some(OrderUpdateEvent {
        exchange: exchange_name.to_string(),
        order_id,
        pair,
        side,
        status,
        filled_qty,
        price,
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}

fn parse_trade_fill(exchange_name: &str, data: &serde_json::Value) -> Option<OrderUpdateEvent> {
    let order_id = data["OrderId"].as_str().or_else(|| data["orderId"].as_str())?.to_string();
    let pair = data["Pair"].as_str().or_else(|| data["pair"].as_str()).unwrap_or("").to_string();
    let side_str = data["Side"].as_str().or_else(|| data["side"].as_str()).unwrap_or("buy");
    let side = if side_str.eq_ignore_ascii_case("sell") { Side::Sell } else { Side::Buy };
    let qty: f64 = data["Quantity"].as_str()
        .or_else(|| data["quantity"].as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let price: f64 = data["Price"].as_str()
        .or_else(|| data["price"].as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    Some(OrderUpdateEvent {
        exchange: exchange_name.to_string(),
        order_id,
        pair,
        side,
        status: OrderStatus::Filled,
        filled_qty: qty,
        price,
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}
