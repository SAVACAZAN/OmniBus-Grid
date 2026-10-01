use std::sync::{Arc, Mutex};

pub type CallbackFn<T> = Box<dyn Fn(T) + Send + Sync>;

/// Shared callback registry for a single data type (Ticker, OrderBook, or Trade).
/// Wraps `Arc<Mutex<Vec<(symbol, callback)>>>` so WebSocket structs avoid
/// repeating the same three-field boilerplate and lock/retain patterns.
pub struct CallbackStore<T> {
    inner: Arc<Mutex<Vec<(String, CallbackFn<T>)>>>,
}

impl<T: Clone + Send + 'static> CallbackStore<T> {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn push(&self, symbol: &str, cb: CallbackFn<T>) {
        self.inner.lock().unwrap().push((symbol.to_string(), cb));
    }

    pub fn remove(&self, symbol: &str) {
        self.inner.lock().unwrap().retain(|(s, _)| s != symbol);
    }

    /// Call every callback registered for `symbol`, cloning `payload` for each.
    pub fn invoke(&self, symbol: &str, payload: T) {
        let callbacks = self.inner.lock().unwrap();
        for (sym, cb) in callbacks.iter() {
            if sym == symbol {
                cb(payload.clone());
            }
        }
    }
}

impl<T> Clone for CallbackStore<T> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

impl<T: Clone + Send + 'static> Default for CallbackStore<T> {
    fn default() -> Self {
        Self::new()
    }
}
