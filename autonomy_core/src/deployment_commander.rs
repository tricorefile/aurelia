use crate::server_config::{ServerConfig, TargetServer};
use crate::ssh_deployer::{AuthMethod, SshDeployer};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Deployment status for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatus {
    pub server_id: String,
    pub ip: String,
    pub status: DeploymentState,
    pub last_attempt: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentState {
    NotDeployed,
    Deploying,
    Running,
    Failed,
    Stopped,
}

/// High-level deployment commander that orchestrates deployments
pub struct DeploymentCommander {
    config: Arc<RwLock<ServerConfig>>,
    deployment_status: Arc<RwLock<HashMap<String, DeploymentStatus>>>,
    binary_path: PathBuf,
    config_files: Vec<PathBuf>,
}

impl DeploymentCommander {
    /// Create a new deployment commander
    pub fn new(binary_path: PathBuf) -> Self {
        let config = ServerConfig::from_file(Path::new("config/target_servers.json"))
            .unwrap_or_else(|_| ServerConfig {
                target_servers: Vec::new(),
                default_settings: crate::server_config::DefaultSettings {
                    port: 22,
                    username: "root".to_string(),
                    ssh_key_path: "~/.ssh/id_rsa".to_string(),
                    remote_path: "/opt/aurelia".to_string(),
                    max_retries: 3,
                    retry_delay_seconds: 5,
                    connection_timeout_seconds: 30,
                    deployment_timeout_seconds: 300,
                },
                deployment_strategy: crate::server_config::DeploymentStrategy {
                    strategy_type: "sequential".to_string(),
                    parallel_deployments: 1,
                    delay_between_deployments_seconds: 10,
                    health_check_after_deployment: true,
                    rollback_on_failure: false,
                },
                ssh_config: crate::server_config::SshConfig {
                    strict_host_key_checking: false,
                    compression: true,
                    keepalive_interval_seconds: 60,
                },
            });

        let mut deployment_status = HashMap::new();
        for server in &config.target_servers {
            deployment_status.insert(
                server.id.clone(),
                DeploymentStatus {
                    server_id: server.id.clone(),
                    ip: server.ip.clone(),
                    status: DeploymentState::NotDeployed,
                    last_attempt: None,
                    last_success: None,
                    error_message: None,
                },
            );
        }

        Self {
            config: Arc::new(RwLock::new(config)),
            deployment_status: Arc::new(RwLock::new(deployment_status)),
            binary_path,
            config_files: vec![PathBuf::from("config/target_servers.json")],
        }
    }

    /// Deploy to a specific server by ID
    pub async fn deploy_to_server(&self, server_id: &str) -> Result<()> {
        let config = self.config.read().await;
        let server = config
            .target_servers
            .iter()
            .find(|s| s.id == server_id)
            .ok_or_else(|| anyhow::anyhow!("Server {} not found", server_id))?
            .clone();

        drop(config); // Release the lock

        self.deploy_to_target(server).await
    }

    /// Deploy to all enabled servers
    pub async fn deploy_to_all(&self) -> Result<Vec<(String, Result<()>)>> {
        let config = self.config.read().await;
        let servers: Vec<_> = config
            .target_servers
            .iter()
            .filter(|s| s.enabled)
            .cloned()
            .collect();
        drop(config);

        let mut results = Vec::new();
        for server in servers {
            let server_id = server.id.clone();
            let result = self.deploy_to_target(server).await;
            results.push((server_id, result));
        }

        Ok(results)
    }

    /// Deploy to high-priority servers
    pub async fn deploy_to_priority_servers(
        &self,
        min_priority: u32,
    ) -> Result<Vec<(String, Result<()>)>> {
        let config = self.config.read().await;
        let servers: Vec<_> = config
            .target_servers
            .iter()
            .filter(|s| s.enabled && s.priority <= min_priority)
            .cloned()
            .collect();
        drop(config);

        let mut results = Vec::new();
        for server in servers {
            let server_id = server.id.clone();
            let result = self.deploy_to_target(server).await;
            results.push((server_id, result));
        }

        Ok(results)
    }

    /// Internal deployment logic
    async fn deploy_to_target(&self, server: TargetServer) -> Result<()> {
        info!("Starting deployment to {} ({})", server.name, server.ip);

        // Update status to deploying
        {
            let mut status = self.deployment_status.write().await;
            if let Some(s) = status.get_mut(&server.id) {
                s.status = DeploymentState::Deploying;
                s.last_attempt = Some(Utc::now());
            }
        }

        // Create SSH deployer
        let mut deployer = SshDeployer::new();

        // Determine authentication method
        let auth = match server.auth_method {
            crate::server_config::AuthMethod::Password => {
                let password = server
                    .get_password()
                    .ok_or_else(|| anyhow::anyhow!("Password not available for {}", server.name))?;
                AuthMethod::Password(password)
            }
            crate::server_config::AuthMethod::Key => AuthMethod::Key {
                path: server.get_expanded_ssh_key_path(),
                passphrase: None,
            },
            crate::server_config::AuthMethod::KeyWithPassphrase => AuthMethod::Key {
                path: server.get_expanded_ssh_key_path(),
                passphrase: server.get_password(),
            },
        };

        // Perform deployment
        let result = deployer.full_deploy(
            &server.ip,
            server.port as u16,
            &server.username,
            auth,
            &self.binary_path,
            &server.remote_path,
            Some(self.config_files.clone()),
            true, // Setup systemd service
        );

        // Update status based on result
        {
            let mut status = self.deployment_status.write().await;
            if let Some(s) = status.get_mut(&server.id) {
                match &result {
                    Ok(_) => {
                        s.status = DeploymentState::Running;
                        s.last_success = Some(Utc::now());
                        s.error_message = None;
                        info!("Successfully deployed to {} ({})", server.name, server.ip);
                    }
                    Err(e) => {
                        s.status = DeploymentState::Failed;
                        s.error_message = Some(e.to_string());
                        error!("Failed to deploy to {} ({}): {}", server.name, server.ip, e);
                    }
                }
            }
        }

        result
    }

    /// Check status of a deployed server
    pub async fn check_server_status(&self, server_id: &str) -> Result<bool> {
        let config = self.config.read().await;
        let server = config
            .target_servers
            .iter()
            .find(|s| s.id == server_id)
            .ok_or_else(|| anyhow::anyhow!("Server {} not found", server_id))?
            .clone();
        drop(config);

        let mut deployer = SshDeployer::new();

        // Connect
        match server.auth_method {
            crate::server_config::AuthMethod::Password => {
                let password = server
                    .get_password()
                    .ok_or_else(|| anyhow::anyhow!("Password not available"))?;
                deployer.connect_with_password(
                    &server.ip,
                    server.port as u16,
                    &server.username,
                    &password,
                )?;
            }
            crate::server_config::AuthMethod::Key => {
                deployer.connect_with_key(
                    &server.ip,
                    server.port as u16,
                    &server.username,
                    &server.get_expanded_ssh_key_path(),
                    None,
                )?;
            }
            crate::server_config::AuthMethod::KeyWithPassphrase => {
                deployer.connect_with_key(
                    &server.ip,
                    server.port as u16,
                    &server.username,
                    &server.get_expanded_ssh_key_path(),
                    server.get_password().as_deref(),
                )?;
            }
        }

        // Check status
        deployer.check_kernel_status()
    }

    /// Get logs from a server
    pub async fn get_server_logs(&self, server_id: &str, lines: usize) -> Result<String> {
        let config = self.config.read().await;
        let server = config
            .target_servers
            .iter()
            .find(|s| s.id == server_id)
            .ok_or_else(|| anyhow::anyhow!("Server {} not found", server_id))?
            .clone();
        drop(config);

        let mut deployer = SshDeployer::new();

        // Connect
        match server.auth_method {
            crate::server_config::AuthMethod::Password => {
                let password = server
                    .get_password()
                    .ok_or_else(|| anyhow::anyhow!("Password not available"))?;
                deployer.connect_with_password(
                    &server.ip,
                    server.port as u16,
                    &server.username,
                    &password,
                )?;
            }
            crate::server_config::AuthMethod::Key => {
                deployer.connect_with_key(
                    &server.ip,
                    server.port as u16,
                    &server.username,
                    &server.get_expanded_ssh_key_path(),
                    None,
                )?;
            }
            crate::server_config::AuthMethod::KeyWithPassphrase => {
                deployer.connect_with_key(
                    &server.ip,
                    server.port as u16,
                    &server.username,
                    &server.get_expanded_ssh_key_path(),
                    server.get_password().as_deref(),
                )?;
            }
        }

        // Get logs
        deployer.get_logs(&server.remote_path, lines)
    }

    /// Stop kernel on a server
    pub async fn stop_server(&self, server_id: &str) -> Result<()> {
        let config = self.config.read().await;
        let server = config
            .target_servers
            .iter()
            .find(|s| s.id == server_id)
            .ok_or_else(|| anyhow::anyhow!("Server {} not found", server_id))?
            .clone();
        drop(config);

        let mut deployer = SshDeployer::new();

        // Connect
        match server.auth_method {
            crate::server_config::AuthMethod::Password => {
                let password = server
                    .get_password()
                    .ok_or_else(|| anyhow::anyhow!("Password not available"))?;
                deployer.connect_with_password(
                    &server.ip,
                    server.port as u16,
                    &server.username,
                    &password,
                )?;
            }
            crate::server_config::AuthMethod::Key => {
                deployer.connect_with_key(
                    &server.ip,
                    server.port as u16,
                    &server.username,
                    &server.get_expanded_ssh_key_path(),
                    None,
                )?;
            }
            crate::server_config::AuthMethod::KeyWithPassphrase => {
                deployer.connect_with_key(
                    &server.ip,
                    server.port as u16,
                    &server.username,
                    &server.get_expanded_ssh_key_path(),
                    server.get_password().as_deref(),
                )?;
            }
        }

        // Stop kernel
        deployer.stop_kernel()?;

        // Update status
        {
            let mut status = self.deployment_status.write().await;
            if let Some(s) = status.get_mut(&server.id) {
                s.status = DeploymentState::Stopped;
            }
        }

        Ok(())
    }

    /// Get deployment status for all servers
    pub async fn get_deployment_status(&self) -> HashMap<String, DeploymentStatus> {
        self.deployment_status.read().await.clone()
    }

    /// Reload configuration from file
    pub async fn reload_config(&mut self) -> Result<()> {
        let new_config = ServerConfig::from_file(Path::new("config/target_servers.json"))?;

        // Update deployment status for new servers
        let mut status = self.deployment_status.write().await;
        for server in &new_config.target_servers {
            if !status.contains_key(&server.id) {
                status.insert(
                    server.id.clone(),
                    DeploymentStatus {
                        server_id: server.id.clone(),
                        ip: server.ip.clone(),
                        status: DeploymentState::NotDeployed,
                        last_attempt: None,
                        last_success: None,
                        error_message: None,
                    },
                );
            }
        }
        drop(status);

        *self.config.write().await = new_config;
        info!("Configuration reloaded");
        Ok(())
    }

    /// Execute a command on all deployed servers
    pub async fn execute_on_all(&self, command: &str) -> Result<Vec<(String, Result<String>)>> {
        let config = self.config.read().await;
        let servers: Vec<_> = config
            .target_servers
            .iter()
            .filter(|s| s.enabled)
            .cloned()
            .collect();
        drop(config);

        let mut results = Vec::new();
        for server in servers {
            let result = self.execute_on_server(&server.id, command).await;
            results.push((server.id.clone(), result));
        }

        Ok(results)
    }

    /// Execute a command on a specific server
    async fn execute_on_server(&self, server_id: &str, command: &str) -> Result<String> {
        let config = self.config.read().await;
        let server = config
            .target_servers
            .iter()
            .find(|s| s.id == server_id)
            .ok_or_else(|| anyhow::anyhow!("Server {} not found", server_id))?
            .clone();
        drop(config);

        let mut deployer = SshDeployer::new();

        // Connect
        match server.auth_method {
            crate::server_config::AuthMethod::Password => {
                let password = server
                    .get_password()
                    .ok_or_else(|| anyhow::anyhow!("Password not available"))?;
                deployer.connect_with_password(
                    &server.ip,
                    server.port as u16,
                    &server.username,
                    &password,
                )?;
            }
            crate::server_config::AuthMethod::Key => {
                deployer.connect_with_key(
                    &server.ip,
                    server.port as u16,
                    &server.username,
                    &server.get_expanded_ssh_key_path(),
                    None,
                )?;
            }
            crate::server_config::AuthMethod::KeyWithPassphrase => {
                deployer.connect_with_key(
                    &server.ip,
                    server.port as u16,
                    &server.username,
                    &server.get_expanded_ssh_key_path(),
                    server.get_password().as_deref(),
                )?;
            }
        }

        // Execute command
        deployer.execute_command(command)
    }
}
