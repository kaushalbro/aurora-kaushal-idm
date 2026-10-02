pub mod client;
pub mod probe;
pub mod retry;
pub mod transport;

pub use client::{create_http_client, HttpClientConfig};
pub use probe::probe_url;
pub use retry::{RetryDecision, RetryPolicy};
pub use transport::{HttpTransport, RangeStream};

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::error::HttpError;
    use std::time::Duration;

    #[test]
    fn test_retry_policy_transient_error() {
        let policy = RetryPolicy::new(3, Duration::from_millis(100), Duration::from_secs(5));
        let error = HttpError::Network("Connection reset".to_string());

        let decision = policy.evaluate(0, &error);
        match decision {
            RetryDecision::RetryAfter(delay) => {
                assert!(delay <= Duration::from_millis(200));
            }
            RetryDecision::DoNotRetry => panic!("Expected RetryAfter"),
        }
    }

    #[test]
    fn test_retry_policy_429_rate_limit() {
        let policy = RetryPolicy::default();
        let error = HttpError::HttpStatus {
            status: 429,
            message: "Too Many Requests".to_string(),
            retry_after: Some(Duration::from_secs(12)),
        };

        let decision = policy.evaluate(1, &error);
        match decision {
            RetryDecision::RetryAfter(delay) => {
                assert_eq!(delay, Duration::from_secs(12));
            }
            RetryDecision::DoNotRetry => panic!("Expected RetryAfter with specific delay"),
        }
    }

    #[test]
    fn test_retry_policy_permanent_404() {
        let policy = RetryPolicy::default();
        let error = HttpError::HttpStatus {
            status: 404,
            message: "Not Found".to_string(),
            retry_after: None,
        };

        let decision = policy.evaluate(0, &error);
        assert_eq!(decision, RetryDecision::DoNotRetry);
    }
}
