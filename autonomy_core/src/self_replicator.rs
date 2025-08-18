use crate::server_config::{ServerConfig, TargetServer};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use deployment_tester::{DeploymentClient, ServerConfig as TestServerConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    server_config: Option<ServerConfig>,
}

impl SelfReplicator {
    pub fn new(binary_path: PathBuf) -> Self {
        // 尝试加载配置文件
        let server_config = Self::load_server_config();

        // 如果有配置文件，从中初始化目标服务器
        let targets = if let Some(ref config) = server_config {
            config
                .get_servers_by_priority()
                .into_iter()
                .map(|server| ReplicationTarget {
                    ip: server.ip.clone(),
                    user: server.username.clone(),
                    ssh_key_path: server.get_expanded_ssh_key_path(),
                    remote_path: PathBuf::from(&server.remote_path),
                    priority: server.priority as u8,
                    last_attempt: None,
                    success_count: 0,
                    failure_count: 0,
                })
                .collect()
        } else {
            Vec::new()
        };

        Self {
            strategy: ReplicationStrategy::default(),
            targets: Arc::new(RwLock::new(targets)),
            active_replicas: Arc::new(RwLock::new(HashMap::new())),
            replication_history: Arc::new(RwLock::new(Vec::new())),
            binary_path,
            server_config,
        }
    }

