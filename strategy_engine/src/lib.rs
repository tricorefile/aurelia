use common::AppEvent;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time;
use tracing::{error, info};

const OUTPUT_FILE: &str = "strategy_output.log";

#[no_mangle]
pub extern "C" fn run_strategy_engine() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut engine = StrategyEngine::new();
        engine.run().await;
    });
}

pub struct StrategyEngine {}

impl Default for StrategyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyEngine {
    pub fn new() -> Self {
        // Clear the output file on start
        let _ = std::fs::remove_file(OUTPUT_FILE);
        Self {}
    }

    pub async fn run(&mut self) {
        info!("[Strategy Engine DLL] Starting...");
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            self.reason().await;
        }
    }

    async fn reason(&self) {
        info!("[Strategy Engine] Waking up to analyze market...");
        let event = AppEvent::WebSearchQuery("bitcoin price analysis".to_string());
        self.send_event_to_kernel(event);
    }

    fn send_event_to_kernel(&self, event: AppEvent) {
        let json = serde_json::to_string(&event).unwrap();
        match OpenOptions::new()
            .append(true)
            .create(true)
            .open(OUTPUT_FILE)
        {
            Ok(mut file) => {
                if let Err(e) = writeln!(file, "{}", json) {
                    error!("Failed to write to output file: {}", e);
                }
            }
            Err(e) => error!("Failed to open output file: {}", e),
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn process_event_from_kernel(event_json: *const std::os::raw::c_char) {
    let c_str = std::ffi::CStr::from_ptr(event_json);
    if let Ok(json_str) = c_str.to_str() {
        if let Ok(event) = serde_json::from_str::<AppEvent>(json_str) {
            // In a real implementation, you'd send this to the engine's main task via an internal channel.
            info!(
                "[Strategy Engine DLL] Received event from kernel: {:?}",
                event
            );
        }
    }
}
