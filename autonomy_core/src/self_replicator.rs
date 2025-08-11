use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use deployment_tester::{DeploymentClient, ServerConfig, TestConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationTarget {
    pub ip: String,
    pub user: String,
    pub ssh_key_path: PathBuf,
    pub remote_path: PathBuf,
    pub priority: u8,
    pub last_attempt: Option<DateTime<Utc>>,
    pub success_count: u32,
    pub failure_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationResult {
    pub target: String,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub duration_seconds: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStrategy {
    pub max_replicas: usize,
    pub min_replicas: usize,
    pub replication_interval_seconds: u64,
    pub retry_attempts: u32,
    pub health_check_interval: u64,
    pub auto_scale: bool,
}

impl Default for ReplicationStrategy {
    fn default() -> Self {
        Self {
            max_replicas: 5,
            min_replicas: 2,
            replication_interval_seconds: 300,
            retry_attempts: 3,
            health_check_interval: 60,
            auto_scale: true,
        }
    }
}

pub struct SelfReplicator {
    strategy: ReplicationStrategy,
    targets: Arc<RwLock<Vec<ReplicationTarget>>>,
    active_replicas: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    replication_history: Arc<RwLock<Vec<ReplicationResult>>>,
    binary_path: PathBuf,
}

impl SelfReplicator {
    pub fn new(binary_path: PathBuf) -> Self {
        Self {
            strategy: ReplicationStrategy::default(),
            targets: Arc::new(RwLock::new(Vec::new())),
            active_replicas: Arc::new(RwLock::new(HashMap::new())),
            replication_history: Arc::new(RwLock::new(Vec::new())),
            binary_path,
        }
    }

    pub fn with_strategy(mut self, strategy: ReplicationStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub async fn add_target(&self, target: ReplicationTarget) {
        let mut targets = self.targets.write().await;
        targets.push(target);
        targets.sort_by_key(|t| t.priority);
    }

    pub async fn should_replicate(&self) -> bool {
        let active_count = self.active_replicas.read().await.len();
        
        if active_count < self.strategy.min_replicas {
            info!("Active replicas ({}) below minimum ({}), replication needed", 
                  active_count, self.strategy.min_replicas);
            return true;
        }

        if self.strategy.auto_scale && active_count < self.strategy.max_replicas {
            // Check system load and decide if scaling is needed
            if self.check_scaling_conditions().await {
                info!("Scaling conditions met, initiating replication");
                return true;
            }
        }

        false
    }

    async fn check_scaling_conditions(&self) -> bool {
        // In a real implementation, this would check:
        // - Current system load
        // - Market conditions
        // - Resource availability
        // - Cost considerations
        
        // For now, we'll use a simple random decision with 20% probability
        rand::random::<f64>() < 0.2
    }

    pub async fn replicate(&self) -> Result<Vec<ReplicationResult>> {
        info!("Starting autonomous self-replication process");
        
        let targets = self.targets.read().await.clone();
        let active_replicas = self.active_replicas.read().await.len();
        let mut results = Vec::new();

        let replicas_needed = self.strategy.min_replicas.saturating_sub(active_replicas);
        
        for target in targets.iter().take(replicas_needed) {
            if self.active_replicas.read().await.contains_key(&target.ip) {
                continue; // Skip already active replicas
            }

            let result = self.replicate_to_target(target).await;
            results.push(result.clone());

            if result.success {
                self.active_replicas.write().await.insert(
                    target.ip.clone(),
                    Utc::now(),
                );
                
                // Update target statistics
                let mut targets = self.targets.write().await;
                if let Some(t) = targets.iter_mut().find(|t| t.ip == target.ip) {
                    t.success_count += 1;
                    t.last_attempt = Some(Utc::now());
                }
            } else {
                // Update failure statistics
                let mut targets = self.targets.write().await;
                if let Some(t) = targets.iter_mut().find(|t| t.ip == target.ip) {
                    t.failure_count += 1;
                    t.last_attempt = Some(Utc::now());
                }
            }

            // Record in history
            self.replication_history.write().await.push(result);
        }

        Ok(results)
    }

    async fn replicate_to_target(&self, target: &ReplicationTarget) -> ReplicationResult {
        let start_time = Utc::now();
        info!("Attempting replication to {}", target.ip);

        let server_config = ServerConfig {
            name: format!("replica-{}", target.ip),
            ip: target.ip.clone(),
            port: 22,
            user: target.user.clone(),
            ssh_key_path: target.ssh_key_path.clone(),
            remote_deploy_path: target.remote_path.clone(),
            role: deployment_tester::config::ServerRole::Replica,
        };

        let client = DeploymentClient::new(server_config);
        
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.strategy.retry_attempts {
            attempts += 1;
            
            match client.deploy_agent(&self.binary_path) {
                Ok(_) => {
                    let duration = (Utc::now() - start_time).num_seconds() as u64;
                    info!("Successfully replicated to {} in {} seconds", target.ip, duration);
                    
                    return ReplicationResult {
                        target: target.ip.clone(),
                        success: true,
                        timestamp: Utc::now(),
                        duration_seconds: duration,
                        error: None,
                    };
                }
                Err(e) => {
                    warn!("Replication attempt {} to {} failed: {}", attempts, target.ip, e);
                    last_error = Some(e.to_string());
                    
                    if attempts < self.strategy.retry_attempts {
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        }

        let duration = (Utc::now() - start_time).num_seconds() as u64;
        error!("Failed to replicate to {} after {} attempts", target.ip, attempts);
        
        ReplicationResult {
            target: target.ip.clone(),
            success: false,
            timestamp: Utc::now(),
            duration_seconds: duration,
            error: last_error,
        }
    }

    pub async fn verify_replicas(&self) -> Result<HashMap<String, bool>> {
        let mut health_status = HashMap::new();
        let active_replicas = self.active_replicas.read().await.clone();

        for (ip, _) in active_replicas.iter() {
            let server_config = ServerConfig {
                name: format!("replica-{}", ip),
                ip: ip.clone(),
                port: 22,
                user: "ubuntu".to_string(), // This should come from configuration
                ssh_key_path: PathBuf::from("~/.ssh/id_rsa"),
                remote_deploy_path: PathBuf::from("/home/ubuntu/aurelia_agent"),
                role: deployment_tester::config::ServerRole::Replica,
            };

            let monitor = deployment_tester::AgentMonitor::new(server_config);
            
            match monitor.check_process_status() {
                Ok(is_running) => {
                    health_status.insert(ip.clone(), is_running);
                    if !is_running {
                        warn!("Replica {} is not running", ip);
                        // Remove from active replicas
                        self.active_replicas.write().await.remove(ip);
                    }
                }
                Err(e) => {
                    error!("Failed to check replica {} health: {}", ip, e);
                    health_status.insert(ip.clone(), false);
                    self.active_replicas.write().await.remove(ip);
                }
            }
        }

        Ok(health_status)
    }

    pub async fn auto_manage(&self) {
        info!("Starting autonomous replication management");
        
        loop {
            // 1. Verify existing replicas
            if let Err(e) = self.verify_replicas().await {
                error!("Failed to verify replicas: {}", e);
            }

            // 2. Check if replication is needed
            if self.should_replicate().await {
                if let Err(e) = self.replicate().await {
                    error!("Replication failed: {}", e);
                }
            }

            // 3. Clean up old history
            self.cleanup_history().await;

            // 4. Wait for next cycle
            tokio::time::sleep(
                tokio::time::Duration::from_secs(self.strategy.replication_interval_seconds)
            ).await;
        }
    }

    async fn cleanup_history(&self) {
        let mut history = self.replication_history.write().await;
        
        // Keep only last 1000 entries
        if history.len() > 1000 {
            let drain_count = history.len() - 1000;
            history.drain(0..drain_count);
        }
    }

    pub async fn get_status(&self) -> ReplicationStatus {
        ReplicationStatus {
            active_replicas: self.active_replicas.read().await.len(),
            total_targets: self.targets.read().await.len(),
            recent_failures: self.count_recent_failures().await,
            strategy: self.strategy.clone(),
        }
    }

    async fn count_recent_failures(&self) -> usize {
        let history = self.replication_history.read().await;
        let one_hour_ago = Utc::now() - chrono::Duration::hours(1);
        
        history.iter()
            .filter(|r| !r.success && r.timestamp > one_hour_ago)
            .count()
    }

    pub async fn trigger_emergency_replication(&self) -> Result<()> {
        warn!("Emergency replication triggered!");
        
        // Override normal limits for emergency
        let targets = self.targets.read().await.clone();
        let mut results = Vec::new();

        for target in targets.iter().take(3) {  // Replicate to up to 3 targets immediately
            let result = self.replicate_to_target(target).await;
            results.push(result);
        }

        let successful = results.iter().filter(|r| r.success).count();
        if successful == 0 {
            return Err(anyhow::anyhow!("Emergency replication failed on all targets"));
        }

        info!("Emergency replication completed: {}/{} successful", successful, results.len());
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStatus {
    pub active_replicas: usize,
    pub total_targets: usize,
    pub recent_failures: usize,
    pub strategy: ReplicationStrategy,
}