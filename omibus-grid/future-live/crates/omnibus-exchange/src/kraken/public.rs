use async_trait::async_trait;
use omnibus_core::*;
use super::KrakenExchange;

#[async_trait]
impl PublicAPI for KrakenExchange {
    async fn ticker(&self, symbol: &str) -> Result<Ticker, OmniBusError> {
        let start = std::time::Instant::now();
        let endpoint = "/0/public/Ticker";
        let data = serde_json::json!({
            "pair": symbol.replace("/", ""),
        });
        
        let response: serde_json::Value = self.request(endpoint, Some(data), false).await?;
        
        let ticker_data = response.as_object()
            .and_then(|obj| obj.values().next())
            .ok_or_else(|| OmniBusError::ExchangeError {
                code: "PARSE_ERROR".to_string(),
                message: "Failed to parse ticker response".to_string(),
            })?;
        
        let last = ticker_data["c"][0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
        let bid = ticker_data["b"][0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
        let ask = ticker_data["a"][0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
        let high = ticker_data["h"][1].as_str().and_then(|s| s.parse().ok());
        let low = ticker_data["l"][1].as_str().and_then(|s| s.parse().ok());
        let volume = ticker_data["v"][1].as_str().and_then(|s| s.parse().ok());
        
        Ok(Ticker {
            exchange: self.name().to_string(),
            symbol: symbol.to_string(),
            last,
            bid,
            ask,
            high_24h: high,
            low_24h: low,
            volume_24h: volume,
            change_24h_pct: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }
    
    async fn tickers(&self, symbols: &[String]) -> Result<Vec<Ticker>, OmniBusError> {
        let start = std::time::Instant::now();
        let endpoint = "/0/public/Ticker";
        
        let pair_param = if symbols.is_empty() {
            "all".to_string()
        } else {
            symbols.iter()
                .map(|s| s.replace("/", ""))
                .collect::<Vec<_>>()
                .join(",")
        };
        
        let data = serde_json::json!({
            "pair": pair_param,
        });
        
        let response: serde_json::Value = self.request(endpoint, Some(data), false).await?;
        
        let mut tickers = Vec::new();
        if let Some(obj) = response.as_object() {
            for (kraken_symbol, ticker_data) in obj {
                let last = ticker_data["c"][0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                let bid = ticker_data["b"][0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                let ask = ticker_data["a"][0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                
                // Convert Kraken symbol format (XBTUSD -> BTC/USD)
                let symbol = if kraken_symbol == "XBTUSD" {
                    "BTC/USD".to_string()
                } else if kraken_symbol.len() > 3 {
                    let base = &kraken_symbol[..kraken_symbol.len()-3];
                    let quote = &kraken_symbol[kraken_symbol.len()-3..];
                    format!("{}/{}", base, quote)
                } else {
                    kraken_symbol.clone()
                };
                
                tickers.push(Ticker {
                    exchange: self.name().to_string(),
                    symbol,
                    last,
                    bid,
                    ask,
                    high_24h: ticker_data["h"][1].as_str().and_then(|s| s.parse().ok()),
                    low_24h: ticker_data["l"][1].as_str().and_then(|s| s.parse().ok()),
                    volume_24h: ticker_data["v"][1].as_str().and_then(|s| s.parse().ok()),
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
        let endpoint = "/0/public/Depth";
        let data = serde_json::json!({
            "pair": symbol.replace("/", ""),
            "count": depth,
        });
        
        let response: serde_json::Value = self.request(endpoint, Some(data), false).await?;
        
        let book_data = response.as_object()
            .and_then(|obj| obj.values().next())
            .ok_or_else(|| OmniBusError::ExchangeError {
                code: "PARSE_ERROR".to_string(),
                message: "Failed to parse orderbook".to_string(),
            })?;
        
        let mut orderbook = OrderBook::new(self.name().to_string(), symbol.to_string());
        
        if let Some(bids) = book_data["bids"].as_array() {
            for bid in bids.iter().take(depth) {
                if let Some(price_size) = bid.as_array() {
                    orderbook.bids.push(OrderBookLevel {
                        price: price_size[0].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                        size: price_size[1].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    });
                }
            }
        }
        
        if let Some(asks) = book_data["asks"].as_array() {
            for ask in asks.iter().take(depth) {
                if let Some(price_size) = ask.as_array() {
                    orderbook.asks.push(OrderBookLevel {
                        price: price_size[0].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                        size: price_size[1].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    });
                }
            }
        }
        
        orderbook.calculate_metrics();
        orderbook.latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(orderbook)
    }
    
    async fn trades(&self, symbol: &str, limit: usize) -> Result<Vec<Trade>, OmniBusError> {
        let endpoint = "/0/public/Trades";
        let data = serde_json::json!({
            "pair": symbol.replace("/", ""),
            "count": limit,
        });
        
        let response: serde_json::Value = self.request(endpoint, Some(data), false).await?;
        
        let trades_data = response.as_object()
            .and_then(|obj| obj.values().next())
            .ok_or_else(|| OmniBusError::ExchangeError {
                code: "PARSE_ERROR".to_string(),
                message: "Failed to parse trades".to_string(),
            })?;
        
        let mut trades = Vec::new();
        if let Some(trade_list) = trades_data.as_array() {
            for trade_item in trade_list {
                if let Some(trade_array) = trade_item.as_array() {
                    if trade_array.len() >= 4 {
                        let price = trade_array[0].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                        let volume = trade_array[1].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                        let time = trade_array[2].as_f64().unwrap_or(0.0) as i64;
                        let side = if trade_array[3].as_str() == Some("b") { Side::Buy } else { Side::Sell };
                        
                        trades.push(Trade {
                            id: format!("{}_{}", time, price),
                            exchange: self.name().to_string(),
                            symbol: symbol.to_string(),
                            side, price, quantity: volume,
                            timestamp: time * 1000, fee: None, fee_coin: None,
                        });
                    }
                }
            }
        }
        
        Ok(trades)
    }
    
    async fn candles(&self, symbol: &str, interval: &str, limit: usize) -> Result<Vec<Candle>, OmniBusError> {
        let endpoint = "/0/public/OHLC";
        // Kraken supported intervals (minutes): 1,5,15,30,60,240,1440,10080,21600
        let interval_minutes: i64 = match interval {
            "1m"  => 1,
            "3m"  => 5,    // nearest: 5m
            "5m"  => 5,
            "15m" => 15,
            "30m" => 30,
            "1h"  => 60,
            "2h"  => 60,   // nearest: 1h
            "4h"  => 240,
            "1d"  => 1440,
            "1w"  => 10080,
            _     => 60,
        };
        // Kraken returns max 720 bars per call; paginate with `since` to fill limit
        let bar_secs = interval_minutes * 60;
        let mut all_candles: Vec<Candle> = Vec::new();
        let cutoff = chrono::Utc::now().timestamp() - 10 * 365 * 86400;
        // limit==0: start from 10 years ago (fetch everything); else just enough pages
        let mut since = if limit == 0 {
            cutoff
        } else {
            chrono::Utc::now().timestamp() - bar_secs * (limit as i64 + 720)
        };

        loop {
            let data = serde_json::json!({
                "pair": symbol.replace("/", ""),
                "interval": interval_minutes,
                "since": since,
            });
            let response: serde_json::Value = self.request(endpoint, Some(data), false).await?;

            // Kraken result: {"XBTUSD": [[...], ...], "last": 1234}
            let candles_data = response.as_object()
                .and_then(|obj| obj.values().find(|v| v.is_array()));
            let last_ts = response.get("last").and_then(|v| v.as_i64());

            if let Some(arr) = candles_data.and_then(|v| v.as_array()) {
                if arr.is_empty() { break; }
                for candle in arr.iter() {
                    if let Some(ca) = candle.as_array() {
                        if ca.len() >= 6 {
                            all_candles.push(Candle {
                                timestamp: ca[0].as_i64().unwrap_or(0) * 1000,
                                open:  ca[1].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                                high:  ca[2].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                                low:   ca[3].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                                close: ca[4].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                                volume: ca[6].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                            });
                        }
                    }
                }
            } else { break; }

            if limit > 0 && all_candles.len() >= limit { break; }
            if let Some(next) = last_ts {
                if next <= since || next < cutoff { break; }
                since = next;
            } else { break; }
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
        let kr_pair = symbol.replace("/", "").replace("BTC", "XBT");
        let endpoint = format!("/0/public/AssetPairs?pair={}", kr_pair);
        let response: serde_json::Value = self.request(&endpoint, None, false).await?;

        let base = symbol.split(|c| c == '/' || c == '-').next().unwrap_or("").to_uppercase();
        const LOW_DECIMAL_COINS: &[&str] = &["LCX", "DOGE", "SHIB", "HBAR", "SAND", "MANA", "GRT", "ARB", "FLOKI"];

        if let Some(info) = response.as_object().and_then(|obj| obj.values().next()) {
            let mut amount_dec = info.get("lot_decimals").and_then(|v| v.as_i64()).unwrap_or(8);
            if LOW_DECIMAL_COINS.contains(&base.as_str()) { amount_dec = 1; }
            return Ok(omnibus_core::PairInfo {
                price_decimals: info.get("pair_decimals").and_then(|v| v.as_i64()).unwrap_or(5),
                amount_decimals: amount_dec,
                min_base: info.get("ordermin").and_then(|v| v.as_str()?.parse().ok()).unwrap_or(1.0),
                min_quote: 0.0,
            });
        }
        Ok(omnibus_core::PairInfo::default())
    }

    async fn markets(&self) -> Result<Vec<String>, OmniBusError> {
        let endpoint = "/0/public/AssetPairs";

        let response: serde_json::Value = self.request(endpoint, None, false).await?;
        
        let mut markets = Vec::new();
        if let Some(obj) = response.as_object() {
            for (symbol, info) in obj {
                if let Some(ws_name) = info["wsname"].as_str() {
                    markets.push(ws_name.to_string());
                } else {
                    markets.push(symbol.clone());
                }
            }
        }
        
        Ok(markets)
    }
}