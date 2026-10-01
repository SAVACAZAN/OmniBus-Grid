// crates/omnibus-exchange/src/coinbase/private.rs
use async_trait::async_trait;
use omnibus_core::*;
use super::CoinbaseExchange;

#[async_trait]
impl PrivateAPI for CoinbaseExchange {
    async fn balances(&self) -> Result<Vec<Balance>, OmniBusError> {
        // Fetch all accounts with limit=250 (max) to get everything in one call
        let path = "/api/v3/brokerage/accounts?limit=250";

        // Parse as raw JSON to handle pagination + optional fields
        let response: serde_json::Value = self.request("GET", path, None, true).await?;

        let accounts = response.get("accounts").and_then(|a| a.as_array())
            .ok_or_else(|| OmniBusError::Auth("Coinbase: missing accounts array".to_string()))?;

        let mut balances = Vec::new();
        for acc in accounts {
            let currency = acc.get("currency").and_then(|c| c.as_str()).unwrap_or("").to_string();
            let available = acc.get("available_balance").and_then(|b| b.get("value")).and_then(|v| v.as_str()?.parse::<f64>().ok()).unwrap_or(0.0);
            let hold = acc.get("hold").and_then(|b| b.get("value")).and_then(|v| v.as_str()?.parse::<f64>().ok()).unwrap_or(0.0);
            let total = available + hold;
            if !currency.is_empty() {
                balances.push(Balance {
                    exchange: self.name().to_string(),
                    coin: currency, available, locked: hold, total,
                });
            }
        }

        Ok(balances)
    }
    
    async fn balance(&self, coin: &str) -> Result<Balance, OmniBusError> {
        let balances = self.balances().await?;
        balances.into_iter()
            .find(|b| b.coin == coin)
            .ok_or_else(|| OmniBusError::ExchangeError {
                code: "NOT_FOUND".to_string(),
                message: format!("Coin {} not found", coin),
            })
    }
    
    async fn place_order(&self, symbol: &str, side: Side, order_type: OrderType, qty: f64, price: Option<f64>) -> Result<Order, OmniBusError> {
        let path = "/api/v3/brokerage/orders";
        let product_id = CoinbaseExchange::normalize_symbol(symbol);

        // Round price/qty to product tick (Coinbase rejects mis-aligned increments).
        // Tick is fetched once per pair and cached.
        let tick = self.get_tick(&product_id).await.unwrap_or_default();
        let qty_aligned = if tick.base_increment > 0.0 {
            (qty / tick.base_increment).floor() * tick.base_increment
        } else { qty };
        let price_aligned = price.map(|p| {
            if tick.quote_increment > 0.0 {
                (p / tick.quote_increment).round() * tick.quote_increment
            } else { p }
        });
        let qty_str = if tick.qty_decimals > 0 { format!("{:.*}", tick.qty_decimals, qty_aligned) } else { crate::fmt::fmt_qty(qty_aligned) };
        let price_str = price_aligned.map(|p| {
            if tick.price_decimals > 0 { format!("{:.*}", tick.price_decimals, p) } else { crate::fmt::fmt_price(p) }
        });

        let mut body = serde_json::json!({
            "client_order_id": format!("order_{}", chrono::Utc::now().timestamp_millis()),
            "product_id": product_id,
            "side": if matches!(side, Side::Buy) { "BUY" } else { "SELL" },
            "order_configuration": {},
        });

        if matches!(order_type, OrderType::Market) {
            body["order_configuration"]["market_market_ioc"] = serde_json::json!({
                "quote_size": qty_str,
            });
        } else {
            body["order_configuration"]["limit_limit_gtc"] = serde_json::json!({
                "base_size": qty_str,
                "limit_price": price_str.unwrap_or_else(|| "0".to_string()),
                "post_only": false,
            });
        }

        #[derive(serde::Deserialize)]
        struct SuccessResponse { order_id: Option<String> }
        #[derive(serde::Deserialize)]
        struct ErrorResponse {
            error: Option<String>,
            message: Option<String>,
            error_details: Option<String>,
            preview_failure_reason: Option<String>,
            new_order_failure_reason: Option<String>,
        }
        #[derive(serde::Deserialize)]
        struct CoinbaseOrderResponse {
            #[serde(default)]
            order_id: Option<String>,
            #[serde(default)]
            success: bool,
            success_response: Option<SuccessResponse>,
            error_response: Option<ErrorResponse>,
        }

        let response: CoinbaseOrderResponse = self.request("POST", path, Some(body), true).await?;
        let order_id = response.success_response
            .and_then(|s| s.order_id)
            .or(response.order_id)
            .unwrap_or_default();

        // Return Err on rejection — engines see real exchange error instead of fake "OK with empty id"
        if !response.success || order_id.is_empty() {
            let err = response.error_response;
            let code = err.as_ref().and_then(|e| e.error.clone()).unwrap_or_else(|| "REJECTED".to_string());
            let msg = err.as_ref().and_then(|e| {
                e.message.clone()
                    .or_else(|| e.error_details.clone())
                    .or_else(|| e.preview_failure_reason.clone())
                    .or_else(|| e.new_order_failure_reason.clone())
            }).unwrap_or_else(|| "Coinbase rejected order (no error_response body)".to_string());
            return Err(OmniBusError::ExchangeError { code, message: msg });
        }

        Ok(Order {
            id: order_id,
            exchange: self.name().to_string(),
            symbol: symbol.to_string(),
            side,
            order_type,
            quantity: qty,
            price,
            filled_qty: 0.0,
            status: OrderStatus::Open,
            created_at: chrono::Utc::now().timestamp_millis(),
        })
    }
    
