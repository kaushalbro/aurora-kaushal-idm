use aurora_core::error::{HttpError, Result};
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub connect_timeout: Duration,
    pub read_timeout: Duration,
    pub pool_max_idle_per_host: usize,
    pub pool_idle_timeout: Duration,
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(30),
            pool_max_idle_per_host: 64,
            pool_idle_timeout: Duration::from_secs(90),
            user_agent: "AURORA/0.2.0 (Rust; High-Performance Engine)".to_string(),
        }
    }
}

/// Builds and configures an optimized HTTP/1.1 and HTTP/2 connection client pool.
pub fn create_http_client(config: &HttpClientConfig) -> Result<Client> {
    Client::builder()
        .user_agent(&config.user_agent)
        .connect_timeout(config.connect_timeout)
        .read_timeout(config.read_timeout)
        .pool_max_idle_per_host(config.pool_max_idle_per_host.max(128))
        .pool_idle_timeout(config.pool_idle_timeout)
        .tcp_nodelay(true)
        .tcp_keepalive(Some(Duration::from_secs(60)))
        .http2_adaptive_window(true)
        .http2_initial_stream_window_size(Some(8 * 1024 * 1024))
        .http2_initial_connection_window_size(Some(16 * 1024 * 1024))
        .http2_keep_alive_interval(Some(Duration::from_secs(15)))
        .http2_keep_alive_timeout(Duration::from_secs(30))
        .http2_keep_alive_while_idle(true)
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| HttpError::Network(format!("Failed to build HTTP client: {}", e)).into())
}
