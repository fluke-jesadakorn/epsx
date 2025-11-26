use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
// use std::sync::Arc; // Removed - unused import
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{warn, error, debug};

/// Circuit breaker for blockchain RPC calls
/// Prevents cascade failures when blockchain endpoints are down
pub struct CircuitBreaker {
    failure_count: AtomicUsize,
    last_failure_time: std::sync::Mutex<Option<Instant>>,
    is_open: AtomicBool,
    failure_threshold: usize,
    timeout_duration: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, timeout_duration: Duration) -> Self {
        Self {
            failure_count: AtomicUsize::new(0),
            last_failure_time: std::sync::Mutex::new(None),
            is_open: AtomicBool::new(false),
            failure_threshold,
            timeout_duration,
        }
    }

    /// Check if circuit breaker allows the call
    pub fn can_execute(&self) -> bool {
        if !self.is_open.load(Ordering::Relaxed) {
            return true;
        }

        // Check if timeout has passed
        if let Ok(last_failure) = self.last_failure_time.lock() {
            if let Some(last_failure_time) = *last_failure {
                if last_failure_time.elapsed() > self.timeout_duration {
                    debug!("Circuit breaker timeout elapsed, allowing retry");
                    self.reset();
                    return true;
                }
            }
        }

        false
    }

    /// Record successful call
    pub fn record_success(&self) {
        if self.failure_count.load(Ordering::Relaxed) > 0 {
            debug!("Circuit breaker: successful call, resetting failure count");
            self.reset();
        }
    }

    /// Record failed call
    pub fn record_failure(&self) {
        let new_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        
        if let Ok(mut last_failure) = self.last_failure_time.lock() {
            *last_failure = Some(Instant::now());
        }

        if new_count >= self.failure_threshold {
            warn!(
                failure_count = new_count,
                threshold = self.failure_threshold,
                "Circuit breaker opened due to excessive failures"
            );
            self.is_open.store(true, Ordering::Relaxed);
        }
    }

    fn reset(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.is_open.store(false, Ordering::Relaxed);
        if let Ok(mut last_failure) = self.last_failure_time.lock() {
            *last_failure = None;
        }
    }

    /// Get current state for monitoring
    pub fn state(&self) -> CircuitBreakerState {
        CircuitBreakerState {
            is_open: self.is_open.load(Ordering::Relaxed),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            failure_threshold: self.failure_threshold,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerState {
    pub is_open: bool,
    pub failure_count: usize,
    pub failure_threshold: usize,
}

/// Exponential backoff retry policy
pub struct RetryPolicy {
    max_retries: usize,
    base_delay: Duration,
    max_delay: Duration,
}

impl RetryPolicy {
    pub fn new(max_retries: usize, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
        }
    }

    /// Execute with exponential backoff retry
    pub async fn retry_with_backoff<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Debug,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = std::cmp::min(
                    self.base_delay * 2_u32.pow((attempt - 1) as u32),
                    self.max_delay,
                );
                debug!(attempt = attempt, delay_ms = delay.as_millis(), "Retrying after delay");
                sleep(delay).await;
            }

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    debug!(attempt = attempt, error = ?e, "Operation failed, preparing retry");
                    last_error = Some(e);
                }
            }
        }

        error!(
            max_retries = self.max_retries,
            "All retry attempts exhausted"
        );
        Err(last_error.unwrap())
    }
}

/// Rate limiter for API calls
pub struct RateLimiter {
    calls: std::sync::Mutex<Vec<Instant>>,
    window_duration: Duration,
    max_calls: usize,
}

impl RateLimiter {
    pub fn new(max_calls: usize, window_duration: Duration) -> Self {
        Self {
            calls: std::sync::Mutex::new(Vec::new()),
            window_duration,
            max_calls,
        }
    }

    /// Check if call is allowed within rate limit
    pub fn allow_call(&self) -> bool {
        let now = Instant::now();
        
        if let Ok(mut calls) = self.calls.lock() {
            // Remove old calls outside the window
            calls.retain(|&call_time| now.duration_since(call_time) < self.window_duration);
            
            if calls.len() < self.max_calls {
                calls.push(now);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get remaining calls in current window
    pub fn remaining_calls(&self) -> usize {
        let now = Instant::now();
        
        if let Ok(mut calls) = self.calls.lock() {
            calls.retain(|&call_time| now.duration_since(call_time) < self.window_duration);
            self.max_calls.saturating_sub(calls.len())
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_circuit_breaker() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(100));
        
        // Initially should allow calls
        assert!(cb.can_execute());
        
        // Record failures
        for _ in 0..3 {
            cb.record_failure();
        }
        
        // Should be open now
        assert!(!cb.can_execute());
        
        // Wait for timeout
        sleep(Duration::from_millis(150)).await;
        
        // Should allow calls again
        assert!(cb.can_execute());
        
        // Success should reset
        cb.record_success();
        assert!(cb.can_execute());
    }

    #[tokio::test]
    async fn test_retry_policy() {
        let policy = RetryPolicy::new(3, Duration::from_millis(10), Duration::from_millis(100));
        
        let mut attempt_count = 0;
        let result = policy.retry_with_backoff(|| {
            attempt_count += 1;
            Box::pin(async move {
                if attempt_count < 3 {
                    Err("temporary failure")
                } else {
                    Ok("success")
                }
            })
        }).await;
        
        assert_eq!(result, Ok("success"));
        assert_eq!(attempt_count, 3);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, Duration::from_millis(100));
        
        assert!(limiter.allow_call());
        assert!(limiter.allow_call());
        assert!(!limiter.allow_call()); // Should be rate limited
        
        sleep(Duration::from_millis(150)).await;
        assert!(limiter.allow_call()); // Should allow after window
    }
}