use async_trait::async_trait;
use omnibus_core::*;
use super::LCXExchange;

/// Parse LCX orders from JSON response (PascalCase fields: Id, Pair, Price, Amount, Side, Status, Filled)
fn parse_lcx_orders_json(exchange: &str, response: &serde_json::Value, default_status: OrderStatus) -> Vec<Order> {
    let data = response.get("data").and_then(|d| d.as_array());
    let empty = Vec::new();
    let items = data.unwrap_or(&empty);
    items.iter().filter_map(|item| {
        let id = item.get("Id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if id.is_empty() { return None; }
        let pair = item.get("Pair").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let side_str = item.get("Side").and_then(|v| v.as_str()).unwrap_or("BUY");
        let price = item.get("Price").and_then(|v| v.as_f64());
        let amount = item.get("Amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let filled = item.get("Filled").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let status_str = item.get("Status").and_then(|v| v.as_str()).unwrap_or("");
        // CreatedAt can be 0 for history — fallback to UpdatedAt
        let created = item.get("CreatedAt").and_then(|v| v.as_i64()).unwrap_or(0);
        let updated = item.get("UpdatedAt").and_then(|v| v.as_i64()).unwrap_or(0);
        let timestamp = if created > 0 { created } else { updated };
        let status = match status_str.to_uppercase().as_str() {
            "OPEN" | "PARTIAL" => OrderStatus::Open,
            "CLOSED" | "FILLED" => OrderStatus::Filled,
            "CANCELLED" | "CANCELED" => OrderStatus::Cancelled,
            _ => default_status.clone(),
        };
        Some(Order { id, exchange: exchange.to_string(), symbol: pair, side: if side_str == "BUY" { Side::Buy } else { Side::Sell }, order_type: OrderType::Limit, quantity: amount, price, filled_qty: filled, status, created_at: timestamp * 1000 })
    }).collect()
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct LCXOrderData {
    #[serde(alias = "Id", alias = "id")]
    order_id: String,
    #[serde(alias = "Pair", alias = "pair")]
    pair: String,
    #[serde(alias = "Side", alias = "type", alias = "side")]
    order_side: String,
    #[serde(alias = "OrderType", alias = "order_type", alias = "orderType")]
    order_type: String,
    #[serde(alias = "Amount", alias = "amount")]
    amount: serde_json::Value,
    #[serde(alias = "Price", alias = "price")]
    price: Option<serde_json::Value>,
    #[serde(alias = "Filled", alias = "filled", default)]
    filled: serde_json::Value,
    #[serde(alias = "Status", alias = "status")]
    status: String,
    #[serde(alias = "CreatedAt", alias = "created_at", default)]
    created_at: i64,
}

#[allow(dead_code)]
fn val_to_f64(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or_else(|| v.as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0))
}

#[allow(dead_code)]
fn parse_lcx_order(name: &str, order: LCXOrderData) -> Order {
    Order {
        id: order.order_id,
        exchange: name.to_string(),
        symbol: order.pair,
        side: if order.order_side == "BUY" || order.order_side == "buy" { Side::Buy } else { Side::Sell },
        order_type: if order.order_type == "LIMIT" || order.order_type == "limit" { OrderType::Limit } else { OrderType::Market },
        quantity: val_to_f64(&order.amount),
        price: order.price.as_ref().map(val_to_f64).filter(|p| *p > 0.0),
        filled_qty: val_to_f64(&order.filled),
        status: match order.status.to_uppercase().as_str() {
            "OPEN" | "PARTIAL" => OrderStatus::Open,
            "CLOSED" | "FILLED" => OrderStatus::Filled,
            "CANCELLED" | "CANCELED" => OrderStatus::Cancelled,
            _ => OrderStatus::Open,
        },
        created_at: order.created_at,
    }
}

#[async_trait]
impl PrivateAPI for LCXExchange {
    async fn balances(&self) -> Result<Vec<Balance>, OmniBusError> {
        let endpoint = "/api/balances";

        // Parse as raw JSON — LCX returns PascalCase or camelCase
        let response: serde_json::Value = self.request("GET", endpoint, None, true).await?;

        let data = response.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| OmniBusError::Auth("balances: missing data array".to_string()))?;

        // LCX v1.1.0 format: { coin, balance: { freeBalance, occupiedBalance, totalBalance } }
        let balances: Vec<Balance> = data.iter().filter_map(|item| {
            let coin = item.get("coin")?.as_str()?.to_string();
            let bal = item.get("balance")?;
            let free = bal.get("freeBalance").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let locked = bal.get("occupiedBalance").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let total = bal.get("totalBalance").and_then(|v| v.as_f64()).unwrap_or(free + locked);
            Some(Balance {
                exchange: self.name().to_string(),
                coin, available: free, locked, total,
            })
        }).collect();

        Ok(balances)
    }
    
    async fn balance(&self, coin: &str) -> Result<Balance, OmniBusError> {
        let endpoint = format!("/api/balance?currency={}", coin);
        
        #[derive(serde::Deserialize)]
        struct LCXBalanceResponse {
            #[allow(dead_code)]
            success: bool,
            data: LCXBalanceItem,
        }
        
        #[derive(serde::Deserialize)]
        struct LCXBalanceItem {
            currency: String,
            available: String,
            locked: String,
        }
        
        let response: LCXBalanceResponse = self.request("GET", &endpoint, None, true).await?;
        
        Ok(Balance {
            exchange: self.name().to_string(),
            coin: response.data.currency,
            available: response.data.available.parse().unwrap_or(0.0),
            locked: response.data.locked.parse().unwrap_or(0.0),
            total: response.data.available.parse::<f64>().unwrap_or(0.0) + response.data.locked.parse::<f64>().unwrap_or(0.0),
        })
    }
    
    async fn place_order(&self, symbol: &str, side: Side, order_type: OrderType, qty: f64, price: Option<f64>) -> Result<Order, OmniBusError> {
        let endpoint = "/api/create";

        let side_str = match side { Side::Buy => "BUY", Side::Sell => "SELL" };
        let type_str = match order_type { OrderType::Limit => "LIMIT", OrderType::Market => "MARKET" };

        // Format via fmt_* to strip IEEE 754 artifacts (e.g. 0.04196 → 0.041960000000000003)
        let amount_clean: f64 = crate::fmt::fmt_qty(qty).parse().unwrap_or(qty);
        let mut body = serde_json::json!({
            "Pair": symbol,
            "Side": side_str,
            "OrderType": type_str,
            "Amount": amount_clean,
        });
        if let Some(p) = price {
            let price_clean: f64 = crate::fmt::fmt_price(p).parse().unwrap_or(p);
            body["Price"] = serde_json::json!(price_clean);
        }

        let response: serde_json::Value = self.request("POST", endpoint, Some(body), true).await?;
        tracing::info!("LCX create order response: {:?}", response);

        // LCX returns { status: "success"|"error", message, data: {...} }
        let resp_status = response.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if resp_status.eq_ignore_ascii_case("error") || resp_status == "fail" {
            let msg = response.get("message").and_then(|v| v.as_str()).unwrap_or("LCX rejected order");
            return Err(OmniBusError::ExchangeError { code: "LCX_REJECTED".to_string(), message: msg.to_string() });
        }

        let data = response.get("data").ok_or_else(|| OmniBusError::Exchange("LCX: missing data in response".to_string()))?;
        let id = data.get("Id").or(data.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        if id.is_empty() {
            return Err(OmniBusError::ExchangeError {
                code: "LCX_NO_ID".to_string(),
                message: format!("LCX returned no order Id. raw={}", response),
            });
        }
        let pair = data.get("Pair").or(data.get("pair")).and_then(|v| v.as_str()).unwrap_or(symbol).to_string();
        let resp_side = data.get("Side").or(data.get("side")).and_then(|v| v.as_str()).unwrap_or("");
        let resp_price = data.get("Price").or(data.get("price")).and_then(|v| v.as_f64()).or(price);
        let resp_amount = data.get("Amount").or(data.get("amount")).and_then(|v| v.as_f64()).unwrap_or(qty);
        let _resp_status = data.get("Status").or(data.get("status")).and_then(|v| v.as_str()).unwrap_or("OPEN");

        Ok(Order {
            id,
            exchange: self.name().to_string(),
            symbol: pair,
            side: if resp_side == "BUY" || resp_side == "buy" { Side::Buy } else { Side::Sell },
            order_type: order_type.clone(),
            quantity: resp_amount,
            price: resp_price,
            filled_qty: 0.0,
            status: OrderStatus::Open,
            created_at: chrono::Utc::now().timestamp_millis(),
        })
    }
    
    async fn cancel_order(&self, order_id: &str) -> Result<bool, OmniBusError> {
        let endpoint = format!("/api/cancel?orderId={}", order_id);
        let response: serde_json::Value = self.request("DELETE", &endpoint, None, true).await?;
        tracing::info!("LCX cancel response: {:?}", response);
        let status = response.get("status").and_then(|v| v.as_str()).unwrap_or("");
        Ok(status == "success")
    }
    
    async fn open_orders(&self) -> Result<Vec<Order>, OmniBusError> {
        let response: serde_json::Value = self.request("GET", "/api/open?offset=1&limit=50", None, true).await?;
        Ok(parse_lcx_orders_json(self.name(), &response, OrderStatus::Open))
    }

    async fn order_history(&self, limit: usize) -> Result<Vec<Order>, OmniBusError> {
        let endpoint = format!("/api/orderHistory?offset=1&limit={}", limit);
        let response: serde_json::Value = self.request("GET", &endpoint, None, true).await?;
        Ok(parse_lcx_orders_json(self.name(), &response, OrderStatus::Filled))
    }
    
    async fn trade_history(&self, limit: usize) -> Result<Vec<Trade>, OmniBusError> {
        let _endpoint = format!("/api/uHistory?limit={}", limit);
        
        #[allow(dead_code)]
        #[derive(serde::Deserialize)]
        struct LCXTradeHistoryResponse {
            success: bool,
            data: Vec<LCXHistoryTrade>,
        }

        #[allow(dead_code)]
        #[derive(serde::Deserialize)]
        struct LCXHistoryTrade {
            id: String,
            pair: String,
            price: String,
            volume: String,
            trade_type: String,
            created_at: i64,
        }
        
        // LCX trade history: PascalCase fields, needs offset=1
        let endpoint = format!("/api/uHistory?offset=1&limit={}", limit);
        let response: serde_json::Value = self.request("GET", &endpoint, None, true).await?;
        let data = response.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();

        Ok(data.iter().filter_map(|t| {
            let id = t.get("Id")?.as_str()?.to_string();
            let pair = t.get("Pair")?.as_str()?.to_string();
            let side_str = t.get("Side").and_then(|v| v.as_str()).unwrap_or("BUY");
            let price = t.get("Price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let amount = t.get("Amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let created = t.get("CreatedAt").and_then(|v| v.as_i64()).unwrap_or(0);
            let fee = t.get("Fee").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let fee_coin = t.get("FeeCoin").and_then(|v| v.as_str()).unwrap_or("").to_string();
            Some(Trade {
                id, exchange: self.name().to_string(), symbol: pair,
                side: if side_str == "BUY" { Side::Buy } else { Side::Sell },
                price, quantity: amount, timestamp: created * 1000,
                fee: Some(fee), fee_coin: Some(fee_coin),
            })
        }).collect())
    }
}