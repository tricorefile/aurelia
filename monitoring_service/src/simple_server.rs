use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleAgentStatus {
    pub agent_id: String,
    pub hostname: String,
    pub status: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SimpleMonitoringService {
    pub agents: Arc<RwLock<HashMap<String, SimpleAgentStatus>>>,
    pub port: u16,
}

impl SimpleMonitoringService {
    pub fn new(port: u16) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            port,
        }
    }

    pub async fn start(self: Arc<Self>) -> anyhow::Result<()> {
        // Start background monitoring
        let monitor_service = self.clone();
        tokio::spawn(async move {
            monitor_service.monitor_loop().await;
        });

        // Simple HTTP server
        let addr = format!("0.0.0.0:{}", self.port);
        tracing::info!("Monitoring service starting on {}", addr);
        
        // For now, just log that we're ready
        // In a real implementation, we'd start an actual HTTP server here
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            let agents = self.agents.read().await;
            tracing::info!("Monitoring {} agents", agents.len());
        }
    }

    async fn monitor_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            // Collect local metrics
            let hostname = hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "localhost".to_string());
            
            let status = SimpleAgentStatus {
                agent_id: format!("agent-{}", hostname),
                hostname: hostname.clone(),
                status: "Running".to_string(),
                cpu_usage: 25.0, // Mock value
                memory_usage: 40.0, // Mock value
                last_heartbeat: Utc::now(),
            };
            
            let mut agents = self.agents.write().await;
            agents.insert(status.agent_id.clone(), status);
        }
    }
}