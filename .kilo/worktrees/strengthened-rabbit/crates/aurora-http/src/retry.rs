use aurora_core::error::HttpError;
use rand::Rng;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDecision {
    /// Error is retryable after specified backoff duration.
    RetryAfter(Duration),
    /// Error is permanent and should not be retried.
    DoNotRetry,
}

pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_backoff: Duration,
    pub max_backoff: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(30),
        }
    }
}

impl RetryPolicy {
    pub fn new(max_attempts: u32, base_backoff: Duration, max_backoff: Duration) -> Self {
        Self {
            max_attempts,
            base_backoff,
            max_backoff,
        }
    }

    /// Determines whether an error should be retried and with what backoff.
    pub fn evaluate(&self, attempt: u32, error: &HttpError) -> RetryDecision {
        if attempt >= self.max_attempts {
            return RetryDecision::DoNotRetry;
        }

        match error {
            HttpError::HttpStatus {
                status,
                retry_after,
                ..
            } => match *status {
                // Rate limited: respect explicit Retry-After if present
                429 => {
                    let delay = retry_after.unwrap_or_else(|| self.calculate_backoff(attempt));
                    RetryDecision::RetryAfter(delay.min(self.max_backoff))
                }
                // Server errors: retryable
                500 | 502 | 503 | 504 => {
                    let delay = retry_after.unwrap_or_else(|| self.calculate_backoff(attempt));
                    RetryDecision::RetryAfter(delay)
                }
                // Client errors: generally permanent (400, 401, 403, 404, 410, etc.)
                _ => RetryDecision::DoNotRetry,
            },
            // Network failures & timeouts are retryable
            HttpError::Network(_) | HttpError::Timeout => {
                RetryDecision::RetryAfter(self.calculate_backoff(attempt))
            }
            // Protocol violation or bad header: do not retry blindly
            HttpError::InvalidHeader(_) | HttpError::RedirectLoop(_) => RetryDecision::DoNotRetry,
        }
    }

    /// Computes full-jitter exponential backoff: Uniform(0, min(max_backoff, base * 2^attempt))
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        let exp = 2u64.saturating_pow(attempt);
        let max_ms = (self.base_backoff.as_millis() as u64)
            .saturating_mul(exp)
            .min(self.max_backoff.as_millis() as u64);

        let mut rng = rand::thread_rng();
        let jitter_ms = if max_ms > 0 {
            rng.gen_range(0..=max_ms)
        } else {
            0
        };

        Duration::from_millis(jitter_ms)
    }
}
