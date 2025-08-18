use crate::config::{ServerConfig, ServerRole, TestConfig};
use crate::monitor::{AgentMonitor, MonitoringResult};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub test_name: String,
    pub server: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub passed: bool,
    pub details: HashMap<String, serde_json::Value>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub success_rate: f64,
    pub results: Vec<ValidationResult>,
}

pub struct ValidationSuite {
    config: TestConfig,
    results: Vec<ValidationResult>,
}

impl ValidationSuite {
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    pub async fn run_full_validation(&mut self) -> Result<ValidationSummary> {
        let start_time = Utc::now();
        info!("Starting full validation suite");

        // Test 1: Agent Running
        self.validate_agents_running().await?;

        // Test 2: Resource Usage
        self.validate_resource_usage().await?;

        // Test 3: Log Activity
        self.validate_log_activity().await?;

        // Test 4: Autonomous Behavior
        self.validate_autonomous_behavior().await?;

        // Test 5: Network Communication
        self.validate_network_communication().await?;

        // Test 6: Self-Replication
        let primary = self.config.get_primary_server().cloned();
        let replicas: Vec<_> = self
            .config
            .get_replica_servers()
            .into_iter()
            .cloned()
            .collect();
        if let Some(primary_server) = primary {
            if !replicas.is_empty() {
                self.validate_self_replication(&primary_server, &replicas[0])
                    .await?;
            }
        }

        let end_time = Utc::now();
        let summary = self.generate_summary(start_time, end_time);

        info!(
            "Validation suite completed: {} passed, {} failed",
            summary.passed, summary.failed
        );

        Ok(summary)
    }

    async fn validate_agents_running(&mut self) -> Result<()> {
        info!("Validating agent processes...");

        for server in &self.config.test_environments {
            let monitor = AgentMonitor::new(server.clone());
            let mut result = ValidationResult {
                test_name: "agent_running".to_string(),
                server: Some(server.name.clone()),
                timestamp: Utc::now(),
                passed: false,
                details: HashMap::new(),
                errors: Vec::new(),
            };

            match monitor.check_agent_health() {
                Ok(health) => {
                    result.passed = health.is_running;
                    result.details.insert(
                        "is_running".to_string(),
                        serde_json::json!(health.is_running),
                    );
                    result
                        .details
                        .insert("cpu_usage".to_string(), serde_json::json!(health.cpu_usage));
                    result
                        .details
                        .insert("memory_mb".to_string(), serde_json::json!(health.memory_mb));

                    if !health.errors.is_empty() {
                        result.errors = health.errors;
                    }
                }
                Err(e) => {
                    result.errors.push(format!("Health check failed: {}", e));
                }
            }

            self.results.push(result);
        }

        Ok(())
    }

    async fn validate_resource_usage(&mut self) -> Result<()> {
        info!("Validating resource usage...");

        let limits = &self.config.test_settings.resource_limits;

        for server in &self.config.test_environments {
            let monitor = AgentMonitor::new(server.clone());
            let mut result = ValidationResult {
                test_name: "resource_usage".to_string(),
                server: Some(server.name.clone()),
                timestamp: Utc::now(),
                passed: false,
                details: HashMap::new(),
                errors: Vec::new(),
            };

            match monitor.get_resource_metrics() {
                Ok(metrics) => {
                    let cpu_ok = metrics.cpu_percent < limits.max_cpu_percent;
                    let mem_ok = metrics.memory_mb < limits.max_memory_mb as f64;
                    let disk_ok = metrics.disk_usage_gb < limits.max_disk_gb as f64;

                    result.passed = cpu_ok && mem_ok && disk_ok;
                    result.details.insert(
                        "cpu_percent".to_string(),
                        serde_json::json!(metrics.cpu_percent),
                    );
                    result.details.insert(
                        "cpu_limit".to_string(),
                        serde_json::json!(limits.max_cpu_percent),
                    );
                    result.details.insert(
                        "memory_mb".to_string(),
                        serde_json::json!(metrics.memory_mb),
                    );
                    result.details.insert(
                        "memory_limit_mb".to_string(),
                        serde_json::json!(limits.max_memory_mb),
                    );
                    result.details.insert(
                        "disk_gb".to_string(),
                        serde_json::json!(metrics.disk_usage_gb),
                    );
                    result.details.insert(
                        "disk_limit_gb".to_string(),
                        serde_json::json!(limits.max_disk_gb),
                    );
                }
                Err(e) => {
                    result.errors.push(format!("Resource check failed: {}", e));
                }
            }

            self.results.push(result);
        }

        Ok(())
    }

