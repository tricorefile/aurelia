use crate::config::ServerConfig;
use crate::deployer::DeploymentClient;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringResult {
    pub server_name: String,
    pub timestamp: DateTime<Utc>,
    pub is_running: bool,
    pub cpu_usage: Option<f64>,
    pub memory_mb: Option<f64>,
    pub log_activity: bool,
    pub recent_logs: Vec<String>,
    pub websocket_connections: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub disk_usage_gb: f64,
}

pub struct AgentMonitor {
    client: DeploymentClient,
    config: ServerConfig,
}

impl AgentMonitor {
    pub fn new(config: ServerConfig) -> Self {
        let client = DeploymentClient::new(config.clone());
        Self { client, config }
    }

    pub fn check_agent_health(&self) -> Result<MonitoringResult> {
        let mut result = MonitoringResult {
            server_name: self.config.name.clone(),
            timestamp: Utc::now(),
            is_running: false,
            cpu_usage: None,
            memory_mb: None,
            log_activity: false,
            recent_logs: Vec::new(),
            websocket_connections: 0,
            errors: Vec::new(),
        };

        // Check if agent process is running
        match self.check_process_status() {
            Ok(running) => result.is_running = running,
            Err(e) => result.errors.push(format!("Process check failed: {}", e)),
        }

        // Get resource metrics
        match self.get_resource_metrics() {
            Ok(metrics) => {
                result.cpu_usage = Some(metrics.cpu_percent);
                result.memory_mb = Some(metrics.memory_mb);
            }
            Err(e) => result.errors.push(format!("Resource metrics failed: {}", e)),
        }

        // Check log activity
        match self.check_log_activity() {
            Ok((active, logs)) => {
                result.log_activity = active;
                result.recent_logs = logs;
            }
            Err(e) => result.errors.push(format!("Log check failed: {}", e)),
        }

        // Check network connections
        match self.check_network_connections() {
            Ok(count) => result.websocket_connections = count,
            Err(e) => result.errors.push(format!("Network check failed: {}", e)),
        }

        Ok(result)
    }

    pub fn check_process_status(&self) -> Result<bool> {
        let sess = self.client.connect()?;
        let cmd = format!(
            "cd {:?} && if [ -f aurelia.pid ]; then ps -p $(cat aurelia.pid) > /dev/null 2>&1 && echo 'RUNNING' || echo 'STOPPED'; else echo 'NO_PID'; fi",
            self.config.remote_deploy_path
        );
        
        let output = self.client.execute_command(&sess, &cmd)?;
        Ok(output.contains("RUNNING"))
    }

    pub fn get_resource_metrics(&self) -> Result<ResourceMetrics> {
        let sess = self.client.connect()?;
        
        // Get CPU usage
        let cpu_cmd = "top -bn1 | grep 'Cpu(s)' | awk '{print $2}' | cut -d'%' -f1";
        let cpu_output = self.client.execute_command(&sess, cpu_cmd)?;
        let cpu_percent = cpu_output.trim().parse::<f64>().unwrap_or(0.0);
        
        // Get memory usage for the kernel process
        let mem_cmd = format!(
            "cd {:?} && if [ -f aurelia.pid ]; then ps aux | grep $(cat aurelia.pid) | grep -v grep | awk '{{print $6}}'; else echo '0'; fi",
            self.config.remote_deploy_path
        );
        let mem_output = self.client.execute_command(&sess, &mem_cmd)?;
        let memory_kb = mem_output.trim().parse::<f64>().unwrap_or(0.0);
        let memory_mb = memory_kb / 1024.0;
        
        // Get disk usage
        let disk_cmd = format!(
            "du -sh {:?} | awk '{{print $1}}'",
            self.config.remote_deploy_path
        );
        let disk_output = self.client.execute_command(&sess, &disk_cmd)?;
        let disk_usage_gb = self.parse_disk_usage(&disk_output);
        
        Ok(ResourceMetrics {
            cpu_percent,
            memory_mb,
            disk_usage_gb,
        })
    }

