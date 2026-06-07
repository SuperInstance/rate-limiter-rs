//! Sliding window rate limiter.
//!
//! Tracks individual request timestamps within a rolling time window.
//! A request is allowed only if fewer than `max_requests` occurred in the
//! last `window_secs` seconds.

/// A sliding window rate limiter.
///
/// Stores up to `max_requests` timestamps. When the oldest timestamp falls
/// outside the window, it is evicted.
#[derive(Debug, Clone)]
pub struct SlidingWindow {
    max_requests: u64,
    window_secs: f64,
    timestamps: Vec<f64>,
}

impl SlidingWindow {
    /// Create a new sliding window limiter.
    ///
    /// - `max_requests`: maximum requests allowed within the window.
    /// - `window_secs`: duration of the sliding window in seconds.
    pub fn new(max_requests: u64, window_secs: f64) -> Self {
        Self {
            max_requests,
            window_secs,
            timestamps: Vec::new(),
        }
    }

    /// Try to record a request at the given time. Returns `true` if allowed.
    pub fn try_request(&mut self, time_secs: f64) -> bool {
        self.evict_old(time_secs);
        if self.timestamps.len() < self.max_requests as usize {
            self.timestamps.push(time_secs);
            true
        } else {
            false
        }
    }

    /// Peek whether a request would be allowed without recording it.
    pub fn would_allow(&self, time_secs: f64) -> bool {
        let cutoff = time_secs - self.window_secs;
        let count = self
            .timestamps
            .iter()
            .filter(|&&ts| ts > cutoff)
            .count();
        count < self.max_requests as usize
    }

    /// Current number of requests in the window.
    pub fn current_count(&self) -> usize {
        self.timestamps.len()
    }

    /// Evict timestamps outside the window.
    fn evict_old(&mut self, time_secs: f64) {
        let cutoff = time_secs - self.window_secs;
        self.timestamps.retain(|&ts| ts > cutoff);
    }

    /// Reset all recorded timestamps.
    pub fn reset(&mut self) {
        self.timestamps.clear();
    }

    /// Maximum requests allowed in the window.
    pub fn max_requests(&self) -> u64 {
        self.max_requests
    }

    /// Window duration in seconds.
    pub fn window_secs(&self) -> f64 {
        self.window_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_max() {
        let mut sw = SlidingWindow::new(3, 10.0);
        assert!(sw.try_request(1.0));
        assert!(sw.try_request(1.0));
        assert!(sw.try_request(1.0));
        assert!(!sw.try_request(1.0)); // 4th denied
    }

    #[test]
    fn evicts_old_entries() {
        let mut sw = SlidingWindow::new(2, 5.0);
        assert!(sw.try_request(1.0));
        assert!(sw.try_request(2.0));
        // At t=6.5: cutoff=1.5, ts=1.0 evicted, ts=2.0 stays. Count=1, allowed.
        assert!(sw.try_request(6.5));
        // Now 2 entries: ts=2.0 and ts=6.5. Full.
        assert!(!sw.try_request(6.5));
        // At t=8: cutoff=3.0, ts=2.0 evicted, ts=6.5 stays. Count=1, allowed again.
        assert!(sw.try_request(8.0));
    }

    #[test]
    fn window_expiry() {
        let mut sw = SlidingWindow::new(1, 5.0);
        assert!(sw.try_request(0.0));
        assert!(!sw.try_request(4.0)); // still in window
        assert!(sw.try_request(6.0)); // t=0 expired
    }

    #[test]
    fn current_count() {
        let mut sw = SlidingWindow::new(5, 10.0);
        sw.try_request(1.0);
        sw.try_request(2.0);
        assert_eq!(sw.current_count(), 2);
    }

    #[test]
    fn reset_clears_all() {
        let mut sw = SlidingWindow::new(5, 10.0);
        sw.try_request(1.0);
        sw.try_request(2.0);
        sw.reset();
        assert_eq!(sw.current_count(), 0);
    }

    #[test]
    fn would_allow_peek() {
        let mut sw = SlidingWindow::new(2, 10.0);
        sw.try_request(1.0);
        sw.try_request(2.0);
        assert!(!sw.would_allow(3.0)); // full
        assert!(sw.would_allow(13.0)); // window expired
    }

    #[test]
    fn accessors() {
        let sw = SlidingWindow::new(10, 30.0);
        assert_eq!(sw.max_requests(), 10);
        assert!((sw.window_secs() - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn single_request_window() {
        let mut sw = SlidingWindow::new(1, 1.0);
        assert!(sw.try_request(0.0));
        assert!(!sw.try_request(0.5));
        assert!(sw.try_request(1.1));
    }

    #[test]
    fn boundary_exact() {
        let mut sw = SlidingWindow::new(1, 10.0);
        assert!(sw.try_request(10.0));
        // At t=20 exactly, the cutoff is 10.0, and ts=10.0 is NOT > 10.0
        assert!(sw.try_request(20.0));
    }
}
