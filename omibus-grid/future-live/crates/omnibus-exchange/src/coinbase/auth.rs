use omnibus_core::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64URL, Engine};
use p256::ecdsa::{SigningKey, signature::Signer, Signature};
use p256::SecretKey;
use serde_json::json;

/// Detect if key is Cloud API (JWT/EC) or Legacy (HMAC)
pub fn is_cloud_api_key(api_key: &str) -> bool {
    api_key.starts_with("organizations/")
}

/// Coinbase Cloud API — JWT ES256 signing (manual construction)
pub fn create_jwt(api_key: &str, method: &str, path: &str, pem_secret: &str) -> Result<String, OmniBusError> {
    let now = chrono::Utc::now().timestamp();
    let nonce = format!("{:032x}", rand::random::<u128>());
    // URI must NOT include query params for JWT
    let clean_path = path.split('?').next().unwrap_or(path);
    let uri = format!("{} api.coinbase.com{}", method, clean_path);

    // Header
    let header = json!({
        "alg": "ES256",
        "kid": api_key,
        "nonce": nonce,
        "typ": "JWT"
    });

    // Payload
    let payload = json!({
        "sub": api_key,
        "iss": "cdp",
        "aud": ["cdp_service"],
        "nbf": now,
        "exp": now + 120,
        "uri": uri
    });

    let header_b64 = B64URL.encode(header.to_string().as_bytes());
    let payload_b64 = B64URL.encode(payload.to_string().as_bytes());
    let signing_input = format!("{}.{}", header_b64, payload_b64);

    // Parse PEM and sign with P-256 ECDSA
    let pem_cleaned = pem_secret.trim();
    let der = pem_to_der(pem_cleaned)?;
    let secret_key = SecretKey::from_sec1_der(&der)
        .map_err(|e| OmniBusError::Auth(format!("EC key parse: {}", e)))?;
    let signing_key = SigningKey::from(secret_key);

    let signature: Signature = signing_key.sign(signing_input.as_bytes());
    let sig_b64 = B64URL.encode(signature.to_bytes());

    Ok(format!("{}.{}", signing_input, sig_b64))
}

/// Extract DER bytes from PEM (handles single-line, multi-line, spaces, \r\n)
fn pem_to_der(pem: &str) -> Result<Vec<u8>, OmniBusError> {
    use base64::{engine::general_purpose::STANDARD as B64STD, Engine as _};
    // Remove header/footer/whitespace, extract pure base64
    let b64: String = pem
        .replace("-----BEGIN EC PRIVATE KEY-----", "")
        .replace("-----END EC PRIVATE KEY-----", "")
        .replace("-----BEGIN PRIVATE KEY-----", "")
        .replace("-----END PRIVATE KEY-----", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    B64STD.decode(&b64).map_err(|e| OmniBusError::Auth(format!("PEM decode: {}", e)))
}

/// Legacy HMAC signing (for non-Cloud keys)
pub fn sign_request_hmac(timestamp: i64, method: &str, path: &str, body: &str, secret: &str) -> Result<String, OmniBusError> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let message = format!("{}{}{}{}", timestamp, method, path, body);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| OmniBusError::Auth(format!("HMAC init: {}", e)))?;
    mac.update(message.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}
