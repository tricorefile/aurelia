use crate::config::TestConfig;
use crate::deployer::DeploymentClient;
use crate::monitor::AgentMonitor;
use crate::validator::ValidationSuite;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};

pub struct TestRunner {
    config: TestConfig,
    binary_path: PathBuf,
}

impl TestRunner {
    pub fn new(config: TestConfig, binary_path: PathBuf) -> Self {
        Self {
            config,
            binary_path,
        }
    }

    pub async fn run_complete_test_suite(&self) -> Result<()> {
        info!("=== Starting Aurelia Agent Deployment Test Suite ===");

        // Phase 1: Pre-deployment checks
        self.pre_deployment_checks().await?;

        // Phase 2: Deploy agents
        self.deploy_all_agents().await?;

        // Wait for agents to stabilize
        info!("Waiting for agents to stabilize...");
        time::sleep(Duration::from_secs(10)).await;

        // Phase 3: Test self-replication
        self.test_self_replication().await?;

        // Phase 4: Run validation suite
        self.run_validation().await?;

        // Phase 5: Monitor continuously
        if self.config.test_settings.test_duration_minutes > 0 {
            self.continuous_monitoring().await?;
        }

        info!("=== Test Suite Completed Successfully ===");
        Ok(())
    }

    async fn pre_deployment_checks(&self) -> Result<()> {
        info!("Starting pre-deployment checks...");

        // Check binary exists
        if !self.binary_path.exists() {
            return Err(anyhow::anyhow!(
                "Binary not found at {:?}. Please run 'cargo build --release' first.",
                self.binary_path
            ));
        }

        // Test SSH connectivity to all servers
        for server in &self.config.test_environments {
            info!("Testing connection to {}...", server.name);
            let client = DeploymentClient::new(server.clone());

            match client.test_connection() {
                Ok(true) => info!("✓ Connection to {} successful", server.name),
                Ok(false) => {
                    return Err(anyhow::anyhow!("Connection test to {} failed", server.name))
                }
                Err(e) => return Err(anyhow::anyhow!("Cannot connect to {}: {}", server.name, e)),
            }
        }

        info!("Pre-deployment checks completed successfully");
        Ok(())
    }

    async fn deploy_all_agents(&self) -> Result<()> {
        info!("Starting deployment to all servers...");

        for server in &self.config.test_environments {
            info!("Deploying to {} ({:?})...", server.name, server.role);
            let client = DeploymentClient::new(server.clone());

            client
                .deploy_agent(&self.binary_path)
                .context(format!("Failed to deploy to {}", server.name))?;

            info!("✓ Deployment to {} completed", server.name);
        }

        Ok(())
    }

    async fn test_self_replication(&self) -> Result<()> {
        info!("Testing self-replication capability...");

        let primary = self
            .config
            .get_primary_server()
            .ok_or_else(|| anyhow::anyhow!("No primary server configured"))?;

        let replicas = self.config.get_replica_servers();
        if replicas.is_empty() {
            warn!("No replica servers configured, skipping replication test");
            return Ok(());
        }

        let replica = replicas[0];

        info!(
            "Triggering self-replication from {} to {}",
            primary.name, replica.name
        );

        let primary_client = DeploymentClient::new(primary.clone());
        primary_client.trigger_self_replication(replica)?;

        // Wait for replication to complete
        info!("Waiting for replication to complete...");
        time::sleep(Duration::from_secs(30)).await;

        // Verify replication
        let replica_monitor = AgentMonitor::new(replica.clone());
        match replica_monitor.verify_replica_deployment() {
            Ok(true) => info!("✓ Self-replication successful"),
            Ok(false) => {
                return Err(anyhow::anyhow!(
                    "Self-replication failed - replica not found"
                ))
            }
            Err(e) => return Err(anyhow::anyhow!("Failed to verify replication: {}", e)),
        }

        Ok(())
    }

    async fn run_validation(&self) -> Result<()> {
        info!("Running validation suite...");

        let mut validator = ValidationSuite::new(self.config.clone());
        let summary = validator.run_full_validation().await?;

        validator.print_summary();
        validator.save_results("validation_results.json")?;

        if summary.failed > 0 {
            warn!("{} tests failed", summary.failed);
        } else {
            info!("All validation tests passed!");
        }

        Ok(())
    }

    async fn continuous_monitoring(&self) -> Result<()> {
        let duration_minutes = self.config.test_settings.test_duration_minutes;
        let interval_seconds = self.config.test_settings.health_check_interval_seconds;

        info!(
            "Starting continuous monitoring for {} minutes...",
            duration_minutes
        );

        let start_time = std::time::Instant::now();
        let total_duration = Duration::from_secs(duration_minutes * 60);
        let check_interval = Duration::from_secs(interval_seconds);

        while start_time.elapsed() < total_duration {
            let remaining = total_duration - start_time.elapsed();
            info!(
                "Monitoring... ({} minutes remaining)",
                remaining.as_secs() / 60
            );

            // Perform health checks on all servers
            for server in &self.config.test_environments {
                let monitor = AgentMonitor::new(server.clone());
                match monitor.check_agent_health() {
                    Ok(health) => {
                        if health.is_running {
                            info!("✓ {} is healthy", server.name);
                        } else {
                            warn!("✗ {} is not running", server.name);
                        }
                    }
                    Err(e) => {
                        error!("Failed to check {}: {}", server.name, e);
                    }
                }
            }

            time::sleep(check_interval).await;
        }

        info!("Continuous monitoring completed");
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<()> {
        info!("Cleaning up test deployment...");

        for server in &self.config.test_environments {
            info!("Cleaning up {}...", server.name);
            let client = DeploymentClient::new(server.clone());

            // Stop agent
            if let Err(e) = client.stop_agent() {
                warn!("Failed to stop agent on {}: {}", server.name, e);
            }

            // Remove files
            if let Err(e) = client.cleanup() {
                warn!("Failed to cleanup {}: {}", server.name, e);
            }
        }

        info!("Cleanup completed");
        Ok(())
    }

    pub async fn run_specific_test(&self, test_name: &str) -> Result<()> {
        match test_name {
            "connection" => self.pre_deployment_checks().await,
            "deploy" => self.deploy_all_agents().await,
            "replication" => self.test_self_replication().await,
            "validation" => self.run_validation().await,
            "monitor" => self.continuous_monitoring().await,
            _ => Err(anyhow::anyhow!("Unknown test: {}", test_name)),
        }
    }
}
