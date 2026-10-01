use async_trait::async_trait;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use omnibus_core::*;
use super::CoinbaseExchange;

type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
type WsWrite = futures_util::stream::SplitSink<WsStream, Message>;

const FALLBACK_WS_URL: &str = "wss://ws-feed.exchange.coinbase.com";

pub struct CoinbaseWebSocket {
    exchange: Arc<CoinbaseExchange>,
    connected: bool,
    write: Option<WsWrite>,
    ticker_callbacks: CallbackStore<Ticker>,
    orderbook_callbacks: CallbackStore<OrderBook>,
    trade_callbacks: CallbackStore<Trade>,
}

impl CoinbaseWebSocket {
    pub fn new(exchange: Arc<CoinbaseExchange>) -> Self {
        Self {
            exchange,
            connected: false,
            write: None,
            ticker_callbacks: CallbackStore::new(),
            orderbook_callbacks: CallbackStore::new(),
            trade_callbacks: CallbackStore::new(),
        }
    }

    /// Convert canonical symbol to Coinbase product_id: "BTC/USD" → "BTC-USD", "LCX/USD" → "LCX-USDC"
    fn to_coinbase_pair(symbol: &str) -> String {
        CoinbaseExchange::normalize_symbol(symbol)
    }

    /// Reverse: "BTC-USD" → "BTC/USD", "LCX-USDC" → "LCX/USD"
    fn from_coinbase_pair(product_id: &str) -> String {
        let s = product_id.replace('-', "/");
        match s.as_str() {
            "LCX/USDC" => "LCX/USD".to_string(),
            other => other.to_string(),
        }
    }
}

