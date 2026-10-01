use async_trait::async_trait;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use omnibus_core::*;
use super::KrakenExchange;

type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
type WsWrite = futures_util::stream::SplitSink<WsStream, Message>;

pub struct KrakenWebSocket {
    exchange: Arc<KrakenExchange>,
    connected: bool,
    write: Option<WsWrite>,
    ticker_callbacks: CallbackStore<Ticker>,
    orderbook_callbacks: CallbackStore<OrderBook>,
    trade_callbacks: CallbackStore<Trade>,
}

impl KrakenWebSocket {
    pub fn new(exchange: Arc<KrakenExchange>) -> Self {
        Self {
            exchange,
            connected: false,
            write: None,
            ticker_callbacks: CallbackStore::new(),
            orderbook_callbacks: CallbackStore::new(),
            trade_callbacks: CallbackStore::new(),
        }
    }

    /// Convert canonical symbol to Kraken WS pair format.
    /// "BTC/USD" → "XBT/USD", "ETH/EUR" → "ETH/EUR"
    fn to_kraken_pair(symbol: &str) -> String {
        match symbol.split_once('/') {
            Some((base, quote)) => {
                let kraken_base = match base {
                    "BTC" => "XBT",
                    other => other,
                };
                format!("{}/{}", kraken_base, quote)
            }
            None => symbol.to_string(),
        }
    }
}

/// Kraken sends pair names like "XBT/USD" — normalize to canonical "BTC/USD".
fn normalize_pair(pair: &str) -> String {
    SymbolNormalizer::new().normalize(pair)
}

