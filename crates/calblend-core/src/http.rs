//! HTTP client with retry logic and middleware support

use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use std::time::Duration;
use tracing::debug;

use crate::{CalblendConfig, CalblendError, Result};

/// HTTP client wrapper with retry logic
#[derive(Clone)]
pub struct HttpClient {
    client: ClientWithMiddleware,
}

impl HttpClient {
    /// Create a new HTTP client with retry middleware
    pub fn new(config: &CalblendConfig) -> Result<Self> {
        // Create the base reqwest client
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| CalblendError::InternalError(e.to_string()))?;

        // Create retry policy
        let retry_policy = ExponentialBackoff::builder()
            .build_with_max_retries(config.max_retries);

        // Wrap with middleware
        let client = ClientBuilder::new(client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(Self { client })
    }

    /// Get the underlying client for making requests
    pub fn client(&self) -> &ClientWithMiddleware {
        &self.client
    }
}

/// Rate limiter for API calls
pub struct RateLimiter {
    /// Maximum requests per time window
    max_requests: u32,
    /// Time window in seconds
    window_secs: u64,
    /// Current request count
    request_count: std::sync::atomic::AtomicU32,
    /// Window start time
    window_start: tokio::sync::Mutex<std::time::Instant>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_secs,
            request_count: std::sync::atomic::AtomicU32::new(0),
            window_start: tokio::sync::Mutex::new(std::time::Instant::now()),
        }
    }

    /// Check if we can make a request, blocking if necessary
    pub async fn check_rate_limit(&self) {
        loop {
            let now = std::time::Instant::now();
            let mut window_start = self.window_start.lock().await;
            
            // Reset window if expired
            if now.duration_since(*window_start).as_secs() >= self.window_secs {
                *window_start = now;
                self.request_count.store(0, std::sync::atomic::Ordering::Relaxed);
            }

            let count = self.request_count.load(std::sync::atomic::Ordering::Relaxed);
            if count < self.max_requests {
                self.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                break;
            }

            // Calculate sleep time
            let elapsed = now.duration_since(*window_start);
            let remaining = Duration::from_secs(self.window_secs) - elapsed;
            drop(window_start);

            debug!("Rate limit reached, sleeping for {:?}", remaining);
            tokio::time::sleep(remaining).await;
        }
    }
}

/// Convert Google API errors to CalblendError
pub fn map_google_error(status: reqwest::StatusCode, body: &str) -> CalblendError {
    match status {
        reqwest::StatusCode::UNAUTHORIZED => CalblendError::Authentication("Invalid or expired token".to_string()),
        reqwest::StatusCode::FORBIDDEN => CalblendError::PermissionDenied("Insufficient permissions".to_string()),
        reqwest::StatusCode::NOT_FOUND => CalblendError::EventNotFound("Resource not found".to_string()),
        reqwest::StatusCode::TOO_MANY_REQUESTS => CalblendError::RateLimitExceeded,
        _ => {
            // Try to parse error from response body
            if let Ok(error_response) = serde_json::from_str::<GoogleErrorResponse>(body) {
                CalblendError::Provider(
                    format!("Google: {}", error_response.error.message)
                )
            } else {
                CalblendError::Provider(
                    format!("Google: HTTP {} - {}", status.as_u16(), body)
                )
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct GoogleErrorResponse {
    error: GoogleError,
}

#[derive(Debug, serde::Deserialize)]
struct GoogleError {
    code: u16,
    message: String,
    status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, 1);

        // First two requests should go through immediately
        let start = std::time::Instant::now();
        limiter.check_rate_limit().await;
        limiter.check_rate_limit().await;
        assert!(start.elapsed().as_millis() < 100);

        // Third request should be delayed
        limiter.check_rate_limit().await;
        assert!(start.elapsed().as_secs() >= 1);
    }
}