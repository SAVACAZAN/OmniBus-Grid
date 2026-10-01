use async_trait::async_trait;
use omnibus_core::*;
use super::KrakenExchange;

#[async_trait]
impl PrivateAPI for KrakenExchange {
    async fn balances(&self) -> Result<Vec<Balance>, OmniBusError> {
        let endpoint = "/0/private/Balance";

        // mod.rs request<T>() already unwraps KrakenResponse wrapper — returns T directly
        let result: std::collections::HashMap<String, String> = self.request(endpoint, None, true).await?;

        Ok(result.into_iter().map(|(coin, amount)| {
            let total = amount.parse::<f64>().unwrap_or(0.0);
            Balance { exchange: self.name().to_string(), coin, available: total, locked: 0.0, total }
        }).collect())
    }

    async fn balance(&self, coin: &str) -> Result<Balance, OmniBusError> {
        let balances = self.balances().await?;
        balances.into_iter()
            .find(|b| b.coin.to_uppercase().contains(&coin.to_uppercase()))
            .ok_or_else(|| OmniBusError::Exchange(format!("Coin {} not found", coin)))
    }

    async fn place_order(&self, symbol: &str, side: Side, order_type: OrderType, qty: f64, price: Option<f64>) -> Result<Order, OmniBusError> {
        let endpoint = "/0/private/AddOrder";
        let side_str = match side { Side::Buy => "buy", Side::Sell => "sell" };
        let type_str = match order_type { OrderType::Limit => "limit", OrderType::Market => "market" };

        let mut params = std::collections::HashMap::new();
        params.insert("pair".to_string(), serde_json::Value::String(symbol.to_string()));
        params.insert("type".to_string(), serde_json::Value::String(side_str.to_string()));
        params.insert("ordertype".to_string(), serde_json::Value::String(type_str.to_string()));
        params.insert("volume".to_string(), serde_json::Value::String(crate::fmt::fmt_qty(qty)));
        if let Some(p) = price {
            params.insert("price".to_string(), serde_json::Value::String(crate::fmt::fmt_price(p)));
        }

        #[derive(serde::Deserialize)]
        struct RespDescr { order: Option<String> }
        #[derive(serde::Deserialize)]
        struct RespResult { txid: Option<Vec<String>>, descr: Option<RespDescr> }

        // request() already unwraps the KrakenResponse wrapper — T is the inner result object
        let response: RespResult = self.request(endpoint, Some(serde_json::json!(params)), true).await?;
        let order_id = response.txid.and_then(|ids| ids.into_iter().next()).unwrap_or_default();

        // Empty txid means Kraken accepted the request but did not place (validate-mode or silent rejection)
        if order_id.is_empty() {
            let descr = response.descr.and_then(|d| d.order).unwrap_or_default();
            return Err(OmniBusError::ExchangeError {
                code: "KRAKEN_NO_TXID".to_string(),
                message: format!("Kraken returned no txid (validate-mode or rejected). descr={}", descr),
            });
        }

        Ok(Order { id: order_id, exchange: self.name().to_string(), symbol: symbol.to_string(),
            side, order_type, quantity: qty, price, filled_qty: 0.0, status: OrderStatus::Open,
            created_at: chrono::Utc::now().timestamp() })
    }

    async fn cancel_order(&self, order_id: &str) -> Result<bool, OmniBusError> {
        let endpoint = "/0/private/CancelOrder";
        let mut params = std::collections::HashMap::new();
        params.insert("txid".to_string(), serde_json::Value::String(order_id.to_string()));
        let _: serde_json::Value = self.request(endpoint, Some(serde_json::json!(params)), true).await?;
        Ok(true)
    }

    async fn open_orders(&self) -> Result<Vec<Order>, OmniBusError> {
        let response: serde_json::Value = self.request("/0/private/OpenOrders", None, true).await?;
        Ok(parse_kraken_orders(self.name(), &response, "open"))
    }

    async fn order_history(&self, _limit: usize) -> Result<Vec<Order>, OmniBusError> {
        let response: serde_json::Value = self.request("/0/private/ClosedOrders", None, true).await?;
        Ok(parse_kraken_orders(self.name(), &response, "closed"))
    }

    async fn trade_history(&self, _limit: usize) -> Result<Vec<Trade>, OmniBusError> {
        let response: serde_json::Value = self.request("/0/private/TradesHistory", None, true).await?;
        let trades_map = response.get("trades").and_then(|t| t.as_object());
        let empty = serde_json::Map::new();
        let trades = trades_map.unwrap_or(&empty);
        Ok(trades.iter().map(|(id, t)| {
            Trade {
                id: id.clone(), exchange: self.name().to_string(),
                symbol: t.get("pair").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                side: if t.get("type").and_then(|v| v.as_str()).unwrap_or("buy") == "buy" { Side::Buy } else { Side::Sell },
                price: t.get("price").and_then(|v| v.as_str()?.parse().ok()).unwrap_or(0.0),
                quantity: t.get("vol").and_then(|v| v.as_str()?.parse().ok()).unwrap_or(0.0),
                timestamp: t.get("time").and_then(|v| v.as_f64()).map(|f| (f * 1000.0) as i64).unwrap_or(0),
                fee: t.get("fee").and_then(|v| v.as_str()?.parse().ok()),
                fee_coin: Some("USD".to_string()),
            }
        }).collect())
    }
}

/// Parse Kraken orders: result.open/closed = {order_id: {descr:{pair,type,price}, vol, status}}
fn parse_kraken_orders(exchange: &str, response: &serde_json::Value, key: &str) -> Vec<Order> {
    let orders_map = response.get(key).and_then(|o| o.as_object());
    let empty = serde_json::Map::new();
    let orders = orders_map.unwrap_or(&empty);
    orders.iter().map(|(id, o)| {
        let descr = o.get("descr").unwrap_or(&serde_json::Value::Null);
        let pair = descr.get("pair").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let side_str = descr.get("type").and_then(|v| v.as_str()).unwrap_or("buy");
        let price_str = descr.get("price").and_then(|v| v.as_str()).unwrap_or("0");
        let vol: f64 = o.get("vol").and_then(|v| v.as_str()?.parse().ok()).unwrap_or(0.0);
        let vol_exec: f64 = o.get("vol_exec").and_then(|v| v.as_str()?.parse().ok()).unwrap_or(0.0);
        let status_str = o.get("status").and_then(|v| v.as_str()).unwrap_or("open");
        let opentm: i64 = o.get("opentm").and_then(|v| v.as_f64()).map(|f| f as i64).unwrap_or(0);
        Order {
            id: id.clone(), exchange: exchange.to_string(), symbol: pair,
            side: if side_str == "buy" { Side::Buy } else { Side::Sell },
            order_type: OrderType::Limit,
            quantity: vol, price: price_str.parse().ok(), filled_qty: vol_exec,
            status: match status_str { "open" | "pending" => OrderStatus::Open, "closed" => OrderStatus::Filled, "canceled" => OrderStatus::Cancelled, _ => OrderStatus::Filled },
            created_at: opentm,
        }
    }).collect()
}
