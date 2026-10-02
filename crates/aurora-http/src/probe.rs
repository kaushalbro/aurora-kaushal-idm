use aurora_core::error::{HttpError, Result};
use aurora_core::types::ServerCapabilities;
use reqwest::{header, Client, StatusCode};
use tracing::{debug, info, warn};
use url::Url;

/// Probes a remote URL to discover capabilities, total file size, range support, RTT, and server health.
pub async fn probe_url(client: &Client, url: &Url) -> Result<ServerCapabilities> {
    debug!("Probing URL capabilities for: {}", url);

    let probe_start = std::time::Instant::now();

    // Step 1: Attempt standard HEAD request
    let head_response = client.head(url.as_str()).send().await;

    match head_response {
        Ok(res) if res.status().is_success() => {
            let rtt_ms = probe_start.elapsed().as_millis() as u64;
            let final_url = res.url().clone();
            let http_version = format!("{:?}", res.version());
            let headers = res.headers();

            let accepts_ranges = headers
                .get(header::ACCEPT_RANGES)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.eq_ignore_ascii_case("bytes"))
                .unwrap_or(false);

            let content_length = headers
                .get(header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());

            let etag = headers
                .get(header::ETAG)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let last_modified = headers
                .get(header::LAST_MODIFIED)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let mime_type = headers
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let is_resumable = etag.is_some() || last_modified.is_some();
            let (server_name, health_rating) = extract_server_info(headers, rtt_ms, accepts_ranges);

            if accepts_ranges && content_length.is_some() {
                info!(
                    "HEAD probe successful: size={:?}, ranges=true, version={}, rtt={}ms, server={:?}",
                    content_length, http_version, rtt_ms, server_name
                );
                return Ok(ServerCapabilities {
                    accepts_ranges: true,
                    content_length,
                    etag,
                    last_modified,
                    mime_type,
                    final_url,
                    http_version,
                    is_resumable,
                    rtt_ms: Some(rtt_ms),
                    server_name,
                    health_rating: Some(health_rating),
                });
            }

            debug!("HEAD did not confirm ranges. Probing with Range: bytes=0-0 GET");
        }
        Ok(res) => {
            warn!(
                "HEAD returned status {}, falling back to Range GET probe",
                res.status()
            );
        }
        Err(e) => {
            warn!(
                "HEAD request failed ({:?}), falling back to Range GET probe",
                e
            );
        }
    }

    // Step 2: Fallback probe using GET with Range: bytes=0-0
    probe_with_range_get(client, url).await
}

async fn probe_with_range_get(client: &Client, url: &Url) -> Result<ServerCapabilities> {
    let probe_start = std::time::Instant::now();
    let res = client
        .get(url.as_str())
        .header(header::RANGE, "bytes=0-0")
        .send()
        .await
        .map_err(|e| HttpError::Network(format!("Range GET probe failed: {}", e)))?;

    let rtt_ms = probe_start.elapsed().as_millis() as u64;
    let status = res.status();
    let final_url = res.url().clone();
    let http_version = format!("{:?}", res.version());
    let headers = res.headers();

    let etag = headers
        .get(header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let last_modified = headers
        .get(header::LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mime_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let is_resumable = etag.is_some() || last_modified.is_some();

    // 206 Partial Content indicates Range is supported
    if status == StatusCode::PARTIAL_CONTENT {
        let content_range = headers
            .get(header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok());

        let total_size = content_range.and_then(parse_content_range_total);
        let (server_name, health_rating) = extract_server_info(headers, rtt_ms, true);

        info!(
            "Range GET probe confirmed: total_size={:?}, version={}, rtt={}ms, server={:?}",
            total_size, http_version, rtt_ms, server_name
        );

        return Ok(ServerCapabilities {
            accepts_ranges: true,
            content_length: total_size,
            etag,
            last_modified,
            mime_type,
            final_url,
            http_version,
            is_resumable,
            rtt_ms: Some(rtt_ms),
            server_name,
            health_rating: Some(health_rating),
        });
    }

    // 200 OK means server ignored range header, fallback to single stream
    if status.is_success() {
        let content_length = headers
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let (server_name, health_rating) = extract_server_info(headers, rtt_ms, false);

        info!(
            "Server returned 200 OK (no range support). Single-stream download only. size={:?}",
            content_length
        );

        return Ok(ServerCapabilities {
            accepts_ranges: false,
            content_length,
            etag,
            last_modified,
            mime_type,
            final_url,
            http_version,
            is_resumable: false,
            rtt_ms: Some(rtt_ms),
            server_name,
            health_rating: Some(health_rating),
        });
    }

    Err(HttpError::HttpStatus {
        status: status.as_u16(),
        message: format!("HTTP probe failed with status {}", status),
        retry_after: None,
    }
    .into())
}

/// Identifies server software / CDN provider and rates latency bottleneck.
fn extract_server_info(headers: &header::HeaderMap, rtt_ms: u64, accepts_ranges: bool) -> (Option<String>, String) {
    let mut server = headers
        .get(header::SERVER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if headers.get("cf-ray").is_some() || server.as_deref().unwrap_or("").to_lowercase().contains("cloudflare") {
        server = Some("Cloudflare CDN".to_string());
    } else if headers.get("x-amz-request-id").is_some() {
        server = Some("Amazon S3".to_string());
    } else if headers.get("x-fastly-request-id").is_some() {
        server = Some("Fastly CDN".to_string());
    } else if headers.get("x-akamai-transformed").is_some() {
        server = Some("Akamai CDN".to_string());
    }

    let server_name = server.unwrap_or_else(|| "Web Server".to_string());

    let health = if !accepts_ranges {
        format!("⚠ Single-Stream ({}ms)", rtt_ms)
    } else if rtt_ms < 45 {
        format!("⚡ Local CDN ({}ms)", rtt_ms)
    } else if rtt_ms < 120 {
        format!("🚀 Fast Server ({}ms)", rtt_ms)
    } else {
        format!("🌐 High RTT ({}ms)", rtt_ms)
    };

    (Some(server_name), health)
}

/// Parses "bytes 0-0/123456" -> Some(123456)
fn parse_content_range_total(content_range: &str) -> Option<u64> {
    let parts: Vec<&str> = content_range.split('/').collect();
    if parts.len() == 2 {
        parts[1].trim().parse::<u64>().ok()
    } else {
        None
    }
}
