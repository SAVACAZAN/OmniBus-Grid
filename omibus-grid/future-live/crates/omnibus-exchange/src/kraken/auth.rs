use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512, Digest};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use omnibus_core::*;

type HmacSha512 = Hmac<Sha512>;

pub fn sign_request(endpoint: &str, post_data: &str, nonce: u64, secret: Option<&String>) -> Result<String, OmniBusError> {
    let raw = secret
        .ok_or_else(|| OmniBusError::Auth("Missing API secret".to_string()))?;

    // Strip all whitespace/newlines that break base64 decode
    let cleaned: String = raw.chars().filter(|c| !c.is_whitespace()).collect();

    let decoded_secret = BASE64.decode(cleaned.as_bytes())
        .map_err(|e| OmniBusError::Auth(format!(
            "Failed to decode Kraken secret (len={}): {}. Make sure you copied the base64 secret from Kraken API settings exactly.",
            cleaned.len(), e
        )))?;
    
    let sha256_hash = {
        let nonce_str = nonce.to_string();
        let mut hasher = Sha256::new();
        hasher.update(nonce_str.as_bytes());
        hasher.update(post_data.as_bytes());
        hasher.finalize()
    };
    
    let mut mac = HmacSha512::new_from_slice(&decoded_secret)
        .map_err(|e| OmniBusError::Auth(format!("HMAC init failed: {}", e)))?;
    
    mac.update(endpoint.as_bytes());
    mac.update(&sha256_hash);
    
    let signature = BASE64.encode(mac.finalize().into_bytes());
    
    Ok(signature)
}