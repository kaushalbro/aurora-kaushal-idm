// Suppress console window on Windows GUI releases
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
pub mod server;
pub mod sorting;
pub mod theme;
mod views {
    pub mod batch_view;
    pub mod detail_view;
    pub mod graph_view;
    pub mod queue_view;
    pub mod segment_view;
    pub mod settings_view;
    pub mod sidebar_view;
}

use app::AuroraApp;
use eframe::egui;
use tracing_subscriber::EnvFilter;

fn load_app_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../resources/icon-128.png");
    if let Ok(image) = image::load_from_memory(icon_bytes) {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("aurora=info,info")),
        )
        .init();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([980.0, 620.0])
        .with_min_inner_size([720.0, 440.0])
        .with_title("AURORA Kaushal Download Manager");

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "AURORA Kaushal Download Manager",
        native_options,
        Box::new(|cc| {
            theme::configure_visuals(&cc.egui_ctx);
            Ok(Box::new(AuroraApp::new(cc)))
        }),
    )
}
