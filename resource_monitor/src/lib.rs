use common::{AppEvent, EventReceiver, EventSender, SystemState, SystemVitals};
use std::time::Duration;
use sysinfo::System;

use tracing::info;

pub async fn run(tx: EventSender, mut rx: EventReceiver) {
    let mut sys = System::new_all();
    let mut interval_duration = Duration::from_secs(5);

    loop {
        // Use a non-blocking recv to check for state changes without stopping the tick
        if let Ok(AppEvent::SystemStateChange(new_state)) = rx.try_recv() {
            info!("[Resource Monitor] Received new state: {:?}", new_state);
            interval_duration = match new_state {
                SystemState::Normal => Duration::from_secs(5),
                SystemState::Conservation => Duration::from_secs(30), // Slow down
            };
        }

        tokio::time::sleep(interval_duration).await;

        sys.refresh_cpu();
        sys.refresh_memory();

        let vitals = SystemVitals {
            cpu_usage: sys.global_cpu_info().cpu_usage(),
            mem_usage_mb: sys.used_memory() as f64 / 1_048_576.0,
            mem_total_mb: sys.total_memory() as f64 / 1_048_576.0,
        };

        if let Err(e) = tx.send(AppEvent::SystemVitals(vitals)) {
            eprintln!("[Resource Monitor] Failed to send vitals: {}", e);
        }
    }
}
