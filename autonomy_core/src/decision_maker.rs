use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    Deploy {
        target_servers: Vec<String>,
        priority: Priority,
        reason: String,
    },
    Scale {
        factor: f64,
        reason: String,
    },
    Migrate {
        from: String,
        to: String,
        reason: String,
    },
    Recover {
        failed_node: String,
        recovery_action: RecoveryAction,
    },
    Monitor {
        interval_seconds: u64,
    },
    Wait {
        duration_seconds: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    Restart,
    Redeploy,
    Failover,
    Ignore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub timestamp: DateTime<Utc>,
    pub system_health: f64,
    pub resource_usage: ResourceMetrics,
    pub active_nodes: Vec<NodeInfo>,
    pub failed_nodes: Vec<NodeInfo>,
    pub pending_tasks: usize,
    pub market_conditions: Option<MarketConditions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub disk_gb: f64,
    pub network_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub ip: String,
    pub status: NodeStatus,
    pub last_seen: DateTime<Utc>,
    pub load: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Healthy,
    Degraded,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditions {
    pub volatility: f64,
    pub opportunity_score: f64,
    pub risk_level: f64,
}

pub struct AutonomousDecisionMaker {
    thresholds: DecisionThresholds,
    decision_history: Vec<(DateTime<Utc>, Decision)>,
    learning_rate: f64,
}

#[derive(Debug, Clone)]
struct DecisionThresholds {
    min_health_for_expansion: f64,
    max_cpu_before_scaling: f64,
    max_memory_before_scaling: f64,
    #[allow(dead_code)]
    min_nodes_required: usize,
    max_nodes_allowed: usize,
    failure_tolerance: f64,
}

impl Default for DecisionThresholds {
    fn default() -> Self {
        Self {
            min_health_for_expansion: 0.8,
            max_cpu_before_scaling: 75.0,
            max_memory_before_scaling: 80.0,
            min_nodes_required: 1,
            max_nodes_allowed: 10,
            failure_tolerance: 0.3,
        }
    }
}

impl Default for AutonomousDecisionMaker {
    fn default() -> Self {
        Self::new()
    }
}

impl AutonomousDecisionMaker {
    pub fn new() -> Self {
        Self {
            thresholds: DecisionThresholds::default(),
            decision_history: Vec::new(),
            learning_rate: 0.1,
        }
    }

    pub async fn make_decision(&mut self, context: &DecisionContext) -> Result<Decision> {
        info!("Making autonomous decision based on current context");

        // 1. Check for critical failures first
        if let Some(decision) = self.check_failures(context) {
            self.record_decision(decision.clone());
            return Ok(decision);
        }

        // 2. Check resource pressure
        if let Some(decision) = self.check_resource_pressure(context) {
            self.record_decision(decision.clone());
            return Ok(decision);
        }

        // 3. Check for expansion opportunities
        if let Some(decision) = self.check_expansion_opportunity(context) {
            self.record_decision(decision.clone());
            return Ok(decision);
        }

        // 4. Check market conditions for trading decisions
        if let Some(decision) = self.check_market_conditions(context) {
            self.record_decision(decision.clone());
            return Ok(decision);
        }

        // 5. Default to monitoring
        let decision = Decision::Monitor {
            interval_seconds: 30,
        };
        self.record_decision(decision.clone());
        Ok(decision)
    }

    fn check_failures(&self, context: &DecisionContext) -> Option<Decision> {
        if !context.failed_nodes.is_empty() {
            let failed_node = &context.failed_nodes[0];
            warn!("Detected failed node: {}", failed_node.id);

            // Decide recovery action based on failure rate
            let failure_rate = context.failed_nodes.len() as f64
                / (context.active_nodes.len() + context.failed_nodes.len()) as f64;

            let recovery_action = if failure_rate > self.thresholds.failure_tolerance {
                RecoveryAction::Failover
            } else {
                RecoveryAction::Redeploy
            };

            return Some(Decision::Recover {
                failed_node: failed_node.id.clone(),
                recovery_action,
            });
        }
        None
    }

    fn check_resource_pressure(&self, context: &DecisionContext) -> Option<Decision> {
        let metrics = &context.resource_usage;

        if metrics.cpu_percent > self.thresholds.max_cpu_before_scaling
            || metrics.memory_mb > self.thresholds.max_memory_before_scaling
        {
            info!(
                "Resource pressure detected: CPU {}%, Memory {}MB",
                metrics.cpu_percent, metrics.memory_mb
            );

            // Calculate scaling factor
            let cpu_pressure = metrics.cpu_percent / self.thresholds.max_cpu_before_scaling;
            let mem_pressure = metrics.memory_mb / self.thresholds.max_memory_before_scaling;
            let scale_factor = f64::max(cpu_pressure, mem_pressure).min(2.0);

            return Some(Decision::Scale {
                factor: scale_factor,
                reason: format!(
                    "High resource usage: CPU {:.1}%, Memory {:.0}MB",
                    metrics.cpu_percent, metrics.memory_mb
                ),
            });
        }
        None
    }

    fn check_expansion_opportunity(&self, context: &DecisionContext) -> Option<Decision> {
        if context.system_health >= self.thresholds.min_health_for_expansion
            && context.active_nodes.len() < self.thresholds.max_nodes_allowed
        {
            // Identify potential target servers for expansion
            let target_servers = self.identify_expansion_targets(context);

            if !target_servers.is_empty() {
                info!("Expansion opportunity identified");
                return Some(Decision::Deploy {
                    target_servers,
                    priority: Priority::Normal,
                    reason: format!(
                        "System health good ({:.1}%), expanding network",
                        context.system_health * 100.0
                    ),
                });
            }
        }
        None
    }

    fn check_market_conditions(&self, context: &DecisionContext) -> Option<Decision> {
        if let Some(market) = &context.market_conditions {
            if market.opportunity_score > 0.7 && market.risk_level < 0.3 {
                debug!("Favorable market conditions detected");
                // Market conditions are good but we return None
                // as trading decisions are handled by strategy engine
            }
        }
        None
    }

    fn identify_expansion_targets(&self, _context: &DecisionContext) -> Vec<String> {
        // In a real implementation, this would analyze network topology,
        // geographic distribution, and available resources
        vec!["192.168.1.102".to_string(), "192.168.1.103".to_string()]
    }

    fn record_decision(&mut self, decision: Decision) {
        self.decision_history.push((Utc::now(), decision));

        // Keep only recent history
        if self.decision_history.len() > 1000 {
            self.decision_history.drain(0..100);
        }
    }

    pub fn adjust_thresholds(&mut self, feedback: &DecisionFeedback) {
        // Simple learning: adjust thresholds based on outcome
        match feedback.outcome {
            Outcome::Success => {
                // Slightly relax thresholds on success
                self.thresholds.min_health_for_expansion *= 1.0 - self.learning_rate * 0.1;
                self.thresholds.max_cpu_before_scaling *= 1.0 + self.learning_rate * 0.05;
            }
            Outcome::Failure => {
                // Tighten thresholds on failure
                self.thresholds.min_health_for_expansion *= 1.0 + self.learning_rate * 0.1;
                self.thresholds.max_cpu_before_scaling *= 1.0 - self.learning_rate * 0.05;
            }
            Outcome::Neutral => {
                // No adjustment
            }
        }

        // Ensure thresholds stay within reasonable bounds
        self.thresholds.min_health_for_expansion =
            self.thresholds.min_health_for_expansion.clamp(0.5, 0.95);
        self.thresholds.max_cpu_before_scaling =
            self.thresholds.max_cpu_before_scaling.clamp(50.0, 90.0);
    }

    pub fn get_decision_history(&self) -> &[(DateTime<Utc>, Decision)] {
        &self.decision_history
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionFeedback {
    pub decision_id: String,
    pub outcome: Outcome,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Outcome {
    Success,
    Failure,
    Neutral,
}
