//! # rate-limiter-rs
//!
//! A pure-Rust collection of rate limiting algorithms with no external dependencies.
//!
//! ## Algorithms
//!
//! - **Token Bucket** — tokens accumulate at a fixed rate; each request consumes one.
//! - **Sliding Window** — tracks request timestamps within a rolling time window.
//! - **Fixed Window** — counts requests in fixed time buckets that reset periodically.
//! - **Leaky Bucket** — processes requests at a constant rate, queueing excess.
//!
//! ## Quick Start
//!
//! ```
//! use rate_limiter_rs::token_bucket::TokenBucket;
//!
//! let mut bucket = TokenBucket::new(5, 1.0); // 5 tokens max, 1 token/sec refill
//! assert!(bucket.try_acquire()); // consumes 1 token
//! assert_eq!(bucket.available_tokens(), 4);
//! ```

pub mod config;
pub mod fixed_window;
pub mod leaky_bucket;
pub mod sliding_window;
pub mod token_bucket;
