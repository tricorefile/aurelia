use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_latency_ms: f64,
    pub process_count: usize,
    pub error_rate: f64,
    pub success_rate: f64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Critical(String),
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub consecutive_failures: u32,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct HealthThresholds {
    pub cpu_warning: f64,
    pub cpu_critical: f64,
    pub memory_warning: f64,
    pub memory_critical: f64,
    pub disk_warning: f64,
    pub disk_critical: f64,
    pub error_rate_warning: f64,
    pub error_rate_critical: f64,
    pub max_consecutive_failures: u32,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            cpu_warning: 70.0,
            cpu_critical: 90.0,
            memory_warning: 75.0,
            memory_critical: 90.0,
            disk_warning: 80.0,
            disk_critical: 95.0,
            error_rate_warning: 0.05,
            error_rate_critical: 0.1,
            max_consecutive_failures: 3,
        }
    }
}

pub struct HealthMonitor {
    thresholds: HealthThresholds,
    current_metrics: Arc<RwLock<HealthMetrics>>,
    health_checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
    metrics_history: Arc<RwLock<Vec<HealthMetrics>>>,
    #[allow(clippy::type_complexity)]
    alert_callbacks: Arc<RwLock<Vec<Box<dyn Fn(HealthAlert) + Send + Sync>>>>,
    monitoring_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub timestamp: DateTime<Utc>,
    pub severity: AlertSeverity,
    pub component: String,
    pub message: String,
    pub metrics: Option<HealthMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Fatal,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            thresholds: HealthThresholds::default(),
            current_metrics: Arc::new(RwLock::new(Self::default_metrics())),
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            alert_callbacks: Arc::new(RwLock::new(Vec::new())),
            monitoring_interval: Duration::seconds(30),
        }
    }

    fn default_metrics() -> HealthMetrics {
        HealthMetrics {
            timestamp: Utc::now(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_latency_ms: 0.0,
            process_count: 1,
            error_rate: 0.0,
            success_rate: 1.0,
            uptime_seconds: 0,
        }
    }

    pub async fn start_monitoring(&self) {
        info!("Starting autonomous health monitoring");

        loop {
            // Collect metrics
            if let Err(e) = self.collect_metrics().await {
                error!("Failed to collect metrics: {}", e);
            }

            // Run health checks
            self.run_health_checks().await;

            // Analyze and alert
            self.analyze_health().await;

            // Clean up old data
            self.cleanup_history().await;

            // Wait for next cycle
            tokio::time::sleep(self.monitoring_interval.to_std().unwrap()).await;
        }
    }

    async fn collect_metrics(&self) -> Result<()> {
        let metrics = self.collect_system_metrics().await?;

        // Update current metrics
        *self.current_metrics.write().await = metrics.clone();

        // Add to history
        self.metrics_history.write().await.push(metrics);

        Ok(())
    }

    async fn collect_system_metrics(&self) -> Result<HealthMetrics> {
        // Get system information
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;
        let total_memory = sys.total_memory() as f64;
        let used_memory = sys.used_memory() as f64;
        let memory_usage = (used_memory / total_memory) * 100.0;

        // Get disk usage
        let disk_usage = 30.0; // Placeholder for disk usage

        // Calculate process count
        let process_count = sys.processes().len();

        // Get uptime
        let uptime_seconds = sysinfo::System::uptime();

        Ok(HealthMetrics {
            timestamp: Utc::now(),
            cpu_usage,
            memory_usage,
            disk_usage,
            network_latency_ms: self.measure_network_latency().await,
            process_count,
            error_rate: self.calculate_error_rate().await,
            success_rate: self.calculate_success_rate().await,
            uptime_seconds,
        })
    }

    async fn measure_network_latency(&self) -> f64 {
        // In a real implementation, this would ping a reference server
        // For now, return a simulated value
        5.0 + rand::random::<f64>() * 10.0
    }

    async fn calculate_error_rate(&self) -> f64 {
        // In a real implementation, this would track actual errors
        // For now, return a simulated value
        rand::random::<f64>() * 0.05
    }

    async fn calculate_success_rate(&self) -> f64 {
        1.0 - self.calculate_error_rate().await
    }

    async fn run_health_checks(&self) {
        let mut checks = self.health_checks.write().await;

        // CPU Health Check
        self.check_cpu_health(&mut checks).await;

        // Memory Health Check
        self.check_memory_health(&mut checks).await;

        // Disk Health Check
        self.check_disk_health(&mut checks).await;

        // Network Health Check
        self.check_network_health(&mut checks).await;

        // Process Health Check
        self.check_process_health(&mut checks).await;
    }

    async fn check_cpu_health(&self, checks: &mut HashMap<String, HealthCheck>) {
        let metrics = self.current_metrics.read().await;
        let cpu_usage = metrics.cpu_usage;

        let status = if cpu_usage > self.thresholds.cpu_critical {
            HealthStatus::Critical(format!("CPU usage: {:.1}%", cpu_usage))
        } else if cpu_usage > self.thresholds.cpu_warning {
            HealthStatus::Degraded(format!("CPU usage: {:.1}%", cpu_usage))
        } else {
            HealthStatus::Healthy
        };

        let check = checks.entry("cpu".to_string()).or_insert(HealthCheck {
            name: "CPU Usage".to_string(),
            status: HealthStatus::Healthy,
            last_check: Utc::now(),
            consecutive_failures: 0,
            details: HashMap::new(),
        });

        check.status = status;
        check.last_check = Utc::now();
        check
            .details
            .insert("usage".to_string(), format!("{:.1}%", cpu_usage));
    }

    async fn check_memory_health(&self, checks: &mut HashMap<String, HealthCheck>) {
        let metrics = self.current_metrics.read().await;
        let memory_usage = metrics.memory_usage;

        let status = if memory_usage > self.thresholds.memory_critical {
            HealthStatus::Critical(format!("Memory usage: {:.1}%", memory_usage))
        } else if memory_usage > self.thresholds.memory_warning {
            HealthStatus::Degraded(format!("Memory usage: {:.1}%", memory_usage))
        } else {
            HealthStatus::Healthy
        };

        let check = checks.entry("memory".to_string()).or_insert(HealthCheck {
            name: "Memory Usage".to_string(),
            status: HealthStatus::Healthy,
            last_check: Utc::now(),
            consecutive_failures: 0,
            details: HashMap::new(),
        });

        check.status = status;
        check.last_check = Utc::now();
        check
            .details
            .insert("usage".to_string(), format!("{:.1}%", memory_usage));
    }

    async fn check_disk_health(&self, checks: &mut HashMap<String, HealthCheck>) {
        let metrics = self.current_metrics.read().await;
        let disk_usage = metrics.disk_usage;

        let status = if disk_usage > self.thresholds.disk_critical {
            HealthStatus::Critical(format!("Disk usage: {:.1}%", disk_usage))
        } else if disk_usage > self.thresholds.disk_warning {
            HealthStatus::Degraded(format!("Disk usage: {:.1}%", disk_usage))
        } else {
            HealthStatus::Healthy
        };

        let check = checks.entry("disk".to_string()).or_insert(HealthCheck {
            name: "Disk Usage".to_string(),
            status: HealthStatus::Healthy,
            last_check: Utc::now(),
            consecutive_failures: 0,
            details: HashMap::new(),
        });

        check.status = status;
        check.last_check = Utc::now();
        check
            .details
            .insert("usage".to_string(), format!("{:.1}%", disk_usage));
    }

    async fn check_network_health(&self, checks: &mut HashMap<String, HealthCheck>) {
        let metrics = self.current_metrics.read().await;
        let latency = metrics.network_latency_ms;

        let status = if latency > 100.0 {
            HealthStatus::Critical(format!("Network latency: {:.1}ms", latency))
        } else if latency > 50.0 {
            HealthStatus::Degraded(format!("Network latency: {:.1}ms", latency))
        } else {
            HealthStatus::Healthy
        };

        let check = checks.entry("network".to_string()).or_insert(HealthCheck {
            name: "Network Latency".to_string(),
            status: HealthStatus::Healthy,
            last_check: Utc::now(),
            consecutive_failures: 0,
            details: HashMap::new(),
        });

        check.status = status;
        check.last_check = Utc::now();
        check
            .details
            .insert("latency".to_string(), format!("{:.1}ms", latency));
    }

    async fn check_process_health(&self, checks: &mut HashMap<String, HealthCheck>) {
        let metrics = self.current_metrics.read().await;
        let process_count = metrics.process_count;

        let status = if process_count == 0 {
            HealthStatus::Failed("No processes running".to_string())
        } else if process_count > 100 {
            HealthStatus::Degraded(format!("High process count: {}", process_count))
        } else {
            HealthStatus::Healthy
        };

        let check = checks
            .entry("processes".to_string())
            .or_insert(HealthCheck {
                name: "Process Count".to_string(),
                status: HealthStatus::Healthy,
                last_check: Utc::now(),
                consecutive_failures: 0,
                details: HashMap::new(),
            });

        check.status = status;
        check.last_check = Utc::now();
        check
            .details
            .insert("count".to_string(), process_count.to_string());
    }

    async fn analyze_health(&self) {
        let checks = self.health_checks.read().await;
        let metrics = self.current_metrics.read().await.clone();

        for (name, check) in checks.iter() {
            match &check.status {
                HealthStatus::Critical(msg) => {
                    self.send_alert(HealthAlert {
                        timestamp: Utc::now(),
                        severity: AlertSeverity::Critical,
                        component: name.clone(),
                        message: msg.clone(),
                        metrics: Some(metrics.clone()),
                    })
                    .await;
                }
                HealthStatus::Failed(msg) => {
                    self.send_alert(HealthAlert {
                        timestamp: Utc::now(),
                        severity: AlertSeverity::Fatal,
                        component: name.clone(),
                        message: msg.clone(),
                        metrics: Some(metrics.clone()),
                    })
                    .await;
                }
                HealthStatus::Degraded(msg) => {
                    debug!("Component {} degraded: {}", name, msg);
                }
                HealthStatus::Healthy => {}
            }
        }
    }

    async fn send_alert(&self, alert: HealthAlert) {
        warn!("Health Alert: {:?}", alert);

        let callbacks = self.alert_callbacks.read().await;
        for callback in callbacks.iter() {
            callback(alert.clone());
        }
    }

    async fn cleanup_history(&self) {
        let mut history = self.metrics_history.write().await;

        // Keep only last 24 hours of metrics
        let cutoff = Utc::now() - Duration::hours(24);
        history.retain(|m| m.timestamp > cutoff);
    }

    pub async fn get_current_health(&self) -> HealthSummary {
        let metrics = self.current_metrics.read().await.clone();
        let checks = self.health_checks.read().await.clone();

        let overall_status = self.calculate_overall_status(&checks);

        HealthSummary {
            status: overall_status,
            metrics,
            checks: checks.into_values().collect(),
            timestamp: Utc::now(),
        }
    }

    fn calculate_overall_status(&self, checks: &HashMap<String, HealthCheck>) -> HealthStatus {
        let mut has_critical = false;
        let mut has_failed = false;
        let mut has_degraded = false;

        for check in checks.values() {
            match &check.status {
                HealthStatus::Failed(_) => has_failed = true,
                HealthStatus::Critical(_) => has_critical = true,
                HealthStatus::Degraded(_) => has_degraded = true,
                HealthStatus::Healthy => {}
            }
        }

        if has_failed {
            HealthStatus::Failed("System failure detected".to_string())
        } else if has_critical {
            HealthStatus::Critical("Critical issues detected".to_string())
        } else if has_degraded {
            HealthStatus::Degraded("System degraded".to_string())
        } else {
            HealthStatus::Healthy
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub status: HealthStatus,
    pub metrics: HealthMetrics,
    pub checks: Vec<HealthCheck>,
    pub timestamp: DateTime<Utc>,
}