    async fn validate_log_activity(&mut self) -> Result<()> {
        info!("Validating log activity...");

        for server in &self.config.test_environments {
            let monitor = AgentMonitor::new(server.clone());
            let mut result = ValidationResult {
                test_name: "log_activity".to_string(),
                server: Some(server.name.clone()),
                timestamp: Utc::now(),
                passed: false,
                details: HashMap::new(),
                errors: Vec::new(),
            };

            match monitor.check_log_activity() {
                Ok((active, logs)) => {
                    result.passed = active;
                    result
                        .details
                        .insert("has_activity".to_string(), serde_json::json!(active));
                    result
                        .details
                        .insert("log_count".to_string(), serde_json::json!(logs.len()));

                    if !logs.is_empty() {
                        result.details.insert(
                            "recent_logs".to_string(),
                            serde_json::json!(logs.iter().take(5).collect::<Vec<_>>()),
                        );
                    }
                }
                Err(e) => {
                    result.errors.push(format!("Log check failed: {}", e));
                }
            }

            self.results.push(result);
        }

        Ok(())
    }

    async fn validate_autonomous_behavior(&mut self) -> Result<()> {
        info!("Validating autonomous behavior...");

        for server in &self.config.test_environments {
            let monitor = AgentMonitor::new(server.clone());
            let mut result = ValidationResult {
                test_name: "autonomous_behavior".to_string(),
                server: Some(server.name.clone()),
                timestamp: Utc::now(),
                passed: false,
                details: HashMap::new(),
                errors: Vec::new(),
            };

            match monitor.check_autonomous_behavior() {
                Ok(behaviors) => {
                    let has_any_behavior = behaviors.values().any(|&v| v);
                    result.passed = has_any_behavior;

                    for (key, value) in behaviors {
                        result.details.insert(key, serde_json::json!(value));
                    }
                }
                Err(e) => {
                    result.errors.push(format!("Behavior check failed: {}", e));
                }
            }

            self.results.push(result);
        }

        Ok(())
    }

    async fn validate_network_communication(&mut self) -> Result<()> {
        info!("Validating network communication...");

        for server in &self.config.test_environments {
            let monitor = AgentMonitor::new(server.clone());
            let mut result = ValidationResult {
                test_name: "network_communication".to_string(),
                server: Some(server.name.clone()),
                timestamp: Utc::now(),
                passed: false,
                details: HashMap::new(),
                errors: Vec::new(),
            };

            match monitor.check_network_connections() {
                Ok(count) => {
                    result.passed = count > 0 || matches!(server.role, ServerRole::Replica);
                    result.details.insert(
                        "websocket_connections".to_string(),
                        serde_json::json!(count),
                    );
                }
                Err(e) => {
                    result.errors.push(format!("Network check failed: {}", e));
                }
            }

            self.results.push(result);
        }

        Ok(())
    }

    async fn validate_self_replication(
        &mut self,
        primary: &ServerConfig,
        replica: &ServerConfig,
    ) -> Result<()> {
        info!(
            "Validating self-replication from {} to {}",
            primary.name, replica.name
        );

        let mut result = ValidationResult {
            test_name: "self_replication".to_string(),
            server: Some(format!("{} -> {}", primary.name, replica.name)),
            timestamp: Utc::now(),
            passed: false,
            details: HashMap::new(),
            errors: Vec::new(),
        };

        let replica_monitor = AgentMonitor::new(replica.clone());

        match replica_monitor.verify_replica_deployment() {
            Ok(exists) => {
                result.passed = exists;
                result
                    .details
                    .insert("replica_deployed".to_string(), serde_json::json!(exists));

                if exists {
                    // Check if replica is running
                    if let Ok(health) = replica_monitor.check_agent_health() {
                        result.details.insert(
                            "replica_running".to_string(),
                            serde_json::json!(health.is_running),
                        );
                    }
                }
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Replication check failed: {}", e));
            }
        }

        self.results.push(result);
        Ok(())
    }

    fn generate_summary(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> ValidationSummary {
        let total_tests = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total_tests - passed;
        let success_rate = if total_tests > 0 {
            (passed as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        ValidationSummary {
            start_time,
            end_time,
            total_tests,
            passed,
            failed,
            success_rate,
            results: self.results.clone(),
        }
    }

    pub fn save_results(&self, path: &str) -> Result<()> {
        let summary = self.generate_summary(
            self.results
                .first()
                .map(|r| r.timestamp)
                .unwrap_or_else(Utc::now),
            Utc::now(),
        );

        let json = serde_json::to_string_pretty(&summary)?;
        std::fs::write(path, json)?;
        info!("Validation results saved to {}", path);
        Ok(())
    }

    pub fn print_summary(&self) {
        let summary = self.generate_summary(
            self.results
                .first()
                .map(|r| r.timestamp)
                .unwrap_or_else(Utc::now),
            Utc::now(),
        );

        println!("\n{}", "=".repeat(50));
        println!("VALIDATION SUMMARY");
        println!("{}", "=".repeat(50));
        println!("Total Tests: {}", summary.total_tests);
        println!("Passed: {}", summary.passed);
        println!("Failed: {}", summary.failed);
        println!("Success Rate: {:.1}%", summary.success_rate);
        println!();

        // Print details for failed tests
        if summary.failed > 0 {
            println!("Failed Tests:");
            for result in &self.results {
                if !result.passed {
                    println!(
                        "  - {} ({}): {:?}",
                        result.test_name,
                        result.server.as_ref().unwrap_or(&"N/A".to_string()),
                        result.errors
                    );
                }
            }
        }
    }
}
