use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    println!("Connecting to wss://ws.kraken.com ...");
    let (mut stream, resp) = connect_async("wss://ws.kraken.com").await
        .expect("connect failed");
    println!("Connected: HTTP {}", resp.status());

    // Exact same subscription as FeedHub sends
    let pairs = vec![
        "LCX/USD", "XBT/USD", "XBT/EUR", "ETH/USD", "ETH/EUR", "ETH/XBT",
        "SOL/USD", "SOL/EUR", "XRP/USD", "XRP/EUR",
        "DOGE/USD", "ADA/USD", "DOT/USD", "LINK/USD",
        "AVAX/USD", "UNI/USD", "LTC/USD", "USDC/USD",
        "USDT/USD", "MATIC/USD", "ATOM/USD",
    ];

    let hb = serde_json::json!({"event":"subscribe","subscription":{"name":"heartbeat"}});
    stream.send(Message::Text(hb.to_string())).await.unwrap();

    let ticker = serde_json::json!({"event":"subscribe","pair": pairs,"subscription":{"name":"ticker"}});
    stream.send(Message::Text(ticker.to_string())).await.unwrap();
    println!("Subscriptions sent. Reading 10 messages...\n");

    for i in 0..10 {
        match tokio::time::timeout(std::time::Duration::from_secs(5), stream.next()).await {
            Ok(Some(Ok(Message::Text(t)))) => println!("[{}] {}", i, &t[..t.len().min(200)]),
            Ok(Some(Ok(Message::Close(f)))) => { println!("CLOSED: {:?}", f); break; }
            Ok(Some(Err(e))) => { println!("ERROR: {}", e); break; }
            Ok(None) => { println!("Stream ended"); break; }
            Err(_) => { println!("[{}] timeout", i); break; }
            _ => {}
        }
    }
}
