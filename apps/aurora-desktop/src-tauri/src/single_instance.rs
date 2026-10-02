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

#[cfg(unix)]
fn get_socket_path() -> PathBuf {
    std::env::temp_dir().join("aurora_kaushal_idm.sock")
}

/// Check if another instance is already running.
/// If running, pass CLI arguments and return `true` (caller should exit).
/// If not running, return `false` (this process becomes primary).
pub async fn send_to_existing_instance_if_running(args: &[String]) -> bool {
    #[cfg(unix)]
    {
        use tokio::net::UnixStream;
        let socket_path = get_socket_path();
        if socket_path.exists() {
            if let Ok(mut stream) = UnixStream::connect(&socket_path).await {
                // An existing instance is listening!
                let payload = serde_json::to_string(args).unwrap_or_default();
                let _ = stream.write_all(payload.as_bytes()).await;
                let _ = stream.flush().await;
                return true;
            } else {
                // Stale socket file
                let _ = std::fs::remove_file(&socket_path);
            }
        }
    }
    false
}

/// Start background listener for new instances attempting to pass URLs
pub fn start_single_instance_listener(app_handle: AppHandle) {
    #[cfg(unix)]
    {
        use tokio::net::UnixListener;
        let socket_path = get_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        tauri::async_runtime::spawn(async move {
            if let Ok(listener) = UnixListener::bind(&socket_path) {
                while let Ok((mut stream, _)) = listener.accept().await {
                    let mut buf = Vec::new();
                    if let Ok(_) = stream.read_to_end(&mut buf).await {
                        if let Ok(args) = serde_json::from_slice::<Vec<String>>(&buf) {
                            handle_incoming_args(&app_handle, &args);
                        }
                    }
                }
            }
        });
    }
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
