use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::head;
use axum::Router;
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tracing::info;

#[derive(Debug, Clone)]
pub struct MockServerConfig {
    pub file_size_bytes: usize,
    pub simulated_latency: Duration,
    pub rate_limit_bytes_per_sec: Option<usize>,
    pub inject_429_count: u32,
    pub inject_500_count: u32,
    pub change_etag_midway: bool,
    pub reject_ranges: bool,
    pub etag: String,
}

impl Default for MockServerConfig {
    fn default() -> Self {
        Self {
            file_size_bytes: 20 * 1024 * 1024, // 20 MB default
            simulated_latency: Duration::ZERO,
            rate_limit_bytes_per_sec: None,
            inject_429_count: 0,
            inject_500_count: 0,
            change_etag_midway: false,
            reject_ranges: false,
            etag: "\"aurora-test-etag-v1\"".to_string(),
        }
    }
}

#[derive(Clone)]
struct ServerState {
    config: MockServerConfig,
    data: Arc<Vec<u8>>,
    req_counter: Arc<AtomicU32>,
}

pub struct MockHttpServer {
    addr: SocketAddr,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MockHttpServer {
    /// Starts an in-process mock HTTP server on a random open localhost port.
    pub async fn start(config: MockServerConfig) -> (Self, String) {
        // Generate deterministic test pattern
        let mut data = Vec::with_capacity(config.file_size_bytes);
        for i in 0..config.file_size_bytes {
            data.push((i % 251) as u8);
        }

        let state = ServerState {
            config,
            data: Arc::new(data),
            req_counter: Arc::new(AtomicU32::new(0)),
        };

        let app = Router::new()
            .route("/testfile.bin", head(handle_head).get(handle_get))
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .unwrap();
        });

        let url = format!("http://{}:{}/testfile.bin", addr.ip(), addr.port());
        info!("Mock HTTP Test Server running at {}", url);

        (
            Self {
                addr,
                shutdown_tx: Some(shutdown_tx),
            },
            url,
        )
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }
}

impl Drop for MockHttpServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

async fn handle_head(State(state): State<ServerState>) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&state.data.len().to_string()).unwrap(),
    );
    headers.insert(
        header::ETAG,
        HeaderValue::from_str(&state.config.etag).unwrap(),
    );
    headers.insert(
        header::LAST_MODIFIED,
        HeaderValue::from_static("Thu, 01 Jan 2026 00:00:00 GMT"),
    );

    if !state.config.reject_ranges {
        headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    }

    (StatusCode::OK, headers).into_response()
}

async fn handle_get(State(state): State<ServerState>, headers: HeaderMap) -> Response {
    let req_num = state.req_counter.fetch_add(1, Ordering::SeqCst);

    // Fault injection: 429 Too Many Requests
    if req_num < state.config.inject_429_count {
        let mut h = HeaderMap::new();
        h.insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
        return (StatusCode::TOO_MANY_REQUESTS, h, "Rate limited").into_response();
    }

    // Fault injection: 500 Server Error
    if req_num < state.config.inject_429_count + state.config.inject_500_count {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Simulated Server Error").into_response();
    }

    // Simulated latency
    if state.config.simulated_latency > Duration::ZERO {
        tokio::time::sleep(state.config.simulated_latency).await;
    }

    let total_len = state.data.len();

    // Check for Range header
    if !state.config.reject_ranges {
        if let Some(range_val) = headers.get(header::RANGE).and_then(|v| v.to_str().ok()) {
            if let Some((start, end)) = parse_range_header(range_val, total_len) {
                let slice = state.data[start..=end].to_vec();
                let slice_len = slice.len();

                let mut resp_headers = HeaderMap::new();
                resp_headers.insert(
                    header::CONTENT_RANGE,
                    HeaderValue::from_str(&format!("bytes {}-{}/{}", start, end, total_len)).unwrap(),
                );
                resp_headers.insert(
                    header::CONTENT_LENGTH,
                    HeaderValue::from_str(&slice_len.to_string()).unwrap(),
                );
                resp_headers.insert(
                    header::ETAG,
                    HeaderValue::from_str(&state.config.etag).unwrap(),
                );

                if let Some(rate_limit) = state.config.rate_limit_bytes_per_sec {
                    // Return throttled streaming body
                    let stream = throttled_byte_stream(Bytes::from(slice), rate_limit);
                    return (StatusCode::PARTIAL_CONTENT, resp_headers, Body::from_stream(stream))
                        .into_response();
                } else {
                    return (StatusCode::PARTIAL_CONTENT, resp_headers, Body::from(slice))
                        .into_response();
                }
            }
        }
    }

    // Full file response
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&total_len.to_string()).unwrap(),
    );
    resp_headers.insert(
        header::ETAG,
        HeaderValue::from_str(&state.config.etag).unwrap(),
    );
    (StatusCode::OK, resp_headers, Body::from(state.data.as_ref().clone())).into_response()
}

fn parse_range_header(range_str: &str, total_len: usize) -> Option<(usize, usize)> {
    if !range_str.starts_with("bytes=") {
        return None;
    }
    let range_part = &range_str["bytes=".len()..];
    let parts: Vec<&str> = range_part.split('-').collect();
    if parts.len() != 2 {
        return None;
    }

    let start = parts[0].trim().parse::<usize>().ok()?;
    let end = if parts[1].trim().is_empty() {
        total_len.saturating_sub(1)
    } else {
        parts[1].trim().parse::<usize>().ok()?.min(total_len.saturating_sub(1))
    };

    if start <= end && start < total_len {
        Some((start, end))
    } else {
        None
    }
}

fn throttled_byte_stream(
    data: Bytes,
    bytes_per_sec: usize,
) -> impl futures_util::Stream<Item = std::result::Result<Bytes, std::io::Error>> {
    async_stream::stream! {
        let chunk_size = (bytes_per_sec / 10).max(4096);
        let chunk_delay = Duration::from_millis(100);

        let mut offset = 0;
        while offset < data.len() {
            let end = (offset + chunk_size).min(data.len());
            let chunk = data.slice(offset..end);
            offset = end;
            yield Ok(chunk);
            tokio::time::sleep(chunk_delay).await;
        }
    }
}
