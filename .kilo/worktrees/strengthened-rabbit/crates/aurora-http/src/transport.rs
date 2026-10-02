use crate::client::{create_http_client, HttpClientConfig};
use crate::probe::probe_url;
use aurora_core::error::{HttpError, Result};
use aurora_core::traits::Transport;
use aurora_core::types::{ByteRange, ServerCapabilities};
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::{Client, Response, StatusCode};
use std::time::{Duration, Instant};
use tracing::{debug, trace};
use url::Url;

/// HTTP range streaming response handle with timing metadata.
pub struct RangeStream {
    pub response: Response,
    pub rtt: Duration,
    pub range: ByteRange,
}

/// HTTP/1.1 and HTTP/2 transport implementation.
#[derive(Clone)]
pub struct HttpTransport {
    client: Client,
}

impl HttpTransport {
    pub fn new(config: &HttpClientConfig) -> Result<Self> {
        let client = create_http_client(config)?;
        Ok(Self { client })
    }

    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    /// Fetches an HTTP range stream and measures RTT (Time to First Byte).
    pub async fn fetch_range(
        &self,
        url: &Url,
        range: ByteRange,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<RangeStream> {
        let mut headers = HeaderMap::new();
        let range_header = range.to_http_header();
        headers.insert(
            header::RANGE,
            HeaderValue::from_str(&range_header)
                .map_err(|e| HttpError::InvalidHeader(e.to_string()))?,
        );

        if let Some(tag) = etag {
            if let Ok(val) = HeaderValue::from_str(tag) {
                headers.insert(header::IF_RANGE, val);
            }
        } else if let Some(lm) = last_modified {
            if let Ok(val) = HeaderValue::from_str(lm) {
                headers.insert(header::IF_RANGE, val);
            }
        }

        trace!("Sending range request {} for {}", range_header, url);
        let start_time = Instant::now();
        let response = self
            .client
            .get(url.as_str())
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    HttpError::Timeout
                } else {
                    HttpError::Network(e.to_string())
                }
            })?;

        let rtt = start_time.elapsed();
        let status = response.status();

        if status == StatusCode::PARTIAL_CONTENT || (range.start == 0 && status == StatusCode::OK) {
            debug!("Range request {} succeeded (RTT: {:?})", range_header, rtt);
            Ok(RangeStream {
                response,
                rtt,
                range,
            })
        } else if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get(header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .map(Duration::from_secs);

            Err(HttpError::HttpStatus {
                status: 429,
                message: "Rate limit exceeded (429)".to_string(),
                retry_after,
            }
            .into())
        } else {
            Err(HttpError::HttpStatus {
                status: status.as_u16(),
                message: format!("Unexpected status {} for range request", status),
                retry_after: None,
            }
            .into())
        }
    }
}

#[async_trait::async_trait]
impl Transport for HttpTransport {
    async fn probe(&self, url: &Url) -> Result<ServerCapabilities> {
        probe_url(&self.client, url).await
    }
}
