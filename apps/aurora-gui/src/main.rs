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

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("aurora=info,info")),
        )
        .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 600.0])
            .with_min_inner_size([700.0, 420.0])
            .with_title("AURORA Kaushal Download Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "AURORA Kaushal Download Manager",
        native_options,
        Box::new(|cc| {
            // Apply extension-matched sleek macOS / Modern Web dark visuals
            theme::configure_visuals(&cc.egui_ctx);
            Ok(Box::new(AuroraApp::new(cc)))
        }),
    )
}
