mod auth;
mod api_keys;

pub use auth::AuthStore;
pub use api_keys::ApiKeyStore;

use rusqlite::Connection;
use std::sync::Arc;
use parking_lot::Mutex;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
    pub auth:     AuthStore,
    pub api_keys: ApiKeyStore,
    pub path:     String,
}

impl Database {
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("DB open: {e}"))?;
        let conn = Arc::new(Mutex::new(conn));
        let db = Self {
            auth:     AuthStore     { conn: conn.clone() },
            api_keys: ApiKeyStore   { conn: conn.clone() },
            conn,
            path: path.to_string(),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn.lock().execute_batch("
            CREATE TABLE IF NOT EXISTS auth_users (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                username      TEXT    NOT NULL UNIQUE,
                display_name  TEXT    NOT NULL DEFAULT '',
                password_hash TEXT    NOT NULL,
                pbkdf2_salt   TEXT    NOT NULL,
                created_at    TEXT    DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS api_keys (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id    INTEGER NOT NULL,
                exchange   TEXT    NOT NULL,
                label      TEXT    NOT NULL DEFAULT 'Main Key',
                enc_key    TEXT    NOT NULL,
                enc_secret TEXT    NOT NULL,
                is_active  INTEGER DEFAULT 1,
                created_at TEXT    DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_apikeys_user ON api_keys(user_id, exchange);

            CREATE TABLE IF NOT EXISTS settings (
                key        TEXT PRIMARY KEY,
                value      TEXT NOT NULL,
                updated_at TEXT DEFAULT (datetime('now'))
            );
        ").map_err(|e| format!("DB init: {e}"))
    }
}