    fn load_server_config() -> Option<ServerConfig> {
        let config_path = PathBuf::from("config/target_servers.json");

        match ServerConfig::from_file(&config_path) {
            Ok(config) => {
                info!("Loaded server configuration from {:?}", config_path);
                Some(config)
            }
            Err(e) => {
                warn!(
                    "Failed to load server configuration: {}. Using fallback targets.",
                    e
                );
                None
            }
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

    /// 从配置文件重新加载目标服务器
    pub async fn reload_targets_from_config(&mut self) -> Result<usize> {
        let config = Self::load_server_config()
            .ok_or_else(|| anyhow::anyhow!("Failed to load server configuration"))?;

        let new_targets: Vec<ReplicationTarget> = config
            .get_servers_by_priority()
            .into_iter()
            .map(|server| ReplicationTarget {
                ip: server.ip.clone(),
                user: server.username.clone(),
                ssh_key_path: server.get_expanded_ssh_key_path(),
                remote_path: PathBuf::from(&server.remote_path),
                priority: server.priority as u8,
                last_attempt: None,
                success_count: 0,
                failure_count: 0,
            })
            .collect();

        let count = new_targets.len();
        *self.targets.write().await = new_targets;
        self.server_config = Some(config);

        info!("Reloaded {} target servers from configuration", count);
        Ok(count)
    }

    /// 添加新服务器到配置并保存
    pub async fn add_server_to_config(&mut self, server: TargetServer) -> Result<()> {
        if let Some(ref mut config) = self.server_config {
            config.add_server(server.clone())?;
            config.save_to_file("config/target_servers.json")?;

            // 添加到运行时目标列表
            let target = ReplicationTarget {
                ip: server.ip.clone(),
                user: server.username.clone(),
                ssh_key_path: server.get_expanded_ssh_key_path(),
                remote_path: PathBuf::from(&server.remote_path),
                priority: server.priority as u8,
                last_attempt: None,
                success_count: 0,
                failure_count: 0,
            };
            self.targets.write().await.push(target);

            info!("Added server {} to configuration", server.id);
        } else {
            return Err(anyhow::anyhow!("No server configuration loaded"));
        }
        Ok(())
    }

    /// 获取当前配置的服务器列表
    pub fn get_configured_servers(&self) -> Vec<TargetServer> {
        if let Some(ref config) = self.server_config {
            config.target_servers.clone()
        } else {
            Vec::new()
        }
    }

    pub async fn should_replicate(&self) -> bool {
        let active_count = self.active_replicas.read().await.len();

        if active_count < self.strategy.min_replicas {
            info!(
                "Active replicas ({}) below minimum ({}), replication needed",
                active_count, self.strategy.min_replicas
            );
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
                self.active_replicas
                    .write()
                    .await
                    .insert(target.ip.clone(), Utc::now());

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

        // 从原始配置中获取完整的服务器信息（包括认证方式和密码）
        let full_server_info = if let Some(ref config) = self.server_config {
            config
                .target_servers
                .iter()
                .find(|s| s.ip == target.ip && s.enabled)
                .cloned()
        } else {
            None
        };

        // 构建deployment_tester的ServerConfig
        let server_config = if let Some(server_info) = full_server_info {
            // 根据认证方式构建配置
            match server_info.auth_method {
                crate::server_config::AuthMethod::Password => {
                    // 解码密码
                    let password = server_info.get_password().unwrap_or_default();
                    TestServerConfig {
                        name: format!("replica-{}", target.ip),
                        ip: target.ip.clone(),
                        port: server_info.port,
                        user: target.user.clone(),
                        ssh_key_path: None,
                        password: Some(password),
                        auth_method: deployment_tester::config::AuthMethod::Password,
                        remote_deploy_path: target.remote_path.clone(),
                        role: deployment_tester::config::ServerRole::Replica,
                    }
                }
                crate::server_config::AuthMethod::KeyWithPassphrase => {
                    let passphrase = server_info.get_password();
                    TestServerConfig {
                        name: format!("replica-{}", target.ip),
                        ip: target.ip.clone(),
                        port: server_info.port,
                        user: target.user.clone(),
                        ssh_key_path: Some(target.ssh_key_path.clone()),
                        password: passphrase,
                        auth_method: deployment_tester::config::AuthMethod::KeyWithPassphrase,
                        remote_deploy_path: target.remote_path.clone(),
                        role: deployment_tester::config::ServerRole::Replica,
                    }
                }
                _ => {
                    // 默认使用密钥认证
                    TestServerConfig {
                        name: format!("replica-{}", target.ip),
                        ip: target.ip.clone(),
                        port: server_info.port,
                        user: target.user.clone(),
                        ssh_key_path: Some(target.ssh_key_path.clone()),
                        password: None,
                        auth_method: deployment_tester::config::AuthMethod::Key,
                        remote_deploy_path: target.remote_path.clone(),
                        role: deployment_tester::config::ServerRole::Replica,
                    }
                }
            }
        } else {
            // 如果找不到配置，使用默认的密钥认证
            TestServerConfig {
                name: format!("replica-{}", target.ip),
                ip: target.ip.clone(),
                port: 22,
                user: target.user.clone(),
                ssh_key_path: Some(target.ssh_key_path.clone()),
                password: None,
                auth_method: deployment_tester::config::AuthMethod::Key,
                remote_deploy_path: target.remote_path.clone(),
                role: deployment_tester::config::ServerRole::Replica,
            }
        };

        let client = DeploymentClient::new(server_config);

        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.strategy.retry_attempts {
            attempts += 1;

            match client.deploy_agent(&self.binary_path) {
                Ok(_) => {
                    let duration = (Utc::now() - start_time).num_seconds() as u64;
                    info!(
                        "Successfully replicated to {} in {} seconds",
                        target.ip, duration
                    );

                    return ReplicationResult {
                        target: target.ip.clone(),
                        success: true,
                        timestamp: Utc::now(),
                        duration_seconds: duration,
                        error: None,
                    };
                }
                Err(e) => {
                    warn!(
                        "Replication attempt {} to {} failed: {}",
                        attempts, target.ip, e
                    );
                    last_error = Some(e.to_string());

                    if attempts < self.strategy.retry_attempts {
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        }

        let duration = (Utc::now() - start_time).num_seconds() as u64;
        error!(
            "Failed to replicate to {} after {} attempts",
            target.ip, attempts
        );

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
            // 从配置中获取服务器信息
            let server_info = if let Some(ref config) = self.server_config {
                config.target_servers.iter().find(|s| &s.ip == ip).cloned()
            } else {
                None
            };

            let server_config = if let Some(info) = server_info {
                // 使用实际的服务器配置
                match info.auth_method {
                    crate::server_config::AuthMethod::Password => TestServerConfig {
                        name: format!("replica-{}", ip),
                        ip: ip.clone(),
                        port: info.port,
                        user: info.username.clone(),
                        ssh_key_path: None,
                        password: info.get_password(),
                        auth_method: deployment_tester::config::AuthMethod::Password,
                        remote_deploy_path: PathBuf::from(&info.remote_path),
                        role: deployment_tester::config::ServerRole::Replica,
                    },
                    crate::server_config::AuthMethod::KeyWithPassphrase => TestServerConfig {
                        name: format!("replica-{}", ip),
                        ip: ip.clone(),
                        port: info.port,
                        user: info.username.clone(),
                        ssh_key_path: Some(info.get_expanded_ssh_key_path()),
                        password: info.get_password(),
                        auth_method: deployment_tester::config::AuthMethod::KeyWithPassphrase,
                        remote_deploy_path: PathBuf::from(&info.remote_path),
                        role: deployment_tester::config::ServerRole::Replica,
                    },
                    _ => TestServerConfig {
                        name: format!("replica-{}", ip),
                        ip: ip.clone(),
                        port: info.port,
                        user: info.username.clone(),
                        ssh_key_path: Some(info.get_expanded_ssh_key_path()),
                        password: None,
                        auth_method: deployment_tester::config::AuthMethod::Key,
                        remote_deploy_path: PathBuf::from(&info.remote_path),
                        role: deployment_tester::config::ServerRole::Replica,
                    },
                }
            } else {
                // 使用默认配置
                TestServerConfig {
                    name: format!("replica-{}", ip),
                    ip: ip.clone(),
                    port: 22,
                    user: "ubuntu".to_string(),
                    ssh_key_path: Some(PathBuf::from("~/.ssh/id_rsa")),
                    password: None,
                    auth_method: deployment_tester::config::AuthMethod::Key,
                    remote_deploy_path: PathBuf::from("/home/ubuntu/aurelia_agent"),
                    role: deployment_tester::config::ServerRole::Replica,
                }
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
            tokio::time::sleep(tokio::time::Duration::from_secs(
                self.strategy.replication_interval_seconds,
            ))
            .await;
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

        history
            .iter()
            .filter(|r| !r.success && r.timestamp > one_hour_ago)
            .count()
    }

    pub async fn trigger_emergency_replication(&self) -> Result<()> {
        warn!("Emergency replication triggered!");

        // Override normal limits for emergency
        let targets = self.targets.read().await.clone();
        let mut results = Vec::new();

        for target in targets.iter().take(3) {
            // Replicate to up to 3 targets immediately
            let result = self.replicate_to_target(target).await;
            results.push(result);
        }

        let successful = results.iter().filter(|r| r.success).count();
        if successful == 0 {
            return Err(anyhow::anyhow!(
                "Emergency replication failed on all targets"
            ));
        }

        info!(
            "Emergency replication completed: {}/{} successful",
            successful,
            results.len()
        );
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
