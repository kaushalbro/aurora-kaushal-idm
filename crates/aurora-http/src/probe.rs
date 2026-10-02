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

            let suggested_filename = extract_suggested_filename(headers, mime_type.as_deref(), &final_url);

            let is_resumable = etag.is_some() || last_modified.is_some();
            let (server_name, health_rating) = extract_server_info(headers, rtt_ms, accepts_ranges);

            if accepts_ranges && content_length.is_some() {
                info!(
                    "HEAD probe successful: size={:?}, ranges=true, version={}, rtt={}ms, server={:?}, filename={:?}",
                    content_length, http_version, rtt_ms, server_name, suggested_filename
                );
                return Ok(ServerCapabilities {
                    accepts_ranges: true,
                    content_length,
                    etag,
                    last_modified,
                    mime_type,
                    suggested_filename,
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

    let suggested_filename = extract_suggested_filename(headers, mime_type.as_deref(), &final_url);

    // 206 Partial Content indicates Range is supported
    if status == StatusCode::PARTIAL_CONTENT {
        let content_range = headers
            .get(header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok());

        let total_size = content_range.and_then(parse_content_range_total);
        let (server_name, health_rating) = extract_server_info(headers, rtt_ms, true);

        info!(
            "Range GET probe confirmed: total_size={:?}, version={}, rtt={}ms, server={:?}, filename={:?}",
            total_size, http_version, rtt_ms, server_name, suggested_filename
        );

        return Ok(ServerCapabilities {
            accepts_ranges: true,
            content_length: total_size,
            etag,
            last_modified,
            mime_type,
            suggested_filename,
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
            "Server returned 200 OK (no range support). Single-stream download only. size={:?}, filename={:?}",
            content_length, suggested_filename
        );

        return Ok(ServerCapabilities {
            accepts_ranges: false,
            content_length,
            etag,
            last_modified,
            mime_type,
            suggested_filename,
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

/// Extracts the most accurate suggested filename from:
/// 1. Content-Disposition header (RFC 5987 filename*=UTF-8''... or standard filename="...")
/// 2. MIME type inference if filename is dynamic (.php, .aspx, or missing extension)
/// 3. URL path fallback
fn extract_suggested_filename(
    headers: &header::HeaderMap,
    mime_type: Option<&str>,
    url: &Url,
) -> Option<String> {
    // 1. Try Content-Disposition
    if let Some(cd_val) = headers.get(header::CONTENT_DISPOSITION).and_then(|v| v.to_str().ok()) {
        if let Some(name) = parse_content_disposition_filename(cd_val) {
            let sanitized = sanitize_filename(&name);
            if !sanitized.is_empty() {
                // Check if sanitized filename lacks extension but mime is known
                if !sanitized.contains('.') {
                    if let Some(mime) = mime_type {
                        if let Some(ext) = infer_extension_from_mime(mime) {
                            return Some(format!("{}.{}", sanitized, ext));
                        }
                    }
                }
                return Some(sanitized);
            }
        }
    }

    // 2. Extract from URL path
    let url_filename = url
        .path_segments()
        .and_then(|segments| segments.filter(|s| !s.is_empty()).last())
        .map(|s| s.split('?').next().unwrap_or(s))
        .map(|s| s.split('#').next().unwrap_or(s))
        .map(sanitize_filename);

    let raw_name = url_filename.unwrap_or_else(|| "download".to_string());

    // 3. If filename is a server-side script (.php, .aspx, .jsp, etc.) or has no extension,
    // and we have a valid MIME type, replace or append the correct extension.
    let is_script_or_generic = raw_name.ends_with(".php")
        || raw_name.ends_with(".aspx")
        || raw_name.ends_with(".asp")
        || raw_name.ends_with(".jsp")
        || raw_name.ends_with(".cgi")
        || raw_name.ends_with(".do")
        || raw_name.ends_with(".action")
        || !raw_name.contains('.');

    if is_script_or_generic {
        if let Some(mime) = mime_type {
            if let Some(ext) = infer_extension_from_mime(mime) {
                let stem = if let Some(dot_idx) = raw_name.rfind('.') {
                    &raw_name[..dot_idx]
                } else {
                    &raw_name
                };
                let clean_stem = if stem.is_empty() {
                    "download"
                } else {
                    stem
                };
                return Some(format!("{}.{}", clean_stem, ext));
            }
        }
    }

    if !raw_name.is_empty() && raw_name != "download" {
        Some(raw_name)
    } else if let Some(mime) = mime_type {
        infer_extension_from_mime(mime).map(|ext| format!("download.{}", ext))
    } else {
        Some(raw_name)
    }
}

fn parse_content_disposition_filename(cd: &str) -> Option<String> {
    // Check for RFC 5987 / RFC 6266 filename*=UTF-8''<encoded>
    for part in cd.split(';') {
        let part = part.trim();
        if let Some(stripped) = part.strip_prefix("filename*=") {
            let val = stripped.trim_matches('"').trim();
            if let Some(idx) = val.find("''") {
                let encoded = &val[idx + 2..];
                if let Ok(decoded) = percent_encoding_decode(encoded) {
                    return Some(decoded);
                }
            } else if let Ok(decoded) = percent_encoding_decode(val) {
                return Some(decoded);
            }
        }
    }

    // Check for standard filename="..." or filename=...
    for part in cd.split(';') {
        let part = part.trim();
        if let Some(stripped) = part.strip_prefix("filename=") {
            let val = stripped.trim_matches('"').trim_matches('\'').trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }

    None
}

fn percent_encoding_decode(input: &str) -> std::result::Result<String, ()> {
    let mut bytes = Vec::new();
    let mut chars = input.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let h1 = chars.next().ok_or(())?;
            let h2 = chars.next().ok_or(())?;
            let hex_arr = [h1, h2];
            let hex_str = std::str::from_utf8(&hex_arr).map_err(|_| ())?;
            let byte = u8::from_str_radix(hex_str, 16).map_err(|_| ())?;
            bytes.push(byte);
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8(bytes).map_err(|_| ())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0'..='\x1f' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim_matches('.')
        .trim()
        .to_string()
}

pub fn infer_extension_from_mime(mime: &str) -> Option<&'static str> {
    let clean = mime.split(';').next().unwrap_or(mime).trim().to_lowercase();
    match clean.as_str() {
        "application/pdf" => Some("pdf"),
        "application/zip" | "application/x-zip-compressed" => Some("zip"),
        "application/x-rar-compressed" | "application/vnd.rar" => Some("rar"),
        "application/x-7z-compressed" => Some("7z"),
        "application/x-tar" => Some("tar"),
        "application/gzip" | "application/x-gzip" => Some("tar.gz"),
        "application/x-bzip2" => Some("bz2"),
        "application/x-xz" => Some("xz"),
        "application/x-iso9660-image" => Some("iso"),
        "application/x-msdownload" | "application/vnd.microsoft.portable-executable" => Some("exe"),
        "application/x-apple-diskimage" => Some("dmg"),
        "application/vnd.android.package-archive" => Some("apk"),
        "application/x-debian-package" => Some("deb"),
        "application/x-redhat-package-manager" => Some("rpm"),
        "application/x-msi" => Some("msi"),
        "video/mp4" => Some("mp4"),
        "video/x-matroska" => Some("mkv"),
        "video/webm" => Some("webm"),
        "video/quicktime" => Some("mov"),
        "video/x-msvideo" => Some("avi"),
        "audio/mpeg" => Some("mp3"),
        "audio/flac" => Some("flac"),
        "audio/wav" | "audio/x-wav" => Some("wav"),
        "audio/ogg" => Some("ogg"),
        "audio/aac" => Some("aac"),
        "image/png" => Some("png"),
        "image/jpeg" => Some("jpg"),
        "image/webp" => Some("webp"),
        "image/gif" => Some("gif"),
        "image/svg+xml" => Some("svg"),
        "text/plain" => Some("txt"),
        "text/html" => Some("html"),
        "text/csv" => Some("csv"),
        "application/json" => Some("json"),
        "application/xml" | "text/xml" => Some("xml"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => Some("xlsx"),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => Some("docx"),
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => Some("pptx"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderMap;

    #[test]
    fn test_extract_from_content_disposition_standard() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"invoice_2026.pdf\"".parse().unwrap(),
        );
        let url = Url::parse("https://example.com/download.php?id=123").unwrap();
        let name = extract_suggested_filename(&headers, Some("application/pdf"), &url);
        assert_eq!(name, Some("invoice_2026.pdf".to_string()));
    }

    #[test]
    fn test_extract_from_content_disposition_rfc5987() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_DISPOSITION,
            "attachment; filename*=UTF-8''my%20document%202026.pdf".parse().unwrap(),
        );
        let url = Url::parse("https://example.com/bin/get_doc.php").unwrap();
        let name = extract_suggested_filename(&headers, Some("application/pdf"), &url);
        assert_eq!(name, Some("my document 2026.pdf".to_string()));
    }

    #[test]
    fn test_dynamic_url_with_mime_inference() {
        let headers = HeaderMap::new();
        let url = Url::parse("https://www.netim.com/bin/get_avoir_pdf.php?FACT=986687&CODE=n092kbr979t9ukd&LANG=EN").unwrap();
        let name = extract_suggested_filename(&headers, Some("application/pdf"), &url);
        assert_eq!(name, Some("get_avoir_pdf.pdf".to_string()));
    }

    #[test]
    fn test_generic_endpoint_with_mime() {
        let headers = HeaderMap::new();
        let url = Url::parse("https://example.com/download").unwrap();
        let name = extract_suggested_filename(&headers, Some("application/zip"), &url);
        assert_eq!(name, Some("download.zip".to_string()));
    }
}
