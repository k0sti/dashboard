mod agent;
mod config;
mod plan;
mod storage;
mod toolcall;
mod tts;
mod ui;

use anyhow::Result;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use ui::DashboardApp;

pub struct TestMode {
    pub enabled: bool,
    pub timeout_secs: u64,
    pub exit_message: Option<String>,
    pub start_time: Instant,
    pub log_buffer: Arc<Mutex<Vec<String>>>,
}

impl TestMode {
    fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();
        let enabled = args.contains(&"--test".to_string());

        let timeout_secs = args.iter()
            .position(|arg| arg == "--timeout")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);

        let exit_message = args.iter()
            .position(|arg| arg == "--exit-on")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.to_string());

        Self {
            enabled,
            timeout_secs,
            exit_message,
            start_time: Instant::now(),
            log_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn should_exit(&self) -> bool {
        if !self.enabled {
            return false;
        }

        // Check timeout
        if self.start_time.elapsed() > Duration::from_secs(self.timeout_secs) {
            println!("[TEST] Timeout reached after {} seconds", self.timeout_secs);
            return true;
        }

        // Check exit message
        if let Some(ref exit_msg) = self.exit_message {
            let buffer = self.log_buffer.lock().unwrap();
            for line in buffer.iter() {
                if line.contains(exit_msg) {
                    println!("[TEST] Found exit message: '{}'", exit_msg);
                    return true;
                }
            }
        }

        false
    }

    fn log(&self, message: String) {
        if self.enabled {
            println!("{}", message);
            self.log_buffer.lock().unwrap().push(message);
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let test_mode = TestMode::from_args();

    if test_mode.enabled {
        println!("[TEST] Test mode enabled");
        println!("[TEST] Timeout: {} seconds", test_mode.timeout_secs);
        if let Some(ref msg) = test_mode.exit_message {
            println!("[TEST] Will exit on message: '{}'", msg);
        }
    }

    let test_mode = Arc::new(Mutex::new(test_mode));
    let test_mode_clone = Arc::clone(&test_mode);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Agent Dashboard",
        native_options,
        Box::new(move |cc| {
            let mut app = DashboardApp::new(cc);
            app.test_mode = Some(test_mode_clone);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {}", e))?;

    Ok(())
}