fn parse_ticker(exchange_name: &str, data: &serde_json::Value) -> Option<Ticker> {
    let start = std::time::Instant::now();
    let product_id = data["product_id"].as_str()?;
    let last: f64 = data["price"].as_str()?.parse().ok()?;
    Some(Ticker {
        exchange: exchange_name.to_string(),
        symbol: CoinbaseWebSocket::from_coinbase_pair(product_id),
        last,
        bid: data["best_bid"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
        ask: data["best_ask"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
        high_24h: data["high_24h"].as_str().and_then(|s| s.parse().ok()),
        low_24h:  data["low_24h"].as_str().and_then(|s| s.parse().ok()),
        volume_24h: data["volume_24h"].as_str().and_then(|s| s.parse().ok()),
        change_24h_pct: None,
        timestamp: chrono::Utc::now().timestamp_millis(),
        latency_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

/// Parses a level2 snapshot: `{"type":"snapshot","product_id":"BTC-USD","bids":[["price","size"],...],"asks":[...]}`
fn parse_orderbook_snapshot(exchange_name: &str, data: &serde_json::Value) -> Option<OrderBook> {
    let start = std::time::Instant::now();
    let product_id = data["product_id"].as_str()?;
    let mut ob = OrderBook::new(exchange_name.to_string(), CoinbaseWebSocket::from_coinbase_pair(product_id));

    for (key, is_bid) in [("bids", true), ("asks", false)] {
        if let Some(levels) = data[key].as_array() {
            for level in levels.iter().take(25) {
                if let Some(arr) = level.as_array() {
                    let price: f64 = arr.get(0).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    let size:  f64 = arr.get(1).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    let ol = OrderBookLevel { price, size };
                    if is_bid { ob.bids.push(ol); } else { ob.asks.push(ol); }
                }
            }
        }
    }

    if ob.bids.is_empty() && ob.asks.is_empty() { return None; }
    ob.calculate_metrics();
    ob.latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    Some(ob)
}

/// Parses a match (trade): `{"type":"match","product_id":"BTC-USD","side":"buy","price":"...","size":"...",...}`
fn parse_trade(exchange_name: &str, data: &serde_json::Value) -> Option<Trade> {
    let product_id = data["product_id"].as_str()?;
    Some(Trade {
        id: data["trade_id"].as_u64().map(|n| n.to_string()).unwrap_or_default(),
        exchange: exchange_name.to_string(),
        symbol: CoinbaseWebSocket::from_coinbase_pair(product_id),
        side: if data["side"].as_str() == Some("buy") { Side::Buy } else { Side::Sell },
        price:    data["price"].as_str()?.parse().ok()?,
        quantity: data["size"].as_str()?.parse().ok()?,
        timestamp: chrono::Utc::now().timestamp_millis(),
        fee: None,
        fee_coin: None,
    })
}

fn dispatch_message(
    exchange_name: &str,
    text: &str,
    ticker_cbs:    &CallbackStore<Ticker>,
    orderbook_cbs: &CallbackStore<OrderBook>,
    trade_cbs:     &CallbackStore<Trade>,
) {
    let data: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return,
    };

    let msg_type = match data["type"].as_str() {
        Some(t) => t,
        None => return,
    };

    match msg_type {
        "ticker" => {
            if let Some(ticker) = parse_ticker(exchange_name, &data) {
                ticker_cbs.invoke(&ticker.symbol.clone(), ticker);
            }
        }
        "snapshot" => {
            if let Some(ob) = parse_orderbook_snapshot(exchange_name, &data) {
                orderbook_cbs.invoke(&ob.symbol.clone(), ob);
            }
        }
        "match" | "last_match" => {
            if let Some(trade) = parse_trade(exchange_name, &data) {
                trade_cbs.invoke(&trade.symbol.clone(), trade);
            }
        }
        "subscriptions" | "heartbeat" | "l2update" => {}
        other => tracing::debug!("Coinbase WS unhandled type: {}", other),
    }
}

#[async_trait]
impl WebSocketAPI for CoinbaseWebSocket {
    async fn connect(&mut self) -> Result<(), OmniBusError> {
        let ws_url = self.exchange.ws_url()
            .unwrap_or(FALLBACK_WS_URL)
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
            tracing::info!("Coinbase WebSocket receive loop started");
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        dispatch_message(&exchange_name, &text, &ticker_cbs, &orderbook_cbs, &trade_cbs);
                    }
                    Ok(Message::Ping(_)) => {
                        tracing::debug!("Coinbase WS ping");
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("Coinbase WebSocket closed by server");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Coinbase WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            tracing::info!("Coinbase WebSocket receive loop ended");
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
        let product_id = Self::to_coinbase_pair(symbol);
        let msg = serde_json::json!({"type": "subscribe", "product_ids": [product_id], "channels": ["ticker"]});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Subscribe failed: {}", e)))?;
        }
        self.ticker_callbacks.push(symbol, callback);
        Ok(())
    }

    async fn subscribe_orderbook(&mut self, symbol: &str, callback: OrderBookCallback) -> Result<(), OmniBusError> {
        let product_id = Self::to_coinbase_pair(symbol);
        let msg = serde_json::json!({"type": "subscribe", "product_ids": [product_id], "channels": ["level2"]});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Subscribe failed: {}", e)))?;
        }
        self.orderbook_callbacks.push(symbol, callback);
        Ok(())
    }

    async fn subscribe_trades(&mut self, symbol: &str, callback: TradeCallback) -> Result<(), OmniBusError> {
        let product_id = Self::to_coinbase_pair(symbol);
        let msg = serde_json::json!({"type": "subscribe", "product_ids": [product_id], "channels": ["matches"]});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Subscribe failed: {}", e)))?;
        }
        self.trade_callbacks.push(symbol, callback);
        Ok(())
    }

    async fn unsubscribe_ticker(&mut self, symbol: &str) -> Result<(), OmniBusError> {
        let product_id = Self::to_coinbase_pair(symbol);
        let msg = serde_json::json!({"type": "unsubscribe", "product_ids": [product_id], "channels": ["ticker"]});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Unsubscribe failed: {}", e)))?;
        }
        self.ticker_callbacks.remove(symbol);
        Ok(())
    }

    async fn unsubscribe_orderbook(&mut self, symbol: &str) -> Result<(), OmniBusError> {
        let product_id = Self::to_coinbase_pair(symbol);
        let msg = serde_json::json!({"type": "unsubscribe", "product_ids": [product_id], "channels": ["level2"]});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Unsubscribe failed: {}", e)))?;
        }
        self.orderbook_callbacks.remove(symbol);
        Ok(())
    }

    async fn unsubscribe_trades(&mut self, symbol: &str) -> Result<(), OmniBusError> {
        let product_id = Self::to_coinbase_pair(symbol);
        let msg = serde_json::json!({"type": "unsubscribe", "product_ids": [product_id], "channels": ["matches"]});
        if let Some(write) = &mut self.write {
            write.send(Message::Text(msg.to_string())).await
                .map_err(|e| OmniBusError::WebSocket(format!("Unsubscribe failed: {}", e)))?;
        }
        self.trade_callbacks.remove(symbol);
        Ok(())
    }
}