    pub fn check_log_activity(&self) -> Result<(bool, Vec<String>)> {
        let sess = self.client.connect()?;
        let cmd = format!(
            "cd {:?} && if [ -f aurelia.log ]; then tail -n 50 aurelia.log; else echo 'NO_LOG'; fi",
            self.config.remote_deploy_path
        );
        
        let output = self.client.execute_command(&sess, &cmd)?;
        
        if output.contains("NO_LOG") {
            return Ok((false, Vec::new()));
        }
        
        let lines: Vec<String> = output.lines()
            .take(20)
            .map(|s| s.to_string())
            .collect();
        
        let has_activity = lines.iter()
            .any(|line| line.contains("[INFO]") || line.contains("[WARN]") || line.contains("[ERROR]"));
        
        Ok((has_activity, lines))
    }

    pub fn check_network_connections(&self) -> Result<usize> {
        let sess = self.client.connect()?;
        let cmd = "netstat -an | grep ':8080' | grep ESTABLISHED | wc -l";
        let output = self.client.execute_command(&sess, &cmd)?;
        let count = output.trim().parse::<usize>().unwrap_or(0);
        Ok(count)
    }

    fn parse_disk_usage(&self, output: &str) -> f64 {
        let size_str = output.trim();
        if size_str.ends_with('K') {
            size_str[..size_str.len()-1].parse::<f64>().unwrap_or(0.0) / 1024.0 / 1024.0
        } else if size_str.ends_with('M') {
            size_str[..size_str.len()-1].parse::<f64>().unwrap_or(0.0) / 1024.0
        } else if size_str.ends_with('G') {
            size_str[..size_str.len()-1].parse::<f64>().unwrap_or(0.0)
        } else {
            0.0
        }
    }

    pub fn check_autonomous_behavior(&self) -> Result<HashMap<String, bool>> {
        let sess = self.client.connect()?;
        let mut behaviors = HashMap::new();
        
        // Check for strategy decisions
        let cmd = format!(
            "cd {:?} && grep -E 'StrategyDecision|DECISION' aurelia.log | tail -n 10 | wc -l",
            self.config.remote_deploy_path
        );
        let output = self.client.execute_command(&sess, &cmd)?;
        let decision_count = output.trim().parse::<usize>().unwrap_or(0);
        behaviors.insert("has_decisions".to_string(), decision_count > 0);
        
        // Check for perception events
        let cmd = format!(
            "cd {:?} && grep -E 'MarketData|Perception' aurelia.log | tail -n 10 | wc -l",
            self.config.remote_deploy_path
        );
        let output = self.client.execute_command(&sess, &cmd)?;
        let perception_count = output.trim().parse::<usize>().unwrap_or(0);
        behaviors.insert("has_perception".to_string(), perception_count > 0);
        
        // Check for reasoning events
        let cmd = format!(
            "cd {:?} && grep -E 'Reasoning|Analysis' aurelia.log | tail -n 10 | wc -l",
            self.config.remote_deploy_path
        );
        let output = self.client.execute_command(&sess, &cmd)?;
        let reasoning_count = output.trim().parse::<usize>().unwrap_or(0);
        behaviors.insert("has_reasoning".to_string(), reasoning_count > 0);
        
        // Check for self-replication events
        let cmd = format!(
            "cd {:?} && grep -E 'Deploy|Replication' aurelia.log | tail -n 10 | wc -l",
            self.config.remote_deploy_path
        );
        let output = self.client.execute_command(&sess, &cmd)?;
        let replication_count = output.trim().parse::<usize>().unwrap_or(0);
        behaviors.insert("has_replication".to_string(), replication_count > 0);
        
        Ok(behaviors)
    }

    pub fn get_recent_events(&self, event_type: &str) -> Result<Vec<String>> {
        let sess = self.client.connect()?;
        let cmd = format!(
            "cd {:?} && grep -E '{}' aurelia.log | tail -n 20",
            self.config.remote_deploy_path,
            event_type
        );
        
        let output = self.client.execute_command(&sess, &cmd)?;
        let events: Vec<String> = output.lines()
            .map(|s| s.to_string())
            .collect();
        
        Ok(events)
    }

    pub fn verify_replica_deployment(&self) -> Result<bool> {
        let sess = self.client.connect()?;
        let cmd = format!(
            "test -f {:?}/kernel && echo 'EXISTS' || echo 'NOT_FOUND'",
            self.config.remote_deploy_path
        );
        
        let output = self.client.execute_command(&sess, &cmd)?;
        Ok(output.contains("EXISTS"))
    }
}