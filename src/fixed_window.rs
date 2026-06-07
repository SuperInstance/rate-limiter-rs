//! Fixed window rate limiter.
//!
//! Divides time into fixed windows of `window_secs` seconds. Each window
//! independently tracks request counts up to `max_requests`. At window
//! boundaries, the counter resets.

/// A fixed window rate limiter.
#[derive(Debug, Clone)]
pub struct FixedWindow {
    max_requests: u64,
    window_secs: u64,
    current_count: u64,
    window_start_secs: u64,
}

impl FixedWindow {
    /// Create a new fixed window limiter.
    ///
    /// - `max_requests`: maximum requests per window.
    /// - `window_secs`: duration of each window in seconds.
    pub fn new(max_requests: u64, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_secs,
            current_count: 0,
            window_start_secs: 0,
        }
    }

    /// Try to record a request at the given epoch second. Returns `true` if allowed.
    pub fn try_request(&mut self, time_secs: u64) -> bool {
        self.advance_window(time_secs);
        if self.current_count < self.max_requests {
            self.current_count += 1;
            true
        } else {
            false
        }
    }

    /// Check whether a request would be allowed without incrementing.
    pub fn would_allow(&mut self, time_secs: u64) -> bool {
        self.advance_window(time_secs);
        self.current_count < self.max_requests
    }

    /// Current request count in the active window.
    pub fn current_count(&self) -> u64 {
        self.current_count
    }

    /// Get the current window start time.
    pub fn window_start(&self) -> u64 {
        self.window_start_secs
    }

    /// Maximum requests per window.
    pub fn max_requests(&self) -> u64 {
        self.max_requests
    }

    /// Window duration in seconds.
    pub fn window_secs(&self) -> u64 {
        self.window_secs
    }

    /// Reset the limiter.
    pub fn reset(&mut self) {
        self.current_count = 0;
        self.window_start_secs = 0;
    }

    fn advance_window(&mut self, time_secs: u64) {
        if time_secs >= self.window_start_secs + self.window_secs {
            let windows_elapsed = (time_secs - self.window_start_secs) / self.window_secs;
            self.window_start_secs += windows_elapsed * self.window_secs;
            self.current_count = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_max() {
        let mut fw = FixedWindow::new(3, 10);
        assert!(fw.try_request(0));
        assert!(fw.try_request(0));
        assert!(fw.try_request(0));
        assert!(!fw.try_request(0));
    }

    #[test]
    fn resets_at_window_boundary() {
        let mut fw = FixedWindow::new(2, 10);
        assert!(fw.try_request(5));
        assert!(fw.try_request(8));
        assert!(!fw.try_request(9));
        assert!(fw.try_request(10)); // new window
    }

    #[test]
    fn current_count_tracking() {
        let mut fw = FixedWindow::new(5, 10);
        fw.try_request(0);
        fw.try_request(1);
        fw.try_request(2);
        assert_eq!(fw.current_count(), 3);
    }

    #[test]
    fn window_start_tracking() {
        let mut fw = FixedWindow::new(5, 10);
        assert_eq!(fw.window_start(), 0);
        fw.try_request(15);
        assert_eq!(fw.window_start(), 10);
    }

    #[test]
    fn would_allow_no_side_effect() {
        let mut fw = FixedWindow::new(2, 10);
        fw.try_request(0);
        fw.try_request(0);
        assert!(!fw.would_allow(5));
        assert_eq!(fw.current_count(), 2); // unchanged
    }

    #[test]
    fn reset_clears() {
        let mut fw = FixedWindow::new(5, 10);
        fw.try_request(1);
        fw.try_request(2);
        fw.reset();
        assert_eq!(fw.current_count(), 0);
    }

    #[test]
    fn accessors() {
        let fw = FixedWindow::new(10, 60);
        assert_eq!(fw.max_requests(), 10);
        assert_eq!(fw.window_secs(), 60);
    }

    #[test]
    fn large_time_jump() {
        let mut fw = FixedWindow::new(1, 10);
        assert!(fw.try_request(0));
        assert!(!fw.try_request(5));
        assert!(fw.try_request(100)); // jumped many windows
    }

    #[test]
    fn window_boundary_exact() {
        let mut fw = FixedWindow::new(1, 10);
        assert!(fw.try_request(0));
        assert!(!fw.try_request(9));
        assert!(fw.try_request(10));
    }

    #[test]
    fn multiple_windows() {
        let mut fw = FixedWindow::new(2, 5);
        assert!(fw.try_request(0)); // window [0,5)
        assert!(fw.try_request(4));
        assert!(fw.try_request(5)); // window [5,10)
        assert!(fw.try_request(9));
        assert!(!fw.try_request(9));
    }
}
