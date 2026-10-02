use aurora_history::{DownloadRecord, HistoryDatabase};
use axum::{
    extract::{Json, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

#[derive(Clone)]
pub struct ServerState {
    pub history_db: HistoryDatabase,
    pub command_tx: mpsc::Sender<DesktopBridgeCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DesktopBridgeCommand {
    StartDownload {
        url: String,
        filename: Option<String>,
        connections: Option<usize>,
    },
    FocusWindow,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub app: String,
    pub version: String,
    pub connected: bool,
}

#[derive(Deserialize)]
pub struct AddDownloadRequest {
    pub url: String,
    pub filename: Option<String>,
    pub connections: Option<usize>,
}

#[derive(Deserialize)]
pub struct SyncHistoryRequest {
    pub downloads: Vec<DownloadRecord>,
}

#[derive(Serialize)]
pub struct GenericResponse {
    pub success: bool,
    pub message: String,
}

pub fn get_default_db_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let mut p = PathBuf::from(home);
        p.push(".aurora");
        p.push("history.db");
        p
    } else {
        std::env::temp_dir().join("aurora_history.db")
    }
}

pub async fn start_desktop_server(
    command_tx: mpsc::Sender<DesktopBridgeCommand>,
) -> anyhow::Result<()> {
    let db_path = get_default_db_path();
    let history_db = HistoryDatabase::open(db_path)?;

    let state = ServerState {
        history_db,
        command_tx,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/status", get(status_handler))
        .route("/api/history", get(get_history_handler))
        .route("/api/history/sync", post(sync_history_handler))
        .route("/api/download", post(add_download_handler))
        .route("/api/open-app", post(open_app_handler))
        .route("/api/focus", post(open_app_handler))
        .layer(cors)
        .with_state(Arc::new(state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 28282));
    info!("AURORA Desktop Sync Bridge starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("Desktop sync bridge server error: {}", e);
        }
    });

    Ok(())
}

async fn status_handler() -> impl IntoResponse {
    Json(StatusResponse {
        status: "ok".to_string(),
        app: "AURORA Kaushal IDM".to_string(),
        version: "0.2.0".to_string(),
        connected: true,
    })
}

async fn get_history_handler(
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    match state.history_db.get_all_downloads() {
        Ok(downloads) => (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "downloads": downloads
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
            "downloads": []
        }))),
    }
}

async fn sync_history_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<SyncHistoryRequest>,
) -> impl IntoResponse {
    let mut synced_count = 0;
    for rec in payload.downloads {
        if state.history_db.insert_download(&rec).is_ok() {
            synced_count += 1;
        }
    }
    Json(GenericResponse {
        success: true,
        message: format!("Synced {} records to Desktop SQLite", synced_count),
    })
}

async fn add_download_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<AddDownloadRequest>,
) -> impl IntoResponse {
    let cmd = DesktopBridgeCommand::StartDownload {
        url: payload.url,
        filename: payload.filename,
        connections: payload.connections,
    };

    match state.command_tx.send(cmd).await {
        Ok(_) => (StatusCode::OK, Json(GenericResponse {
            success: true,
            message: "Download queued in AURORA Desktop Engine".to_string(),
        })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(GenericResponse {
            success: false,
            message: format!("Failed to dispatch download: {}", e),
        })),
    }
}

async fn open_app_handler(
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    let _ = state.command_tx.send(DesktopBridgeCommand::FocusWindow).await;
    (StatusCode::OK, Json(GenericResponse {
        success: true,
        message: "AURORA Desktop window focused".to_string(),
    }))
}

