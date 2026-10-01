// Tests LCX public WS connectivity: tries /ws then root /
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    let urls = [
        "wss://exchange-api.lcx.com/ws",
        "wss://exchange-api.lcx.com/",
    ];

    for url in &urls {
        println!("\n=== Testing {} ===", url);
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            connect_async(*url),
        ).await {
            Ok(Ok((mut stream, resp))) => {
                println!("Connected! HTTP status: {}", resp.status());
                // Send ticker subscription
                let sub = serde_json::json!({"Topic":"subscribe","Type":"ticker","Pair":"BTC/EUR"});
                let _ = stream.send(Message::Text(sub.to_string())).await;
                println!("Sent: {}", sub);
                // Wait for up to 3 messages
                for i in 0..3 {
                    match tokio::time::timeout(std::time::Duration::from_secs(5), stream.next()).await {
                        Ok(Some(Ok(msg))) => println!("MSG {}: {:?}", i, &msg.to_string()[..msg.to_string().len().min(200)]),
                        Ok(Some(Err(e))) => { println!("Error: {}", e); break; }
                        Ok(None) => { println!("Stream closed"); break; }
                        Err(_) => { println!("Timeout waiting for message"); break; }
                    }
                }
            }
            Ok(Err(e)) => println!("Connect error: {}", e),
            Err(_) => println!("Timeout connecting"),
        }
    }
}
