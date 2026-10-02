use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub fn clean_download_url(raw: &str) -> Option<String> {
    let mut url = raw.trim();
    if url.starts_with("aurora://") {
        url = &url["aurora://".len()..];
    } else if url.starts_with("auroradl://") {
        url = &url["auroradl://".len()..];
    }
    
    // Percent-decode if browser encoded URL
    let decoded_opt = url::form_urlencoded::parse(url.as_bytes())
        .map(|(k, _)| k.to_string())
        .next();

    if let Some(dec) = decoded_opt {
        if dec.starts_with("http://") || dec.starts_with("https://") {
            return Some(dec);
        }
    }

    if url.starts_with("http://") || url.starts_with("https://") {
        Some(url.to_string())
    } else {
        None
    }
}

const IPC_PORT: u16 = 41419;

/// Check if another instance is already running.
/// If running, pass CLI arguments and return `true` (caller should exit).
/// If not running, return `false` (this process becomes primary).
pub async fn send_to_existing_instance_if_running(args: &[String]) -> bool {
    if let Ok(mut stream) = tokio::net::TcpStream::connect(("127.0.0.1", IPC_PORT)).await {
        let payload = serde_json::to_string(args).unwrap_or_default();
        let _ = stream.write_all(payload.as_bytes()).await;
        let _ = stream.flush().await;
        return true;
    }
    false
}

/// Start background listener for new instances attempting to pass URLs
pub fn start_single_instance_listener(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Ok(listener) = tokio::net::TcpListener::bind(("127.0.0.1", IPC_PORT)).await {
            while let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                loop {
                    match stream.read(&mut chunk).await {
                        Ok(0) => break,
                        Ok(n) => {
                            buf.extend_from_slice(&chunk[..n]);
                            if n < chunk.len() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                if !buf.is_empty() {
                    if let Ok(args) = serde_json::from_slice::<Vec<String>>(&buf) {
                        handle_incoming_args(&app_handle, &args);
                    }
                }
            }
        }
    });
}

pub fn handle_incoming_args(app: &AppHandle, args: &[String]) {
    // Focus or show main window
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }

    // Look for URLs in arguments
    for arg in args {
        if let Some(url) = clean_download_url(arg) {
            let payload = serde_json::json!({
                "url": url
            });
            let _ = app.emit("external-download-url", payload);
            break;
        }
    }
}
