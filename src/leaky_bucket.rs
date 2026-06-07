//! Leaky bucket rate limiter.
//!
//! Models a bucket that "leaks" at a constant rate. Requests are added as drops.
//! When the bucket overflows (exceeds capacity), the request is denied.

/// A leaky bucket rate limiter.
///
/// Water (requests) fills the bucket and drains at `leak_rate` per second.
/// If the water level would exceed `capacity`, the request is rejected.
#[derive(Debug, Clone)]
pub struct LeakyBucket {
    capacity: u64,
    leak_rate: f64, // drops per second
    water_level: f64,
    last_leak_secs: f64,
}

impl LeakyBucket {
    /// Create a new leaky bucket.
    ///
    /// - `capacity`: maximum water level (drops).
    /// - `leak_rate`: drops drained per second.
    pub fn new(capacity: u64, leak_rate: f64) -> Self {
        Self {
            capacity,
            leak_rate,
            water_level: 0.0,
            last_leak_secs: 0.0,
        }
    }

    /// Try to add one drop (request). Returns `true` if accepted.
    pub fn try_request(&mut self, time_secs: f64) -> bool {
        self.leak_to(time_secs);
        if self.water_level < self.capacity as f64 {
            self.water_level += 1.0;
            true
        } else {
            false
        }
    }

    /// Try to add `n` drops. Returns `true` if all accepted.
    pub fn try_request_n(&mut self, n: u64, time_secs: f64) -> bool {
        self.leak_to(time_secs);
        if self.water_level + n as f64 <= self.capacity as f64 {
            self.water_level += n as f64;
            true
        } else {
            false
        }
    }

    /// Drain water based on elapsed time.
    fn leak_to(&mut self, time_secs: f64) {
        let elapsed = time_secs - self.last_leak_secs;
        if elapsed > 0.0 {
            let drained = elapsed * self.leak_rate;
            self.water_level = (self.water_level - drained).max(0.0);
            self.last_leak_secs = time_secs;
        }
    }

    /// Current water level.
    pub fn water_level(&self) -> u64 {
        self.water_level as u64
    }

    /// Current water level as float.
    pub fn water_level_f64(&self) -> f64 {
        self.water_level
    }

    /// Bucket capacity.
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Leak rate (drops per second).
    pub fn leak_rate(&self) -> f64 {
        self.leak_rate
    }

    /// Reset the bucket.
    pub fn reset(&mut self) {
        self.water_level = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_when_empty() {
        let mut lb = LeakyBucket::new(5, 1.0);
        assert!(lb.try_request(0.0));
    }

    #[test]
    fn rejects_when_full() {
        let mut lb = LeakyBucket::new(3, 1.0);
        lb.try_request(0.0);
        lb.try_request(0.0);
        lb.try_request(0.0);
        assert!(!lb.try_request(0.0));
    }

    #[test]
    fn drains_over_time() {
        let mut lb = LeakyBucket::new(5, 2.0);
        // Fill to 5
        for _ in 0..5 {
            lb.try_request(0.0);
        }
        assert_eq!(lb.water_level(), 5);
        // After 2 seconds, 4 drained → level 1
        assert!(lb.try_request(2.0)); // level goes 1→2
        assert_eq!(lb.water_level(), 2);
    }

    #[test]
    fn water_level_f64() {
        let mut lb = LeakyBucket::new(5, 1.0);
        lb.try_request(0.0);
        lb.try_request(0.0);
        assert!((lb.water_level_f64() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn try_request_n_success() {
        let mut lb = LeakyBucket::new(10, 1.0);
        assert!(lb.try_request_n(5, 0.0));
        assert_eq!(lb.water_level(), 5);
    }

    #[test]
    fn try_request_n_overflow() {
        let mut lb = LeakyBucket::new(3, 1.0);
        assert!(!lb.try_request_n(5, 0.0));
        assert_eq!(lb.water_level(), 0); // unchanged
    }

    #[test]
    fn full_drain() {
        let mut lb = LeakyBucket::new(5, 10.0);
        for _ in 0..5 {
            lb.try_request(0.0);
        }
        // After 1 second at 10 drops/sec, fully drained
        assert!(lb.try_request(1.0));
        assert_eq!(lb.water_level(), 1);
    }

    #[test]
    fn reset() {
        let mut lb = LeakyBucket::new(5, 1.0);
        for _ in 0..5 {
            lb.try_request(0.0);
        }
        lb.reset();
        assert_eq!(lb.water_level(), 0);
    }

    #[test]
    fn capacity_accessor() {
        let lb = LeakyBucket::new(42, 1.0);
        assert_eq!(lb.capacity(), 42);
    }

    #[test]
    fn leak_rate_accessor() {
        let lb = LeakyBucket::new(5, 3.5);
        assert!((lb.leak_rate() - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn burst_then_drain_cycle() {
        let mut lb = LeakyBucket::new(5, 1.0);
        // Burst fill
        for _ in 0..5 {
            assert!(lb.try_request(0.0));
        }
        assert!(!lb.try_request(0.0));
        // Drain over 3 seconds
        assert!(lb.try_request(3.0)); // drained 3, level was 5→2, then +1=3
        assert!(lb.try_request(3.0)); // level 3+1=4
        assert!(lb.try_request(3.0)); // level 4+1=5
        assert!(!lb.try_request(3.0)); // full
    }
}
