/// Token-bucket rate limiter shared by Kraken and Coinbase adapters.
/// Blocks the calling thread until a token is available.
pub(crate) struct TokenBucket {
    tokens: usize,
    max_tokens: usize,
    last_refill: std::time::Instant,
    refill_rate: f64,
}

impl TokenBucket {
    pub(crate) fn new(tokens_per_second: usize) -> Self {
        Self {
            tokens: tokens_per_second,
            max_tokens: tokens_per_second,
            last_refill: std::time::Instant::now(),
            refill_rate: tokens_per_second as f64,
        }
    }

    pub(crate) fn acquire(&mut self) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let new_tokens = (elapsed * self.refill_rate) as usize;

        if new_tokens > 0 {
            self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
            self.last_refill = now;
        }

        while self.tokens == 0 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let elapsed = now.elapsed().as_secs_f64();
            let new_tokens = (elapsed * self.refill_rate) as usize;
            if new_tokens > 0 {
                self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
            }
        }

        self.tokens -= 1;
    }
}
