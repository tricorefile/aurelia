use crate::{
    decision_maker::{
        AutonomousDecisionMaker, Decision, DecisionContext, NodeInfo, NodeStatus, ResourceMetrics,
    },
    health_monitor::{HealthMonitor, HealthStatus},
    recovery_manager::{FailureEvent, FailureType, RecoveryManager},
    self_replicator::{ReplicationTarget, SelfReplicator},
    task_scheduler::{
        HealthCheckExecutor, ReplicationExecutor, Task, TaskScheduler, TaskStatus, TaskType,
    },
};
use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// The main autonomous agent that coordinates all self-management capabilities
pub struct AutonomousAgent {
    decision_maker: Arc<RwLock<AutonomousDecisionMaker>>,
    health_monitor: Arc<HealthMonitor>,
    recovery_manager: Arc<RecoveryManager>,
    self_replicator: Arc<SelfReplicator>,
    task_scheduler: Arc<TaskScheduler>,
    is_running: Arc<RwLock<bool>>,
}

impl AutonomousAgent {
    pub fn new(binary_path: PathBuf) -> Self {
        let decision_maker = Arc::new(RwLock::new(AutonomousDecisionMaker::new()));
        let health_monitor = Arc::new(HealthMonitor::new());
        let recovery_manager = Arc::new(RecoveryManager::new());
        let self_replicator = Arc::new(SelfReplicator::new(binary_path));
        let task_scheduler = Arc::new(TaskScheduler::new());

        Self {
            decision_maker,
            health_monitor,
            recovery_manager,
            self_replicator,
            task_scheduler,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing Autonomous Agent");

        // Register task executors
        self.task_scheduler
            .register_executor(TaskType::HealthCheck, Box::new(HealthCheckExecutor))
            .await;

        self.task_scheduler
            .register_executor(TaskType::Replication, Box::new(ReplicationExecutor))
            .await;

        // Schedule initial tasks
        self.schedule_core_tasks().await?;

        // Add initial replication targets
        self.configure_replication_targets().await?;

        info!("Autonomous Agent initialized successfully");
        Ok(())
    }

    async fn schedule_core_tasks(&self) -> Result<()> {
        // Schedule recurring health checks
        let health_check_task = Task {
            id: "health-check-recurring".to_string(),
            name: "System Health Check".to_string(),
            task_type: TaskType::HealthCheck,
            priority: 8,
            scheduled_time: Utc::now(),
            dependencies: vec![],
            max_retries: 3,
            retry_count: 0,
            timeout_seconds: 30,
            status: TaskStatus::Pending,
            result: None,
        };

        self.task_scheduler
            .schedule_recurring_task(
                health_check_task,
                chrono::Duration::minutes(5),
                288, // 24 hours worth of 5-minute intervals
            )
            .await?;

        // Schedule periodic replication checks
        let replication_task = Task {
            id: "replication-check".to_string(),
            name: "Replication Status Check".to_string(),
            task_type: TaskType::Replication,
            priority: 6,
            scheduled_time: Utc::now() + chrono::Duration::minutes(10),
            dependencies: vec![],
            max_retries: 2,
            retry_count: 0,
            timeout_seconds: 60,
            status: TaskStatus::Pending,
            result: None,
        };

        self.task_scheduler
            .schedule_recurring_task(replication_task, chrono::Duration::hours(1), 24)
            .await?;

        Ok(())
    }

    async fn configure_replication_targets(&self) -> Result<()> {
        // Add default replication targets
        // In production, these would come from configuration
        let targets = vec![
            ReplicationTarget {
                ip: "192.168.1.101".to_string(),
                user: "ubuntu".to_string(),
                ssh_key_path: PathBuf::from("~/.ssh/id_rsa"),
                remote_path: PathBuf::from("/home/ubuntu/aurelia_replica1"),
                priority: 1,
                last_attempt: None,
                success_count: 0,
                failure_count: 0,
            },
            ReplicationTarget {
                ip: "192.168.1.102".to_string(),
                user: "ubuntu".to_string(),
                ssh_key_path: PathBuf::from("~/.ssh/id_rsa"),
                remote_path: PathBuf::from("/home/ubuntu/aurelia_replica2"),
                priority: 2,
                last_attempt: None,
                success_count: 0,
                failure_count: 0,
            },
        ];

        for target in targets {
            self.self_replicator.add_target(target).await;
        }

        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting Autonomous Agent");
        *self.is_running.write().await = true;

        // Start all autonomous subsystems
        let health_monitor = self.health_monitor.clone();
        let health_handle = tokio::spawn(async move {
            health_monitor.start_monitoring().await;
        });

        let recovery_manager = self.recovery_manager.clone();
        let recovery_handle = tokio::spawn(async move {
            recovery_manager.auto_recover().await;
        });

        let self_replicator = self.self_replicator.clone();
        let replication_handle = tokio::spawn(async move {
            self_replicator.auto_manage().await;
        });

        let task_scheduler = self.task_scheduler.clone();
        let scheduler_handle = tokio::spawn(async move {
            task_scheduler.run().await;
        });

        // Main decision loop
        let decision_loop_handle = tokio::spawn({
            let is_running = self.is_running.clone();
            let decision_maker = self.decision_maker.clone();
            let health_monitor = self.health_monitor.clone();
            let self_replicator = self.self_replicator.clone();
            let recovery_manager = self.recovery_manager.clone();

            async move {
                while *is_running.read().await {
                    // Gather context
                    let context = Self::gather_context(&health_monitor).await;

                    // Make decision
                    let decision = {
                        let mut dm = decision_maker.write().await;
                        match dm.make_decision(&context).await {
                            Ok(d) => d,
                            Err(e) => {
                                error!("Decision making failed: {}", e);
                                Decision::Wait {
                                    duration_seconds: 30,
                                }
                            }
                        }
                    };

                    // Execute decision
                    Self::execute_decision(decision, &self_replicator, &recovery_manager).await;

                    // Wait before next decision cycle
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                }
            }
        });

        // Wait for all tasks
        tokio::select! {
            _ = health_handle => warn!("Health monitor stopped"),
            _ = recovery_handle => warn!("Recovery manager stopped"),
            _ = replication_handle => warn!("Self replicator stopped"),
            _ = scheduler_handle => warn!("Task scheduler stopped"),
            _ = decision_loop_handle => warn!("Decision loop stopped"),
        }

        info!("Autonomous Agent stopped");
        Ok(())
    }

    async fn gather_context(health_monitor: &Arc<HealthMonitor>) -> DecisionContext {
        let health_summary = health_monitor.get_current_health().await;

        let system_health = match health_summary.status {
            HealthStatus::Healthy => 1.0,
            HealthStatus::Degraded(_) => 0.7,
            HealthStatus::Critical(_) => 0.4,
            HealthStatus::Failed(_) => 0.1,
        };

        let metrics = &health_summary.metrics;

        DecisionContext {
            timestamp: Utc::now(),
            system_health,
            resource_usage: ResourceMetrics {
                cpu_percent: metrics.cpu_usage,
                memory_mb: metrics.memory_usage,
                disk_gb: metrics.disk_usage,
                network_mbps: 0.0,
            },
            active_nodes: vec![NodeInfo {
                id: "primary".to_string(),
                ip: "127.0.0.1".to_string(),
                status: NodeStatus::Healthy,
                last_seen: Utc::now(),
                load: metrics.cpu_usage / 100.0,
            }],
            failed_nodes: vec![],
            pending_tasks: 0,
            market_conditions: None,
        }
    }

    async fn execute_decision(
        decision: Decision,
        self_replicator: &Arc<SelfReplicator>,
        recovery_manager: &Arc<RecoveryManager>,
    ) {
        info!("Executing decision: {:?}", decision);

        match decision {
            Decision::Deploy {
                target_servers,
                priority,
                reason,
            } => {
                info!("Deploying to {} servers: {}", target_servers.len(), reason);

                for server_ip in target_servers {
                    let target = ReplicationTarget {
                        ip: server_ip.clone(),
                        user: "ubuntu".to_string(),
                        ssh_key_path: PathBuf::from("~/.ssh/id_rsa"),
                        remote_path: PathBuf::from(format!(
                            "/home/ubuntu/aurelia_{}",
                            server_ip.replace(".", "_")
                        )),
                        priority: match priority {
                            crate::decision_maker::Priority::Critical => 1,
                            crate::decision_maker::Priority::High => 2,
                            crate::decision_maker::Priority::Normal => 3,
                            crate::decision_maker::Priority::Low => 4,
                        },
                        last_attempt: None,
                        success_count: 0,
                        failure_count: 0,
                    };

                    self_replicator.add_target(target).await;
                }

                if let Err(e) = self_replicator.replicate().await {
                    error!("Replication failed: {}", e);
                }
            }

            Decision::Scale { factor, reason } => {
                info!("Scaling by factor {}: {}", factor, reason);

                // Trigger scaling operations
                let replicas_needed = (factor * 2.0) as usize;
                for _ in 0..replicas_needed {
                    if let Err(e) = self_replicator.replicate().await {
                        error!("Scaling replication failed: {}", e);
                    }
                }
            }

            Decision::Recover {
                failed_node,
                recovery_action,
            } => {
                info!(
                    "Recovering failed node {}: {:?}",
                    failed_node, recovery_action
                );

                let failure = FailureEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    failure_type: FailureType::ProcessCrash,
                    component: failed_node,
                    description: "Node failure detected".to_string(),
                    severity: 7,
                    auto_recoverable: true,
                };

                if let Err(e) = recovery_manager.handle_failure(failure).await {
                    error!("Recovery failed: {}", e);
                }
            }

            Decision::Monitor { interval_seconds } => {
                debug!("Monitoring with interval {} seconds", interval_seconds);
            }

            Decision::Wait { duration_seconds } => {
                debug!("Waiting for {} seconds", duration_seconds);
            }

            _ => {
                warn!("Unhandled decision type: {:?}", decision);
            }
        }
    }

    pub async fn stop(&self) {
        info!("Stopping Autonomous Agent");
        *self.is_running.write().await = false;
    }

    pub async fn get_status(&self) -> AgentStatus {
        let health_summary = self.health_monitor.get_current_health().await;
        let replication_status = self.self_replicator.get_status().await;
        let recovery_stats = self.recovery_manager.get_recovery_stats().await;
        let scheduler_status = self.task_scheduler.get_status().await;

        AgentStatus {
            is_running: *self.is_running.read().await,
            health_status: format!("{:?}", health_summary.status),
            active_replicas: replication_status.active_replicas,
            pending_tasks: scheduler_status.pending_tasks,
            running_tasks: scheduler_status.running_tasks,
            recovery_success_rate: recovery_stats.success_rate,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentStatus {
    pub is_running: bool,
    pub health_status: String,
    pub active_replicas: usize,
    pub pending_tasks: usize,
    pub running_tasks: usize,
    pub recovery_success_rate: f64,
}
