use async_trait::async_trait;
use omnibus_core::*;
use super::LCXExchange;

#[async_trait]
impl PublicAPI for LCXExchange {
    async fn ticker(&self, symbol: &str) -> Result<Ticker, OmniBusError> {
        let start = std::time::Instant::now();
        let url = format!("/api/ticker?pair={}", symbol);

        #[derive(serde::Deserialize)]
        struct LCXTickerResponse {
            data: LCXTickerData,
        }

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LCXTickerData {
            best_bid: Option<f64>,
            best_ask: Option<f64>,
            change: Option<f64>,
            #[serde(default)]
            high24h: Option<f64>,
            #[serde(default)]
            low24h: Option<f64>,
            #[serde(default)]
            volume: Option<f64>,
        }

        let response: LCXTickerResponse = self.request("GET", &url, None, false).await?;

        let data = response.data;
        let bid = data.best_bid.unwrap_or(0.0);
        let ask = data.best_ask.unwrap_or(0.0);
        let last = (bid + ask) / 2.0; // LCX doesn't return "last", use mid
        
        Ok(Ticker {
            exchange: self.name().to_string(),
            symbol: symbol.to_string(),
            last,
            bid,
            ask,
            high_24h: data.high24h,
            low_24h: data.low24h,
            volume_24h: data.volume,
            change_24h_pct: data.change,
            timestamp: chrono::Utc::now().timestamp_millis(),
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }
    
    async fn tickers(&self, symbols: &[String]) -> Result<Vec<Ticker>, OmniBusError> {
        let start = std::time::Instant::now();
        let endpoint = "/api/tickers";
        
        #[derive(serde::Deserialize)]
        struct LCXTickersResponse {
            #[allow(dead_code)]
            success: bool,
            data: Vec<LCXTickerItem>,
        }
        
        #[derive(serde::Deserialize)]
        struct LCXTickerItem {
            pair: String,
            buy: String,
            sell: String,
            last: String,
            high: Option<String>,
            low: Option<String>,
            vol: Option<String>,
        }
        
        let response: LCXTickersResponse = self.request("GET", endpoint, None, false).await?;
        
        let mut tickers = Vec::new();
        for item in response.data {
            if symbols.is_empty() || symbols.contains(&item.pair) {
                tickers.push(Ticker {
                    exchange: self.name().to_string(),
                    symbol: item.pair,
                    last: item.last.parse().unwrap_or(0.0),
                    bid: item.buy.parse().unwrap_or(0.0),
                    ask: item.sell.parse().unwrap_or(0.0),
                    high_24h: item.high.and_then(|h| h.parse().ok()),
                    low_24h: item.low.and_then(|l| l.parse().ok()),
                    volume_24h: item.vol.and_then(|v| v.parse().ok()),
                    change_24h_pct: None,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    latency_ms: start.elapsed().as_secs_f64() * 1000.0,
                });
            }
        }
        
        Ok(tickers)
    }
    
    async fn orderbook(&self, symbol: &str, depth: usize) -> Result<OrderBook, OmniBusError> {
        let start = std::time::Instant::now();
        let endpoint = format!("/api/book?pair={}", symbol);

        #[derive(serde::Deserialize)]
        struct LCXOrderBookResponse {
            data: LCXOrderBookData,
        }

        #[derive(serde::Deserialize)]
        struct LCXOrderBookData {
            buy: Vec<Vec<f64>>,
            sell: Vec<Vec<f64>>,
        }

        let response: LCXOrderBookResponse = self.request("GET", &endpoint, None, false).await?;

        let bids: Vec<OrderBookLevel> = response.data.buy.iter().take(depth)
            .filter(|b| b.len() >= 2)
            .map(|b| OrderBookLevel { price: b[0], size: b[1] })
            .collect();
        let asks: Vec<OrderBookLevel> = response.data.sell.iter().take(depth)
            .filter(|a| a.len() >= 2)
            .map(|a| OrderBookLevel { price: a[0], size: a[1] })
            .collect();

        let best_bid = bids.first().map(|b| b.price).unwrap_or(0.0);
        let best_ask = asks.first().map(|a| a.price).unwrap_or(0.0);
        let spread = best_ask - best_bid;
        let spread_pct = if best_bid > 0.0 { (spread / best_bid) * 100.0 } else { 0.0 };
        let mid_price = (best_bid + best_ask) / 2.0;
        let latency = start.elapsed().as_secs_f64() * 1000.0;

        Ok(OrderBook {
            exchange: self.name().to_string(),
            symbol: symbol.to_string(),
            bids,
            asks,
            spread,
            spread_pct,
            mid_price,
            timestamp: chrono::Utc::now().timestamp_millis(),
            latency_ms: latency,
        })
    }
    
    async fn trades(&self, symbol: &str, limit: usize) -> Result<Vec<Trade>, OmniBusError> {
        // LCX requires `offset` param. Returns trades as ARRAYS: [price, volume, "BUY", timestamp_secs]
        let endpoint = format!("/api/trades?pair={}&limit={}&offset=0", symbol, limit);
        let response: serde_json::Value = self.request("GET", &endpoint, None, false).await?;
        let arr = response.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();

        let parse_f64 = |v: &serde_json::Value| -> f64 {
            v.as_f64().unwrap_or_else(|| v.as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0))
        };

        Ok(arr.into_iter().enumerate().filter_map(|(idx, t)| {
            // Array format (current LCX): [price, volume, "BUY"|"SELL", timestamp_secs]
            if let Some(a) = t.as_array() {
                if a.len() < 4 { return None; }
                let price = parse_f64(&a[0]);
                let qty   = parse_f64(&a[1]);
                let side_str = a[2].as_str().unwrap_or("BUY").to_uppercase();
                let ts    = a[3].as_i64().unwrap_or(0) * 1000; // secs → ms
                return Some(Trade {
                    id: format!("{}-{}", ts, idx),
                    exchange: self.name().to_string(),
                    symbol: symbol.to_string(),
                    side: if side_str == "BUY" { Side::Buy } else { Side::Sell },
                    price, quantity: qty,
                    timestamp: ts, fee: None, fee_coin: None,
                });
            }
            // Fallback object format (in case LCX adds it back): try aliases
            let id = t.get("id").or(t.get("Id"))
                .and_then(|v| v.as_str().map(String::from).or_else(|| v.as_i64().map(|n| n.to_string())))
                .unwrap_or_default();
            let price = parse_f64(t.get("price").or(t.get("Price")).unwrap_or(&serde_json::Value::Null));
            let qty = parse_f64(t.get("volume").or(t.get("Volume")).or(t.get("amount")).unwrap_or(&serde_json::Value::Null));
            let side_str = t.get("trade_type").or(t.get("side")).or(t.get("Side"))
                .and_then(|v| v.as_str()).unwrap_or("buy").to_uppercase();
            let ts = t.get("created_at").or(t.get("timestamp"))
                .and_then(|v| v.as_i64()).unwrap_or(0);
            Some(Trade {
                id, exchange: self.name().to_string(), symbol: symbol.to_string(),
                side: if side_str == "BUY" { Side::Buy } else { Side::Sell },
                price, quantity: qty, timestamp: ts, fee: None, fee_coin: None,
            })
        }).collect())
    }
    
    async fn candles(&self, symbol: &str, interval: &str, limit: usize) -> Result<Vec<Candle>, OmniBusError> {
        // LCX kline is on separate domain: api-kline.lcx.com
        // Kline API expects pair without slash: "LCXEUR" not "LCX/EUR"
        let kline_pair = symbol.replace('/', "");
        let resolution = match interval {
            "1m" => "1", "3m" => "3", "5m" => "5", "15m" => "15", "30m" => "30",
            "1h" => "60", "2h" => "120", "4h" => "240", "1d" => "1440", "1w" => "10080", _ => "15",
        };
        let bar_secs: i64 = match interval {
            "1m" => 60, "3m" => 180, "5m" => 300, "15m" => 900, "30m" => 1800,
            "1h" => 3600, "2h" => 7200, "4h" => 14400, "1d" => 86400, "1w" => 604800, _ => 900,
        };

        // LCX kline API max ~500 bars per request — paginate backward to fill limit
        let page_size: i64 = 500;
        let mut all_candles: Vec<Candle> = Vec::new();
        let client = reqwest::Client::new();
        let mut to = chrono::Utc::now().timestamp();

        loop {
            let from = to - bar_secs * page_size;
            let url = format!(
                "https://api-kline.lcx.com/v1/market/kline?pair={}&resolution={}&from={}&to={}",
                kline_pair, resolution, from, to
            );
            let resp = client.get(&url).timeout(std::time::Duration::from_secs(15)).send().await?;
            let data: serde_json::Value = resp.json().await?;

            let batch: Vec<Candle> = data.get("data").and_then(|d| d.as_array())
                .map(|arr| arr.iter().filter_map(|c| {
                    Some(Candle {
                        timestamp: c.get("timestamp")?.as_i64()? * 1000,
                        open:   c.get("open")?.as_f64()?,
                        high:   c.get("high")?.as_f64()?,
                        low:    c.get("low")?.as_f64()?,
                        close:  c.get("close")?.as_f64()?,
                        volume: c.get("volume")?.as_f64().unwrap_or(0.0),
                    })
                }).collect())
                .unwrap_or_default();

            if batch.is_empty() { break; }
            let oldest = batch.iter().map(|c| c.timestamp / 1000).min().unwrap_or(0);
            all_candles.extend(batch);

            if limit > 0 && all_candles.len() >= limit { break; }
            // Move window backward — stop if oldest bar is very old (> 10 years)
            to = oldest - 1;
            let cutoff = chrono::Utc::now().timestamp() - 10 * 365 * 86400;
            if to < cutoff { break; }
        }

        all_candles.sort_by_key(|c| c.timestamp);
        all_candles.dedup_by_key(|c| c.timestamp);
        // limit==0 means all; otherwise take most recent `limit` bars
        if limit > 0 {
            let len = all_candles.len();
            if len > limit { all_candles = all_candles.split_off(len - limit); }
        }
        Ok(all_candles)
    }
    
    async fn pair_info(&self, symbol: &str) -> Result<omnibus_core::PairInfo, OmniBusError> {
        let endpoint = format!("/api/pair?pair={}", symbol);

        #[derive(serde::Deserialize, Default)]
        struct LCXOrderPrecision { #[serde(rename = "Price")] price: Option<i64>, #[serde(rename = "Amount")] amount: Option<i64> }
        #[derive(serde::Deserialize, Default)]
        struct LCXMinOrder { #[serde(rename = "Base")] base: Option<f64>, #[serde(rename = "Quote")] quote: Option<f64> }
        #[derive(serde::Deserialize, Default)]
        struct LCXPairDetail {
            #[serde(rename = "Orderprecision", default)] precision: LCXOrderPrecision,
            #[serde(rename = "MinOrder", default)] min_order: LCXMinOrder,
        }
        #[derive(serde::Deserialize)]
        struct LCXPairResponse { data: Option<LCXPairDetail> }

        let response: LCXPairResponse = self.request("GET", &endpoint, None, false).await?;
        let d = response.data.unwrap_or_default();
        Ok(omnibus_core::PairInfo {
            price_decimals: d.precision.price.unwrap_or(5),
            amount_decimals: d.precision.amount.unwrap_or(2),
            min_base: d.min_order.base.unwrap_or(1.0),
            min_quote: d.min_order.quote.unwrap_or(0.01),
        })
    }

    async fn markets(&self) -> Result<Vec<String>, OmniBusError> {
        let endpoint = "/api/pairs";

        #[derive(serde::Deserialize)]
        struct LCXPairsResponse {
            data: Vec<LCXPair>,
        }

        #[derive(serde::Deserialize)]
        #[allow(non_snake_case)]
        struct LCXPair {
            Symbol: String,
            Mode: Option<String>,
        }

        let response: LCXPairsResponse = self.request("GET", endpoint, None, false).await?;

        Ok(response.data.into_iter()
            .filter(|p| p.Mode.as_deref() == Some("trade") || p.Mode.as_deref() == Some("post_only"))
            .map(|p| p.Symbol)
            .collect())
    }
}