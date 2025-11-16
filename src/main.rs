mod agent;
mod config;
mod plan;
mod storage;
mod toolcall;
mod tts;
mod ui;

use anyhow::Result;
use ui::DashboardApp;

fn main() -> Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Agent Dashboard",
        native_options,
        Box::new(|cc| Ok(Box::new(DashboardApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {}", e))?;

    Ok(())
}
