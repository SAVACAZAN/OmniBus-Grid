use async_trait::async_trait;
use omnibus_core::*;
use super::CoinbaseExchange;

#[async_trait]
impl PublicAPI for CoinbaseExchange {
    async fn ticker(&self, symbol: &str) -> Result<Ticker, OmniBusError> {
        let start = std::time::Instant::now();
        let symbol = CoinbaseExchange::normalize_symbol(symbol);
        // product_book gives real best bid/ask from the order book.
        // The products/{symbol} endpoint returns best_bid_price: null for thinly-traded pairs like LCX-USD.
        let path = format!("/api/v3/brokerage/market/product_book?product_id={}&limit=1", symbol);

        #[derive(serde::Deserialize)]
        struct ProductBookResponse {
            pricebook: PriceBook,
            last: Option<String>,
        }
        #[derive(serde::Deserialize)]
        struct PriceBook {
            bids: Vec<BookLevel>,
            asks: Vec<BookLevel>,
        }
        #[derive(serde::Deserialize)]
        struct BookLevel {
            price: String,
        }

        let response: ProductBookResponse = self.request("GET", &path, None, false).await?;

        let bid = response.pricebook.bids.first().and_then(|b| b.price.parse::<f64>().ok()).unwrap_or(0.0);
        let ask = response.pricebook.asks.first().and_then(|a| a.price.parse::<f64>().ok()).unwrap_or(0.0);
        let last = response.last.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(0.0);

        Ok(Ticker {
            exchange: self.name().to_string(),
            symbol: symbol.to_string(),
            last,
            bid,
            ask,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            change_24h_pct: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }
    
    async fn tickers(&self, symbols: &[String]) -> Result<Vec<Ticker>, OmniBusError> {
        let start = std::time::Instant::now();
        let path = "/api/v3/brokerage/market/products";
        
        #[derive(serde::Deserialize)]
        struct CoinbaseProductsResponse {
            products: Vec<CoinbaseProduct>,
        }
        
        #[derive(serde::Deserialize)]
        struct CoinbaseProduct {
            product_id: String,
            price: String,
            bid: String,
            ask: String,
            high_24h: Option<String>,
            low_24h: Option<String>,
            volume_24h: Option<String>,
            price_percentage_change_24h: Option<String>,
        }
        
        let response: CoinbaseProductsResponse = self.request("GET", path, None, false).await?;
        
        let mut tickers = Vec::new();
        for product in response.products {
            if symbols.is_empty() || symbols.contains(&product.product_id) {
                tickers.push(Ticker {
                    exchange: self.name().to_string(),
                    symbol: product.product_id,
                    last: product.price.parse().unwrap_or(0.0),
                    bid: product.bid.parse().unwrap_or(0.0),
                    ask: product.ask.parse().unwrap_or(0.0),
                    high_24h: product.high_24h.and_then(|h| h.parse().ok()),
                    low_24h: product.low_24h.and_then(|l| l.parse().ok()),
                    volume_24h: product.volume_24h.and_then(|v| v.parse().ok()),
                    change_24h_pct: product.price_percentage_change_24h.and_then(|c| c.parse().ok()),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    latency_ms: start.elapsed().as_secs_f64() * 1000.0,
                });
            }
        }
        
        Ok(tickers)
    }
    
    async fn orderbook(&self, symbol: &str, depth: usize) -> Result<OrderBook, OmniBusError> {
        let start = std::time::Instant::now();
        let product_id = symbol.replace('/', "-");
        let mut orderbook = OrderBook::new(self.name().to_string(), product_id.clone());

        // USDC pairs (e.g. LCX-USDC) only exist on Advanced Trade, not Exchange Pro
        if product_id.ends_with("-USDC") {
            let path = format!("/api/v3/brokerage/product_book?product_id={}&limit={}", product_id, depth.max(10));
            let data: serde_json::Value = self.request("GET", &path, None, true).await?;
            let pricebook = data.get("pricebook").unwrap_or(&data);
            if let Some(bids) = pricebook.get("bids").and_then(|b| b.as_array()) {
                for bid in bids.iter().take(depth) {
                    orderbook.bids.push(OrderBookLevel {
                        price: bid.get("price").and_then(|v| v.as_str()?.parse().ok()).unwrap_or(0.0),
                        size:  bid.get("size").and_then(|v| v.as_str()?.parse().ok()).unwrap_or(0.0),
                    });
                }
            }
            if let Some(asks) = pricebook.get("asks").and_then(|a| a.as_array()) {
                for ask in asks.iter().take(depth) {
                    orderbook.asks.push(OrderBookLevel {
                        price: ask.get("price").and_then(|v| v.as_str()?.parse().ok()).unwrap_or(0.0),
                        size:  ask.get("size").and_then(|v| v.as_str()?.parse().ok()).unwrap_or(0.0),
                    });
                }
            }
        } else {
            // Exchange Pro public REST — no auth, same product IDs as the WS feed
            let url = format!("https://api.exchange.coinbase.com/products/{}/book?level=2", product_id);
            let resp = self.client.get(&url).send().await?;
            if !resp.status().is_success() {
                let msg = resp.text().await.unwrap_or_default();
                return Err(OmniBusError::ExchangeError { code: "HTTP".to_string(), message: msg });
            }
            let data: serde_json::Value = resp.json().await?;
            // Exchange Pro format: {"bids": [["price","size",num_orders], ...], "asks": [...]}
            if let Some(bids) = data.get("bids").and_then(|b| b.as_array()) {
                for bid in bids.iter().take(depth) {
                    if let Some(arr) = bid.as_array() {
                        let price: f64 = arr.get(0).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                        let size:  f64 = arr.get(1).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                        if price > 0.0 { orderbook.bids.push(OrderBookLevel { price, size }); }
                    }
                }
            }
            if let Some(asks) = data.get("asks").and_then(|a| a.as_array()) {
                for ask in asks.iter().take(depth) {
                    if let Some(arr) = ask.as_array() {
                        let price: f64 = arr.get(0).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                        let size:  f64 = arr.get(1).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                        if price > 0.0 { orderbook.asks.push(OrderBookLevel { price, size }); }
                    }
                }
            }
        }

        orderbook.calculate_metrics();
        orderbook.latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok(orderbook)
    }
    
    async fn trades(&self, symbol: &str, limit: usize) -> Result<Vec<Trade>, OmniBusError> {
        let symbol = CoinbaseExchange::normalize_symbol(symbol);
        let path = format!("/api/v3/brokerage/market/products/{}/trades?limit={}", symbol, limit);
        
        #[derive(serde::Deserialize)]
        struct CoinbaseTradesResponse {
            trades: Vec<CoinbaseTrade>,
        }
        
        #[derive(serde::Deserialize)]
        struct CoinbaseTrade {
            trade_id: String,
            price: String,
            size: String,
            side: String,
            time: String,
        }
        
        let response: CoinbaseTradesResponse = self.request("GET", &path, None, false).await?;
        
        Ok(response.trades.into_iter().map(|trade| Trade {
            id: trade.trade_id,
            exchange: self.name().to_string(),
            symbol: symbol.to_string(),
            side: if trade.side == "BUY" { Side::Buy } else { Side::Sell },
            price: trade.price.parse().unwrap_or(0.0),
            quantity: trade.size.parse().unwrap_or(0.0),
            timestamp: chrono::DateTime::parse_from_rfc3339(&trade.time)
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0),
            fee: None, fee_coin: None,
        }).collect())
    }
    
    async fn candles(&self, symbol: &str, interval: &str, limit: usize) -> Result<Vec<Candle>, OmniBusError> {
        let symbol = CoinbaseExchange::normalize_symbol(symbol);
        // Coinbase supported: ONE_MINUTE,FIVE_MINUTE,FIFTEEN_MINUTE,THIRTY_MINUTE,
        //   ONE_HOUR,TWO_HOUR,SIX_HOUR,ONE_DAY,ONE_WEEK  (no 4h, no 12h)
        let granularity = match interval {
            "1m"  => "ONE_MINUTE",
            "3m"  => "FIVE_MINUTE",  // nearest
            "5m"  => "FIVE_MINUTE",
            "15m" => "FIFTEEN_MINUTE",
            "30m" => "THIRTY_MINUTE",
            "1h"  => "ONE_HOUR",
            "2h"  => "TWO_HOUR",
            "4h"  => "SIX_HOUR",
            "6h"  => "SIX_HOUR",
            "1d"  => "ONE_DAY",
            "1w"  => "ONE_WEEK",
            _     => "ONE_HOUR",
        };
        let bar_secs: i64 = match interval {
            "1m" => 60, "3m" => 180, "5m" => 300, "15m" => 900, "30m" => 1800,
            "1h" => 3600, "2h" => 7200, "4h" => 21600, "6h" => 21600,
            "1d" => 86400, "1w" => 604800, _ => 3600,
        };

        #[derive(serde::Deserialize)]
        struct CoinbaseCandlesResponse { candles: Vec<CoinbaseCandle> }
        #[derive(serde::Deserialize)]
        struct CoinbaseCandle { start: String, low: String, high: String, open: String, close: String, volume: String }

        // Coinbase max 300 bars per request — paginate with start/end
        let page_size: i64 = 300;
        let mut all_candles: Vec<Candle> = Vec::new();
        let mut end_ts = chrono::Utc::now().timestamp();
        let cutoff = chrono::Utc::now().timestamp() - 10 * 365 * 86400;

        loop {
            let start_ts = end_ts - bar_secs * page_size;
            let path = format!(
                "/api/v3/brokerage/market/products/{}/candles?granularity={}&start={}&end={}",
                symbol, granularity, start_ts, end_ts
            );
            let response: CoinbaseCandlesResponse = self.request("GET", &path, None, false).await?;
            if response.candles.is_empty() { break; }

            let oldest = response.candles.iter()
                .map(|c| c.start.parse::<i64>().unwrap_or(i64::MAX))
                .min().unwrap_or(0);

            for candle in response.candles {
                all_candles.push(Candle {
                    timestamp: candle.start.parse::<i64>().unwrap_or(0) * 1000,
                    open:   candle.open.parse().unwrap_or(0.0),
                    high:   candle.high.parse().unwrap_or(0.0),
                    low:    candle.low.parse().unwrap_or(0.0),
                    close:  candle.close.parse().unwrap_or(0.0),
                    volume: candle.volume.parse().unwrap_or(0.0),
                });
            }

            if limit > 0 && all_candles.len() >= limit { break; }
            end_ts = oldest - 1;
            if end_ts < cutoff { break; }
        }

        all_candles.sort_by_key(|c| c.timestamp);
        all_candles.dedup_by_key(|c| c.timestamp);
        if limit > 0 {
            let len = all_candles.len();
            if len > limit { all_candles = all_candles.split_off(len - limit); }
        }
        Ok(all_candles)
    }
    
    async fn pair_info(&self, symbol: &str) -> Result<omnibus_core::PairInfo, OmniBusError> {
        let cb_symbol = CoinbaseExchange::normalize_symbol(symbol);
        let path = format!("/api/v3/brokerage/market/products/{}", cb_symbol);

        #[derive(serde::Deserialize)]
        struct CoinbaseProductDetail {
            base_increment: Option<String>,
            quote_increment: Option<String>,
            base_min_size: Option<String>,
        }

        let resp: CoinbaseProductDetail = self.request("GET", &path, None, false).await?;
        let base_inc = resp.base_increment.as_deref().unwrap_or("0.01");
        let quote_inc = resp.quote_increment.as_deref().unwrap_or("0.01");
        let amount_dec = base_inc.find('.').map(|i| (base_inc.len() - i - 1) as i64).unwrap_or(2);
        let price_dec = quote_inc.find('.').map(|i| (quote_inc.len() - i - 1) as i64).unwrap_or(2);
        Ok(omnibus_core::PairInfo {
            price_decimals: price_dec,
            amount_decimals: amount_dec,
            min_base: resp.base_min_size.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0.01),
            min_quote: 0.0,
        })
    }

    async fn markets(&self) -> Result<Vec<String>, OmniBusError> {
        let path = "/api/v3/brokerage/market/products";
        
        #[derive(serde::Deserialize)]
        struct CoinbaseProductsResponse {
            products: Vec<CoinbaseProductInfo>,
        }
        
        #[derive(serde::Deserialize)]
        struct CoinbaseProductInfo {
            product_id: String,
            status: String,
        }
        
        let response: CoinbaseProductsResponse = self.request("GET", path, None, false).await?;
        
        Ok(response.products
            .into_iter()
            .filter(|p| p.status == "online" || p.status == "active")
            .map(|p| p.product_id)
            .collect())
    }
}