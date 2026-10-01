use rusqlite::{Connection, params};
use std::sync::Arc;
use parking_lot::Mutex;
use sha2::{Sha256, Digest};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};

pub struct AuthStore {
    pub(super) conn: Arc<Mutex<Connection>>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Derive a 32-byte AES key from password + hex salt using PBKDF2-HMAC-SHA256.
pub fn derive_key(password: &str, salt_hex: &str) -> Result<[u8; 32], String> {
    let salt = hex::decode(salt_hex).map_err(|e| format!("Bad salt: {e}"))?;
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100_000, &mut key);
    Ok(key)
}

fn hash_password(password: &str, salt_hex: &str) -> String {
    let mut h = Sha256::new();
    h.update(password.as_bytes());
    h.update(salt_hex.as_bytes());
    hex::encode(h.finalize())
}

fn random_salt() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

// ── AuthStore ─────────────────────────────────────────────────────────────────

impl AuthStore {
    pub fn get_user_count(&self) -> Result<i64, String> {
        let conn = self.conn.lock();
        conn.query_row("SELECT COUNT(*) FROM auth_users", [], |r| r.get(0))
            .map_err(|e| format!("DB: {e}"))
    }

    /// Register a new user. Returns (user_id, session_key_hex).
    pub fn register(&self, username: &str, password: &str, display_name: &str) -> Result<(i64, [u8; 32]), String> {
        let username = username.trim();
        if username.len() < 3 || username.len() > 32 {
            return Err("Username must be 3–32 characters".into());
        }
        if password.len() < 6 {
            return Err("Password must be at least 6 characters".into());
        }
        let salt = random_salt();
        let hash = hash_password(password, &salt);
        let key  = derive_key(password, &salt)?;
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO auth_users (username, display_name, password_hash, pbkdf2_salt) VALUES (?1,?2,?3,?4)",
            params![username, display_name.trim(), hash, salt],
        ).map_err(|e| {
            if e.to_string().contains("UNIQUE") { "Username already taken".to_string() }
            else { format!("DB: {e}") }
        })?;
        Ok((conn.last_insert_rowid(), key))
    }

    // ── Saved credentials (Remember Me) ──────────────────────────────────────

    /// Derive a machine-local 32-byte key from the DB path — never leaves disk.
    fn machine_key(db_path: &str) -> [u8; 32] {
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(db_path.as_bytes(), b"omibus-local-cred-salt-v1", 50_000, &mut key);
        key
    }

    fn cred_encrypt(machine_key: &[u8; 32], plaintext: &str) -> Result<String, String> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(machine_key));
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ct = cipher.encrypt(nonce, plaintext.as_bytes()).map_err(|e| format!("Encrypt: {e}"))?;
        Ok(format!("{}:{}", hex::encode(nonce_bytes), hex::encode(ct)))
    }

    fn cred_decrypt(machine_key: &[u8; 32], encoded: &str) -> Result<String, String> {
        let (n, c) = encoded.split_once(':').ok_or("Bad format")?;
        let nonce_bytes = hex::decode(n).map_err(|e| format!("Bad nonce: {e}"))?;
        let ct = hex::decode(c).map_err(|e| format!("Bad ct: {e}"))?;
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(machine_key));
        let plain = cipher.decrypt(Nonce::from_slice(&nonce_bytes), ct.as_ref()).map_err(|_| "Decrypt failed".to_string())?;
        String::from_utf8(plain).map_err(|e| format!("UTF-8: {e}"))
    }

    /// Save username + encrypted password for one-click login.
    pub fn save_credentials(&self, db_path: &str, username: &str, password: &str) -> Result<(), String> {
        let mk = Self::machine_key(db_path);
        let enc_pw = Self::cred_encrypt(&mk, password)?;
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('saved_cred_user', ?1)",
            params![username],
        ).map_err(|e| format!("DB: {e}"))?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('saved_cred_pass', ?1)",
            params![enc_pw],
        ).map_err(|e| format!("DB: {e}"))?;
        Ok(())
    }

    /// Load saved credentials. Returns (username, plaintext_password) if present and decryptable.
    pub fn load_credentials(&self, db_path: &str) -> Option<(String, String)> {
        let conn = self.conn.lock();
        let user: Option<String> = conn.query_row(
            "SELECT value FROM settings WHERE key = 'saved_cred_user'", [], |r| r.get(0)
        ).ok();
        let enc: Option<String> = conn.query_row(
            "SELECT value FROM settings WHERE key = 'saved_cred_pass'", [], |r| r.get(0)
        ).ok();
        drop(conn);
        let user = user?;
        let enc  = enc?;
        let mk   = Self::machine_key(db_path);
        let pass = Self::cred_decrypt(&mk, &enc).ok()?;
        Some((user, pass))
    }

    /// Remove saved credentials.
    pub fn forget_credentials(&self) -> Result<(), String> {
        self.conn.lock().execute_batch(
            "DELETE FROM settings WHERE key IN ('saved_cred_user','saved_cred_pass');"
        ).map_err(|e| format!("DB: {e}"))
    }

    /// Login. Returns (user_id, display_name, session_key).
    pub fn login(&self, username: &str, password: &str) -> Result<(i64, String, [u8; 32]), String> {
        let conn = self.conn.lock();
        let (id, display_name, stored_hash, salt): (i64, String, String, String) = conn
            .query_row(
                "SELECT id, display_name, password_hash, pbkdf2_salt FROM auth_users WHERE username = ?1",
                params![username.trim()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .map_err(|_| "Invalid username or password".to_string())?;

        let attempt = hash_password(password, &salt);
        if attempt != stored_hash {
            return Err("Invalid username or password".into());
        }
        drop(conn); // release lock before derive_key
        let key = derive_key(password, &salt)?;
        Ok((id, display_name, key))
    }
}
