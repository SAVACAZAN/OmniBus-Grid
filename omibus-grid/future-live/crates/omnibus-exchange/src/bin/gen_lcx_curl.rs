use rusqlite::Connection;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

type HmacSha256 = Hmac<Sha256>;

fn sign(secret: &str, method: &str, endpoint: &str) -> String {
    let message = format!("{}{}", method, endpoint);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(message.as_bytes());
    BASE64.encode(mac.finalize().into_bytes())
}

fn pct(s: &str) -> String {
    s.replace('+', "%2B").replace('/', "%2F").replace('=', "%3D")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let (api_key, api_secret) = if args.len() >= 3 {
        (args[1].clone(), args[2].clone())
    } else {
        let db_path = format!(
            "{}\\omnibus-terminal\\omnibus.db",
            std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA not set")
        );
        let conn = Connection::open(&db_path).expect("cannot open omnibus.db");
        let mut stmt = conn
            .prepare("SELECT api_key, api_secret FROM api_keys WHERE lower(exchange) LIKE '%lcx%' LIMIT 1")
            .unwrap();
        stmt.query_row([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .expect("no LCX key found in DB")
    };

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // 1. WebSocket upgrade (GET) with API-VERSION header
    let sig_get = pct(&sign(&api_secret, "GET", "/api/auth/ws"));
    let url_wss = format!(
        "wss://exchange-api.lcx.com/api/auth/ws?x-access-key={}&x-access-sign={}&x-access-timestamp={}",
        api_key, sig_get, ts
    );
    let url_https = url_wss.replace("wss://", "https://");

    println!("# 1. WebSocket upgrade (GET) + API-VERSION: 1.1.0:");
    println!(
        "curl -v --no-buffer -i -N \\\n  -H \"Connection: Upgrade\" \\\n  -H \"Upgrade: websocket\" \\\n  -H \"Sec-WebSocket-Version: 13\" \\\n  -H \"Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\" \\\n  -H \"Origin: https://exchange.lcx.com\" \\\n  -H \"API-VERSION: 1.1.0\" \\\n  \"{}\"\n",
        url_https
    );

    // 2. Plain HTTP GET (no WS upgrade) — simpler diagnostic
    println!("# 2. Plain HTTP GET (no WS upgrade):");
    println!("curl -v -H \"API-VERSION: 1.1.0\" \"{}\"", url_https);
    println!();

    // 3. POST with body
    let sig_post = pct(&sign(&api_secret, "POST", "/api/auth/ws"));
    let url_post = format!(
        "https://exchange-api.lcx.com/api/auth/ws?x-access-key={}&x-access-sign={}&x-access-timestamp={}",
        api_key, sig_post, ts
    );
    println!("# 3. POST {{Topic:subscribe, Type:user_trades}} + API-VERSION:");
    println!(
        "curl -v -X POST \\\n  -H \"Content-Type: application/json\" \\\n  -H \"API-VERSION: 1.1.0\" \\\n  -d '{{\"Topic\":\"subscribe\",\"Type\":\"user_trades\"}}' \\\n  \"{}\"",
        url_post
    );
    println!();

    // 4. No signature at all (sanity: should give 400)
    let url_nosig = format!(
        "https://exchange-api.lcx.com/api/auth/ws?x-access-key={}&x-access-timestamp={}",
        api_key, ts
    );
    println!("# 4. No signature (expect 400):");
    println!("curl -v -H \"API-VERSION: 1.1.0\" \"{}\"", url_nosig);
}