    async fn cancel_order(&self, order_id: &str) -> Result<bool, OmniBusError> {
        let path = "/api/v3/brokerage/orders/batch_cancel";
        let body = serde_json::json!({
            "order_ids": [order_id],
        });
        
        #[derive(serde::Deserialize)]
        struct CoinbaseCancelResponse {
            results: Vec<CoinbaseCancelResult>,
        }
        
        #[derive(serde::Deserialize)]
        struct CoinbaseCancelResult {
            success: bool,
        }
        
        let response: CoinbaseCancelResponse = self.request("POST", path, Some(body), true).await?;
        
        Ok(response.results.first().map(|r| r.success).unwrap_or(false))
    }
    
    async fn open_orders(&self) -> Result<Vec<Order>, OmniBusError> {
        // Coinbase Advanced Trade: GET /api/v3/brokerage/orders/historical/batch?order_status=OPEN
        let path = "/api/v3/brokerage/orders/historical/batch?order_status=OPEN&limit=100";

        let response: serde_json::Value = self.request("GET", path, None, true).await?;
        let orders_arr = response.get("orders").and_then(|o| o.as_array()).cloned().unwrap_or_default();
        Ok(parse_cb_orders(&self.name().to_string(), &orders_arr))
    }

    async fn order_history(&self, limit: usize) -> Result<Vec<Order>, OmniBusError> {
        // Exclude OPEN orders — only FILLED, CANCELLED, etc.
        let path = format!("/api/v3/brokerage/orders/historical/batch?limit={}&order_status=FILLED&order_status=CANCELLED", limit);
        let response: serde_json::Value = self.request("GET", &path, None, true).await?;
        let orders_arr = response.get("orders").and_then(|o| o.as_array()).cloned().unwrap_or_default();
        Ok(parse_cb_orders(&self.name().to_string(), &orders_arr))
    }

    async fn trade_history(&self, _limit: usize) -> Result<Vec<Trade>, OmniBusError> {
        Ok(Vec::new())
    }
}

fn parse_cb_orders(exchange: &str, orders: &[serde_json::Value]) -> Vec<Order> {
    orders.iter().filter_map(|o| {
        let id = o.get("order_id")?.as_str()?.to_string();
        let symbol = o.get("product_id")?.as_str()?.to_string();
        let side_str = o.get("side").and_then(|s| s.as_str()).unwrap_or("BUY");
        let status_str = o.get("status").and_then(|s| s.as_str()).unwrap_or("OPEN");
        let filled: f64 = o.get("filled_size").and_then(|v| v.as_str()?.parse().ok()).unwrap_or(0.0);
        let created = o.get("created_time").and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp_millis()).unwrap_or(0);
        let cfg = o.get("order_configuration");
        let limit_cfg = cfg.and_then(|c| c.get("limit_limit_gtc").or(c.get("limit_limit_gtd")));
        let price: Option<f64> = limit_cfg.and_then(|l| l.get("limit_price")?.as_str()?.parse().ok());
        let qty: f64 = limit_cfg.and_then(|l| l.get("base_size")?.as_str()?.parse().ok())
            .or(cfg.and_then(|c| c.get("market_market_ioc")).and_then(|m| m.get("quote_size")?.as_str()?.parse().ok()))
            .unwrap_or(0.0);
        Some(Order {
            id, exchange: exchange.to_string(), symbol,
            side: if side_str == "BUY" { Side::Buy } else { Side::Sell },
            order_type: if limit_cfg.is_some() { OrderType::Limit } else { OrderType::Market },
            quantity: qty, price, filled_qty: filled,
            status: match status_str { "OPEN" | "PENDING" => OrderStatus::Open, "FILLED" => OrderStatus::Filled, "CANCELLED" => OrderStatus::Cancelled, _ => OrderStatus::Filled },
            created_at: created,
        })
    }).collect()
}