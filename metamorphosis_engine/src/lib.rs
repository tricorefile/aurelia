use common::{AppEvent, EventSender};
use std::fs;
use std::process::Command;
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

const STRATEGY_ENGINE_SOURCE_PATH: &str = "strategy_engine/src/lib.rs";
#[cfg(target_os = "linux")]
const STRATEGY_ENGINE_LIB_PATH: &str = "target/release/libstrategy_engine.so";
#[cfg(target_os = "macos")]
const STRATEGY_ENGINE_LIB_PATH: &str = "target/release/libstrategy_engine.dylib";
#[cfg(target_os = "windows")]
const STRATEGY_ENGINE_LIB_PATH: &str = "target/release/strategy_engine.dll";

pub struct MetamorphosisEngine {
    tx: EventSender,
}

impl MetamorphosisEngine {
    pub fn new(tx: EventSender) -> Self {
        Self { tx }
    }

    pub async fn run(&mut self) {
        info!("[Metamorphosis Engine] Starting self-evolution loop...");
        // For this demo, we'll only try to evolve once, 30 seconds after startup.
        time::sleep(Duration::from_secs(30)).await;
        self.evolve().await;
    }

    async fn evolve(&self) {
        info!("[Metamorphosis Engine] Waking up to consider evolution...");

        // 1. Read the source code
        let source_code = match fs::read_to_string(STRATEGY_ENGINE_SOURCE_PATH) {
            Ok(code) => code,
            Err(e) => {
                error!("Failed to read strategy engine source: {}", e);
                return;
            }
        };

        // 2. Perform a simple, targeted modification
        let new_code = source_code.replace("Duration::from_secs(60)", "Duration::from_secs(30)");
        if new_code == source_code {
            info!("[Metamorphosis Engine] No changes to make. Already evolved.");
            return;
        }

        // 3. Write the new code back
        if let Err(e) = fs::write(STRATEGY_ENGINE_SOURCE_PATH, new_code) {
            error!("Failed to write new strategy engine source: {}", e);
            return;
        }
        info!("[Metamorphosis Engine] Source code modified. Recompiling...");

        // 4. Recompile the crate
        let output = Command::new("cargo")
            .args(["build", "-p", "strategy_engine", "--release"])
            .output()
            .expect("Failed to execute cargo build");

        if !output.status.success() {
            error!(
                "Failed to recompile strategy engine: {}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            // Optional: revert the source code change here
            return;
        }

        info!("Recompilation successful. Notifying kernel for hot-swap.");

        // 5. Notify the kernel
        let event = AppEvent::ModuleReadyForHotSwap(STRATEGY_ENGINE_LIB_PATH.to_string());
        if let Err(e) = self.tx.send(event) {
            error!("Failed to send ModuleReadyForHotSwap event: {}", e);
        }
    }
}