/// Kraken WS ticker: object with fields a[0]=ask, b[0]=bid, c[0]=last,
/// h[1]=24h high, l[1]=24h low, v[1]=24h volume.
fn parse_ticker(exchange_name: &str, pair: &str, data: &serde_json::Value) -> Option<Ticker> {
    let start = std::time::Instant::now();
    let last: f64 = data["c"][0].as_str()?.parse().ok()?;
    Some(Ticker {
        exchange: exchange_name.to_string(),
        symbol: normalize_pair(pair),
        last,
        bid: data["b"][0].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
        ask: data["a"][0].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
        high_24h: data["h"][1].as_str().and_then(|s| s.parse().ok()),
        low_24h:  data["l"][1].as_str().and_then(|s| s.parse().ok()),
        volume_24h: data["v"][1].as_str().and_then(|s| s.parse().ok()),
        change_24h_pct: None,
        timestamp: chrono::Utc::now().timestamp_millis(),
        latency_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

/// Kraken WS trades: array of [price, volume, time, side, orderType, misc].
/// `side`: "s" = sell, "b" = buy.
fn parse_trades(exchange_name: &str, pair: &str, data: &serde_json::Value) -> Vec<Trade> {
    let canonical = normalize_pair(pair);
    data.as_array().unwrap_or(&vec![]).iter().filter_map(|t| {
        let arr = t.as_array()?;
        Some(Trade {
            id: String::new(),
            exchange: exchange_name.to_string(),
            symbol: canonical.clone(),
            side: if arr.get(3).and_then(|v| v.as_str()) == Some("s") { Side::Sell } else { Side::Buy },
            price:    arr.get(0).and_then(|v| v.as_str())?.parse().ok()?,
            quantity: arr.get(1).and_then(|v| v.as_str())?.parse().ok()?,
            timestamp: arr.get(2).and_then(|v| v.as_f64()).map(|t| (t * 1000.0) as i64).unwrap_or(0),
            fee: None,
            fee_coin: None,
        })
    }).collect()
}

/// Kraken WS orderbook: snapshot uses "as"/"bs", updates use "a"/"b".
/// Each level: [price, volume, timestamp].
fn parse_orderbook(exchange_name: &str, pair: &str, data: &serde_json::Value) -> Option<OrderBook> {
    let start = std::time::Instant::now();
    let mut orderbook = OrderBook::new(exchange_name.to_string(), normalize_pair(pair));

    let bids_key = if data.get("bs").is_some() { "bs" } else { "b" };
    let asks_key = if data.get("as").is_some() { "as" } else { "a" };

    for entry in [("bids", bids_key, true), ("asks", asks_key, false)] {
        if let Some(levels) = data[entry.1].as_array() {
            for level in levels.iter().take(25) {
                if let Some(arr) = level.as_array() {
                    let price:  f64 = arr.get(0).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    let size:   f64 = arr.get(1).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    let ol = OrderBookLevel { price, size };
                    if entry.2 { orderbook.bids.push(ol); } else { orderbook.asks.push(ol); }
                }
            }
        }
    }

    if orderbook.bids.is_empty() && orderbook.asks.is_empty() {
        return None;
    }
    orderbook.calculate_metrics();
    orderbook.latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    Some(orderbook)
}

/// Kraken data messages are JSON arrays: [channelID, data, channelName, pair]
/// Control messages are JSON objects (heartbeat, subscriptionStatus, etc.) — ignored here.
fn dispatch_message(
    exchange_name: &str,
    text: &str,
    ticker_cbs:    &CallbackStore<Ticker>,
    orderbook_cbs: &CallbackStore<OrderBook>,
    trade_cbs:     &CallbackStore<Trade>,
) {
    let value: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return,
    };

    let arr = match value.as_array() {
        Some(a) if a.len() >= 4 => a,
        _ => return,
    };

    let channel = match arr[arr.len() - 2].as_str() { Some(c) => c, None => return };
    let pair    = match arr[arr.len() - 1].as_str() { Some(p) => p, None => return };
    let canonical = normalize_pair(pair);
    let data = &arr[1];

    match channel {
        "ticker" => {
            if let Some(ticker) = parse_ticker(exchange_name, pair, data) {
                ticker_cbs.invoke(&canonical, ticker);
            }
        }
        c if c.starts_with("book") => {
            if let Some(ob) = parse_orderbook(exchange_name, pair, data) {
                orderbook_cbs.invoke(&canonical, ob);
            }
        }
        "trade" => {
            for trade in parse_trades(exchange_name, pair, data) {
                trade_cbs.invoke(&canonical, trade);
            }
        }
        _ => {}
    }
}

#[async_trait]
impl WebSocketAPI for KrakenWebSocket {
    async fn connect(&mut self) -> Result<(), OmniBusError> {
        let ws_url = self.exchange.ws_url()
            .ok_or_else(|| OmniBusError::WebSocket("No WebSocket URL configured".to_string()))?
            .to_string();

        let (stream, _) = connect_async(&ws_url).await
            .map_err(|e| OmniBusError::WebSocket(format!("Connection failed: {}", e)))?;

        let (mut write, mut read) = stream.split();
        self.connected = true;

        // Subscribe to heartbeat to keep connection alive
        let heartbeat = serde_json::json!({"event": "subscribe", "subscription": {"name": "heartbeat"}});
        write.send(Message::Text(heartbeat.to_string())).await
            .map_err(|e| OmniBusError::WebSocket(format!("Heartbeat subscribe failed: {}", e)))?;

        self.write = Some(write);

        let ticker_cbs    = self.ticker_callbacks.clone();
        let orderbook_cbs = self.orderbook_callbacks.clone();
        let trade_cbs     = self.trade_callbacks.clone();
        let exchange_name = self.exchange.name().to_string();

        tokio::spawn(async move {
            tracing::info!("Kraken WebSocket receive loop started");
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        dispatch_message(&exchange_name, &text, &ticker_cbs, &orderbook_cbs, &trade_cbs);
                    }
                    Ok(Message::Ping(_)) => {
                        tracing::debug!("Kraken WS ping");
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("Kraken WebSocket closed by server");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Kraken WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            tracing::info!("Kraken WebSocket receive loop ended");
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
        let kraken_symbol = Self::to_kraken_pair(symbol);
        let msg = serde_json::json!({"event": "subscribe", "pair": [kraken_symbol], "subscription": {"name": "ticker"}});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Subscribe failed: {}", e)))?;
        }
        self.ticker_callbacks.push(symbol, callback);
        Ok(())
    }

    async fn subscribe_orderbook(&mut self, symbol: &str, callback: OrderBookCallback) -> Result<(), OmniBusError> {
        let kraken_symbol = Self::to_kraken_pair(symbol);
        let msg = serde_json::json!({"event": "subscribe", "pair": [kraken_symbol], "subscription": {"name": "book", "depth": 25}});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Subscribe failed: {}", e)))?;
        }
        self.orderbook_callbacks.push(symbol, callback);
        Ok(())
    }

    async fn subscribe_trades(&mut self, symbol: &str, callback: TradeCallback) -> Result<(), OmniBusError> {
        let kraken_symbol = Self::to_kraken_pair(symbol);
        let msg = serde_json::json!({"event": "subscribe", "pair": [kraken_symbol], "subscription": {"name": "trade"}});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Subscribe failed: {}", e)))?;
        }
        self.trade_callbacks.push(symbol, callback);
        Ok(())
    }

    async fn unsubscribe_ticker(&mut self, symbol: &str) -> Result<(), OmniBusError> {
        let kraken_symbol = Self::to_kraken_pair(symbol);
        let msg = serde_json::json!({"event": "unsubscribe", "pair": [kraken_symbol], "subscription": {"name": "ticker"}});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Unsubscribe failed: {}", e)))?;
        }
        self.ticker_callbacks.remove(symbol);
        Ok(())
    }

    async fn unsubscribe_orderbook(&mut self, symbol: &str) -> Result<(), OmniBusError> {
        let kraken_symbol = Self::to_kraken_pair(symbol);
        let msg = serde_json::json!({"event": "unsubscribe", "pair": [kraken_symbol], "subscription": {"name": "book"}});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Unsubscribe failed: {}", e)))?;
        }
        self.orderbook_callbacks.remove(symbol);
        Ok(())
    }

    async fn unsubscribe_trades(&mut self, symbol: &str) -> Result<(), OmniBusError> {
        let kraken_symbol = Self::to_kraken_pair(symbol);
        let msg = serde_json::json!({"event": "unsubscribe", "pair": [kraken_symbol], "subscription": {"name": "trade"}});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Unsubscribe failed: {}", e)))?;
        }
        self.trade_callbacks.remove(symbol);
        Ok(())
    }
}
