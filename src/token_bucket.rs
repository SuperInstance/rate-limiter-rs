//! Token bucket rate limiter.
//!
//! Tokens accumulate at a fixed `refill_rate` (tokens per second) up to `capacity`.
//! Each [`TokenBucket::try_acquire`] consumes one token. When the bucket is empty,
//! requests are denied until tokens replenish.

/// A token bucket rate limiter.
///
/// # Time Model
///
/// Uses an internal timestamp (seconds as `f64`) that must be advanced manually
/// via [`TokenBucket::advance_time`] or set via [`TokenBucket::set_time`].
/// This allows deterministic testing.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: u64,
    refill_rate: f64,
    tokens: f64,
    last_refill_secs: f64,
}

impl TokenBucket {
    /// Create a new token bucket.
    ///
    /// - `capacity`: maximum number of tokens.
    /// - `refill_rate`: tokens added per second.
    ///
    /// The bucket starts full.
    pub fn new(capacity: u64, refill_rate: f64) -> Self {
        Self {
            capacity,
            refill_rate,
            tokens: capacity as f64,
            last_refill_secs: 0.0,
        }
    }

    /// Create with a starting time.
    pub fn with_time(capacity: u64, refill_rate: f64, time_secs: f64) -> Self {
        Self {
            capacity,
            refill_rate,
            tokens: capacity as f64,
            last_refill_secs: time_secs,
        }
    }

    /// Try to acquire one token. Returns `true` if successful.
    pub fn try_acquire(&mut self) -> bool {
        self.try_acquire_n(1)
    }

    /// Try to acquire `n` tokens. Returns `true` if successful.
    pub fn try_acquire_n(&mut self, n: u64) -> bool {
        if self.tokens >= n as f64 {
            self.tokens -= n as f64;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time since last refill.
    pub fn refill(&mut self) {
        let now = self.last_refill_secs;
        self.refill_to(now);
    }

    /// Refill tokens up to a given time.
    pub fn refill_to(&mut self, time_secs: f64) {
        let elapsed = time_secs - self.last_refill_secs;
        if elapsed > 0.0 {
            let added = elapsed * self.refill_rate;
            self.tokens = (self.tokens + added).min(self.capacity as f64);
            self.last_refill_secs = time_secs;
        }
    }

    /// Advance the internal clock by `secs` seconds and refill.
    pub fn advance_time(&mut self, secs: f64) {
        let new_time = self.last_refill_secs + secs;
        self.refill_to(new_time);
    }

    /// Set the internal clock to a specific time (must be >= current).
    pub fn set_time(&mut self, time_secs: f64) {
        self.refill_to(time_secs);
    }

    /// Current number of available tokens (truncated to integer).
    pub fn available_tokens(&self) -> u64 {
        self.tokens as u64
    }

    /// Current fractional token count.
    pub fn available_tokens_f64(&self) -> f64 {
        self.tokens
    }

    /// Maximum capacity.
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Refill rate (tokens per second).
    pub fn refill_rate(&self) -> f64 {
        self.refill_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bucket_is_full() {
        let b = TokenBucket::new(10, 1.0);
        assert_eq!(b.available_tokens(), 10);
    }

    #[test]
    fn try_acquire_decrements() {
        let mut b = TokenBucket::new(5, 1.0);
        assert!(b.try_acquire());
        assert_eq!(b.available_tokens(), 4);
    }

    #[test]
    fn try_acquire_n_multiple() {
        let mut b = TokenBucket::new(10, 1.0);
        assert!(b.try_acquire_n(3));
        assert_eq!(b.available_tokens(), 7);
    }

    #[test]
    fn try_acquire_n_fails_when_insufficient() {
        let mut b = TokenBucket::new(5, 1.0);
        assert!(!b.try_acquire_n(6));
        assert_eq!(b.available_tokens(), 5); // unchanged
    }

    #[test]
    fn exhaust_and_fail() {
        let mut b = TokenBucket::new(2, 1.0);
        assert!(b.try_acquire());
        assert!(b.try_acquire());
        assert!(!b.try_acquire());
    }

    #[test]
    fn refill_after_time() {
        let mut b = TokenBucket::new(10, 2.0); // 2 tokens/sec
        b.try_acquire_n(10); // empty
        b.advance_time(3.0); // 6 tokens
        assert_eq!(b.available_tokens(), 6);
    }

    #[test]
    fn refill_capped_at_capacity() {
        let mut b = TokenBucket::new(5, 10.0);
        b.advance_time(100.0);
        assert_eq!(b.available_tokens(), 5);
    }

    #[test]
    fn fractional_tokens() {
        let mut b = TokenBucket::new(10, 0.5); // 1 token every 2 sec
        b.try_acquire_n(10);
        b.advance_time(1.0);
        // 0.5 tokens after 1 second
        assert!(b.available_tokens_f64() > 0.0);
        assert_eq!(b.available_tokens(), 0); // truncated
    }

    #[test]
    fn capacity_accessor() {
        let b = TokenBucket::new(42, 1.0);
        assert_eq!(b.capacity(), 42);
    }

    #[test]
    fn refill_rate_accessor() {
        let b = TokenBucket::new(10, 3.5);
        assert!((b.refill_rate() - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn burst_then_refill_cycle() {
        let mut b = TokenBucket::new(5, 1.0);
        // Burst
        for _ in 0..5 {
            assert!(b.try_acquire());
        }
        assert!(!b.try_acquire());
        // Wait 3 seconds
        b.advance_time(3.0);
        assert_eq!(b.available_tokens(), 3);
        assert!(b.try_acquire());
        assert!(b.try_acquire());
        assert!(b.try_acquire());
        assert!(!b.try_acquire());
    }

    #[test]
    fn set_time_advances() {
        let mut b = TokenBucket::with_time(10, 1.0, 0.0);
        b.try_acquire_n(10);
        b.set_time(5.0);
        assert_eq!(b.available_tokens(), 5);
    }

    #[test]
    fn zero_elapsed_no_refill() {
        let mut b = TokenBucket::new(10, 1.0);
        b.try_acquire_n(10);
        b.advance_time(0.0);
        assert_eq!(b.available_tokens(), 0);
    }
}
