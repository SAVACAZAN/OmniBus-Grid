use rusqlite::{Connection, params};
use std::sync::Arc;
use parking_lot::Mutex;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use rand::RngCore;

pub struct ApiKeyStore {
    pub(super) conn: Arc<Mutex<Connection>>,
}

// ── AES-256-GCM helpers ───────────────────────────────────────────────────────

fn encrypt(session_key: &[u8; 32], plaintext: &str) -> Result<String, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(session_key));
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher.encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encrypt: {e}"))?;
    // Store as: hex(nonce) + ":" + hex(ciphertext)
    Ok(format!("{}:{}", hex::encode(nonce_bytes), hex::encode(ct)))
}

fn decrypt(session_key: &[u8; 32], stored: &str) -> Result<String, String> {
    let (nonce_hex, ct_hex) = stored.split_once(':')
        .ok_or("Bad encrypted format")?;
    let nonce_bytes = hex::decode(nonce_hex).map_err(|e| format!("Bad nonce: {e}"))?;
    let ct = hex::decode(ct_hex).map_err(|e| format!("Bad ciphertext: {e}"))?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(session_key));
    let pt = cipher.decrypt(Nonce::from_slice(&nonce_bytes), ct.as_ref())
        .map_err(|_| "Decryption failed — wrong session key or corrupted data".to_string())?;
    String::from_utf8(pt).map_err(|e| format!("UTF-8: {e}"))
}

// ── ApiKeyStore ───────────────────────────────────────────────────────────────

impl ApiKeyStore {
    /// Save a new API key pair, encrypted with the session key. Returns new row id.
    pub fn save(
        &self,
        user_id: i64,
        exchange: &str,
        label: &str,
        api_key: &str,
        api_secret: &str,
        session_key: &[u8; 32],
    ) -> Result<i64, String> {
        let enc_key    = encrypt(session_key, api_key)?;
        let enc_secret = encrypt(session_key, api_secret)?;
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO api_keys (user_id, exchange, label, enc_key, enc_secret) VALUES (?1,?2,?3,?4,?5)",
            params![user_id, exchange, label, enc_key, enc_secret],
        ).map_err(|e| format!("DB: {e}"))?;
        Ok(conn.last_insert_rowid())
    }

    /// Get all keys for a user, decrypted. Returns JSON-ready values.
    pub fn get_all(
        &self,
        user_id: i64,
        exchange: Option<&str>,
        session_key: &[u8; 32],
    ) -> Result<Vec<serde_json::Value>, String> {
        let conn = self.conn.lock();
        let rows: Vec<(i64, String, String, String, String)> = {
            let (sql, ex_param): (&str, Option<&str>) = match exchange {
                Some(ex) => (
                    "SELECT id, exchange, label, enc_key, enc_secret FROM api_keys WHERE user_id=?1 AND exchange=?2 AND is_active=1 ORDER BY created_at DESC",
                    Some(ex),
                ),
                None => (
                    "SELECT id, exchange, label, enc_key, enc_secret FROM api_keys WHERE user_id=?1 AND is_active=1 ORDER BY exchange, created_at DESC",
                    None,
                ),
            };
            let mut stmt = conn.prepare(sql).map_err(|e| format!("DB: {e}"))?;
            match ex_param {
                Some(ex) => stmt.query_map(params![user_id, ex], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
                    .map_err(|e| format!("DB: {e}"))?.filter_map(|r| r.ok()).collect(),
                None => stmt.query_map(params![user_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
                    .map_err(|e| format!("DB: {e}"))?.filter_map(|r| r.ok()).collect(),
            }
        };
        drop(conn);

        rows.into_iter().map(|(id, exch, label, enc_key, enc_secret)| {
            let key    = decrypt(session_key, &enc_key)?;
            let secret = decrypt(session_key, &enc_secret)?;
            Ok(serde_json::json!({
                "id":       id,
                "exchange": exch,
                "label":    label,
                "api_key":  key,
                "api_key_masked": format!("{}...{}", &key[..key.len().min(6)], &key[key.len().saturating_sub(4)..]),
                "api_secret": secret,
                "api_secret_masked": "••••••••",
            }))
        }).collect()
    }

    pub fn delete(&self, id: i64, user_id: i64) -> Result<(), String> {
        let conn = self.conn.lock();
        let changed = conn.execute(
            "DELETE FROM api_keys WHERE id=?1 AND user_id=?2",
            params![id, user_id],
        ).map_err(|e| format!("DB: {e}"))?;
        if changed == 0 { return Err("Key not found or not yours".into()); }
        Ok(())
    }

    pub fn rename(&self, id: i64, user_id: i64, label: &str) -> Result<(), String> {
        let label = label.trim();
        if label.is_empty() { return Err("Label cannot be empty".into()); }
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE api_keys SET label=?1 WHERE id=?2 AND user_id=?3",
            params![label, id, user_id],
        ).map_err(|e| format!("DB: {e}"))?;
        Ok(())
    }

    /// Returns decrypted (api_key, api_secret) for a specific key id, verified against user.
    pub fn get_by_id(
        &self,
        id: i64,
        user_id: i64,
        session_key: &[u8; 32],
    ) -> Result<Option<(String, String, String)>, String> {
        let conn = self.conn.lock();
        let row: Option<(String, String, String)> = conn.query_row(
            "SELECT exchange, enc_key, enc_secret FROM api_keys WHERE id=?1 AND user_id=?2 AND is_active=1",
            params![id, user_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ).ok();
        drop(conn);
        match row {
            None => Ok(None),
            Some((exch, enc_key, enc_secret)) => {
                let key    = decrypt(session_key, &enc_key)?;
                let secret = decrypt(session_key, &enc_secret)?;
                Ok(Some((exch, key, secret)))
            }
        }
    }
}
