mod app;
mod views {
    pub mod detail_view;
    pub mod queue_view;
    pub mod segment_view;
    pub mod settings_view;
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
            .with_inner_size([920.0, 580.0])
            .with_min_inner_size([650.0, 400.0])
            .with_title("AURORA Kaushal Download Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "AURORA Kaushal Download Manager",
        native_options,
        Box::new(|cc| {
            // Apply custom sleek dark theme styling
            let mut visuals = egui::Visuals::dark();
            visuals.window_rounding = 8.0.into();
            visuals.menu_rounding = 6.0.into();
            visuals.panel_fill = egui::Color32::from_rgb(22, 24, 30);
            visuals.override_text_color = Some(egui::Color32::from_rgb(225, 230, 240));
            cc.egui_ctx.set_visuals(visuals);

            Ok(Box::new(AuroraApp::new(cc)))
        }),
    )
}
