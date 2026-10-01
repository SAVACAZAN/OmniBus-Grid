use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    // Coinbase Advanced Trade WS
    println!("=== Coinbase Advanced Trade WS: LCX-USDC ===");
    let cb_result = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        let (mut stream, resp) = connect_async("wss://advanced-trade-ws.coinbase.com").await?;
        println!("  Connected: HTTP {}", resp.status());
        let sub = serde_json::json!({
            "type": "subscribe",
            "product_ids": ["LCX-USDC"],
            "channel": "ticker"
        });
        stream.send(Message::Text(sub.to_string())).await?;
        for _ in 0..4 {
            if let Some(Ok(Message::Text(t))) = stream.next().await {
                println!("  {}", &t[..t.len().min(400)]);
            }
        }
        Ok::<_, Box<dyn std::error::Error>>(())
    }).await;
    match cb_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => println!("  ERROR: {}", e),
        Err(_) => println!("  TIMEOUT"),
    }

    // Old Coinbase WS (for comparison)
    println!("\n=== Coinbase Old WS: LCX-USDC ===");
    let old_result = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let (mut stream, _) = connect_async("wss://ws-feed.exchange.coinbase.com").await?;
        let sub = serde_json::json!({
            "type": "subscribe",
            "product_ids": ["LCX-USDC"],
            "channels": ["ticker"]
        });
        stream.send(Message::Text(sub.to_string())).await?;
        if let Some(Ok(Message::Text(t))) = stream.next().await {
            println!("  {}", &t[..t.len().min(300)]);
        }
        Ok::<_, Box<dyn std::error::Error>>(())
    }).await;
    match old_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => println!("  ERROR: {}", e),
        Err(_) => println!("  TIMEOUT"),
    }
}
