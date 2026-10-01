use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use omnibus_core::*;

type HmacSha256 = Hmac<Sha256>;

/// LCX API signature: HMAC-SHA256(secret_raw_bytes, METHOD + ENDPOINT + JSON(payload))
/// Secret is used as RAW bytes (NOT base64 decoded)
/// Output is base64 encoded
pub fn sign_request(method: &str, endpoint: &str, body: &str, secret: Option<&String>) -> Result<String, OmniBusError> {
    let secret_str = secret
        .ok_or_else(|| OmniBusError::Auth("Missing API secret".to_string()))?;

    // LCX uses secret as raw bytes (NOT base64 decoded)
    let secret_bytes = secret_str.as_bytes();

    // Message = METHOD + ENDPOINT + BODY (no timestamp!)
    let message = format!("{}{}{}", method, endpoint, body);

    let mut mac = HmacSha256::new_from_slice(secret_bytes)
        .map_err(|e| OmniBusError::Auth(format!("HMAC init failed: {}", e)))?;

    mac.update(message.as_bytes());
    let result = mac.finalize();
    let signature = BASE64.encode(result.into_bytes());

    Ok(signature)
}

/// Alias for compatibility
pub fn sign_request_with_timestamp(method: &str, endpoint: &str, _timestamp: &str, body: &str, secret: Option<&String>) -> Result<String, OmniBusError> {
    // LCX does NOT include timestamp in signature — only in header
    sign_request(method, endpoint, body, secret)
}
