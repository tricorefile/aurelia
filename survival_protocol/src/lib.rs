use common::{AppEvent, EventReceiver, EventSender, SystemState};
use tracing::{error, info, warn};

use tokio::time::{self, Duration};

const SIMULATED_HOURLY_COST: f64 = 0.5; // e.g., $0.50 per hour
const MINIMUM_RUNWAY_HOURS: f64 = 24.0; // Require at least 24 hours of runway

pub struct SurvivalProtocol {
    tx: EventSender,
    rx: EventReceiver,
    current_funds: f64,
    current_state: SystemState,
}

impl SurvivalProtocol {
    pub fn new(tx: EventSender, rx: EventReceiver, initial_funds: f64) -> Self {
        Self {
            tx,
            rx,
            current_funds: initial_funds,
            current_state: SystemState::Normal,
        }
    }

    pub async fn run(&mut self) {
        info!("[Survival Protocol] Starting...");
        let mut health_check_interval = time::interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                _ = health_check_interval.tick() => {
                    self.check_runway().await;
                }
                Ok(event) = self.rx.recv() => {
                    match event {
                        AppEvent::FinancialUpdate(funds) => {
                            self.current_funds = funds;
                            self.check_runway().await;
                        }
                        _ => {}
                    }
                }
                else => { break; } // Channel closed
            }
        }
    }

    async fn check_runway(&mut self) {
        let runway_hours = self.current_funds / SIMULATED_HOURLY_COST;
        info!(
            funds = self.current_funds,
            runway_hours = runway_hours,
            "[Survival Protocol] Runway check."
        );

        if runway_hours < MINIMUM_RUNWAY_HOURS && self.current_state == SystemState::Normal {
            warn!("[Survival Protocol] Runway is below threshold! Entering CONSERVATION mode.");
            self.change_system_state(SystemState::Conservation).await;
        } else if runway_hours >= MINIMUM_RUNWAY_HOURS
            && self.current_state == SystemState::Conservation
        {
            info!("[Survival Protocol] Runway is healthy again. Returning to NORMAL mode.");
            self.change_system_state(SystemState::Normal).await;
        }
    }

    async fn change_system_state(&mut self, new_state: SystemState) {
        self.current_state = new_state.clone();
        if let Err(e) = self.tx.send(AppEvent::SystemStateChange(new_state)) {
            error!(
                "[Survival Protocol] Failed to send SystemStateChange event: {}",
                e
            );
        }
    }
}
