use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Information required for deploying the agent to a new server.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeploymentInfo {
    pub ip: String,
    pub remote_user: String,
    pub private_key_path: String,
    pub remote_path: String, // e.g., "/home/user/aurelia"
}

/// The central message type for the entire application.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AppEvent {
    SystemVitals(SystemVitals),
    MarketData(MarketData),
    StrategyDecision(StrategyDecision),
    ReloadConfig,
    SystemStateChange(SystemState),
    FinancialUpdate(f64),
    WebSearchQuery(String),
    WebSearchResponse(Vec<String>),
    LlmQuery(String),
    LlmResponse(String),
    ModuleReadyForHotSwap(String),
    Deploy(DeploymentInfo),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum SystemState {
    Normal,
    Conservation,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StrategyDecision {
    Buy(String, f64),  // Symbol, Price
    Sell(String, f64), // Symbol, Price
    Hold(String),      // Symbol
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemVitals {
    pub cpu_usage: f32,
    pub mem_usage_mb: f64,
    pub mem_total_mb: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: u64,
}

/// A type alias for the broadcast sender.
pub type EventSender = broadcast::Sender<AppEvent>;

/// A type alias for the broadcast receiver.
pub type EventReceiver = broadcast::Receiver<AppEvent>;
