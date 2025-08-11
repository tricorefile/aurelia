use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub timestamp: DateTime<Utc>,
    pub period_minutes: u32,
    pub agent_count: usize,
    pub avg_cpu_usage: f32,
    pub max_cpu_usage: f32,
    pub min_cpu_usage: f32,
    pub avg_memory_usage: f32,
    pub max_memory_usage: f32,
    pub min_memory_usage: f32,
    pub total_tasks_completed: u32,
    pub total_tasks_failed: u32,
    pub task_success_rate: f32,
    pub total_network_rx_gb: f64,
    pub total_network_tx_gb: f64,
    pub availability_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    pub timestamps: Vec<DateTime<Utc>>,
    pub cpu_usage: Vec<f32>,
    pub memory_usage: Vec<f32>,
    pub task_throughput: Vec<u32>,
    pub network_throughput: Vec<f64>,
}

pub struct MetricsAggregator {
    history: Vec<AggregatedMetrics>,
    time_series: HashMap<String, TimeSeriesData>,
    retention_days: u32,
}

impl MetricsAggregator {
    pub fn new(retention_days: u32) -> Self {
        Self {
            history: Vec::new(),
            time_series: HashMap::new(),
            retention_days,
        }
    }

    pub fn aggregate(
        &mut self,
        agents: &[crate::AgentStatus],
        period_minutes: u32,
    ) -> AggregatedMetrics {
        let timestamp = Utc::now();
        
        if agents.is_empty() {
            return AggregatedMetrics {
                timestamp,
                period_minutes,
                agent_count: 0,
                avg_cpu_usage: 0.0,
                max_cpu_usage: 0.0,
                min_cpu_usage: 0.0,
                avg_memory_usage: 0.0,
                max_memory_usage: 0.0,
                min_memory_usage: 0.0,
                total_tasks_completed: 0,
                total_tasks_failed: 0,
                task_success_rate: 0.0,
                total_network_rx_gb: 0.0,
                total_network_tx_gb: 0.0,
                availability_percentage: 0.0,
            };
        }

        // Calculate CPU metrics
        let cpu_values: Vec<f32> = agents.iter().map(|a| a.cpu_usage).collect();
        let avg_cpu = cpu_values.iter().sum::<f32>() / cpu_values.len() as f32;
        let max_cpu = cpu_values.iter().cloned().fold(0.0, f32::max);
        let min_cpu = cpu_values.iter().cloned().fold(100.0, f32::min);

        // Calculate memory metrics
        let memory_values: Vec<f32> = agents.iter().map(|a| a.memory_usage).collect();
        let avg_memory = memory_values.iter().sum::<f32>() / memory_values.len() as f32;
        let max_memory = memory_values.iter().cloned().fold(0.0, f32::max);
        let min_memory = memory_values.iter().cloned().fold(100.0, f32::min);

        // Calculate task metrics
        let total_completed: u32 = agents.iter().map(|a| a.tasks_completed).sum();
        let total_failed: u32 = agents.iter().map(|a| a.tasks_failed).sum();
        let task_success_rate = if total_completed + total_failed > 0 {
            (total_completed as f32 / (total_completed + total_failed) as f32) * 100.0
        } else {
            0.0
        };

        // Calculate network metrics (convert to GB)
        let total_rx_gb: f64 = agents.iter()
            .map(|a| a.network_rx_bytes as f64 / 1_073_741_824.0)
            .sum();
        let total_tx_gb: f64 = agents.iter()
            .map(|a| a.network_tx_bytes as f64 / 1_073_741_824.0)
            .sum();

        // Calculate availability
        let cutoff = timestamp - Duration::minutes(period_minutes as i64);
        let available_agents = agents.iter()
            .filter(|a| a.last_heartbeat > cutoff)
            .count();
        let availability = (available_agents as f32 / agents.len() as f32) * 100.0;

        let metrics = AggregatedMetrics {
            timestamp,
            period_minutes,
            agent_count: agents.len(),
            avg_cpu_usage: avg_cpu,
            max_cpu_usage: max_cpu,
            min_cpu_usage: min_cpu,
            avg_memory_usage: avg_memory,
            max_memory_usage: max_memory,
            min_memory_usage: min_memory,
            total_tasks_completed: total_completed,
            total_tasks_failed: total_failed,
            task_success_rate,
            total_network_rx_gb: total_rx_gb,
            total_network_tx_gb: total_tx_gb,
            availability_percentage: availability,
        };

        // Store in history
        self.history.push(metrics.clone());
        self.cleanup_old_data();

        // Update time series for each agent
        for agent in agents {
            self.update_time_series(&agent.agent_id, &agent);
        }

        metrics
    }

    fn update_time_series(&mut self, agent_id: &str, agent: &crate::AgentStatus) {
        let entry = self.time_series.entry(agent_id.to_string()).or_insert_with(|| {
            TimeSeriesData {
                timestamps: Vec::new(),
                cpu_usage: Vec::new(),
                memory_usage: Vec::new(),
                task_throughput: Vec::new(),
                network_throughput: Vec::new(),
            }
        });

        entry.timestamps.push(Utc::now());
        entry.cpu_usage.push(agent.cpu_usage);
        entry.memory_usage.push(agent.memory_usage);
        entry.task_throughput.push(agent.tasks_completed);
        
        let network_throughput = (agent.network_rx_bytes + agent.network_tx_bytes) as f64 / 1_048_576.0; // MB/s
        entry.network_throughput.push(network_throughput);

        // Keep only last 1000 points
        if entry.timestamps.len() > 1000 {
            entry.timestamps.drain(0..100);
            entry.cpu_usage.drain(0..100);
            entry.memory_usage.drain(0..100);
            entry.task_throughput.drain(0..100);
            entry.network_throughput.drain(0..100);
        }
    }

    fn cleanup_old_data(&mut self) {
        let cutoff = Utc::now() - Duration::days(self.retention_days as i64);
        self.history.retain(|m| m.timestamp > cutoff);
    }

    pub fn get_history(&self, hours: u32) -> Vec<AggregatedMetrics> {
        let cutoff = Utc::now() - Duration::hours(hours as i64);
        self.history.iter()
            .filter(|m| m.timestamp > cutoff)
            .cloned()
            .collect()
    }

    pub fn get_time_series(&self, agent_id: &str) -> Option<&TimeSeriesData> {
        self.time_series.get(agent_id)
    }

    pub fn get_summary_stats(&self, hours: u32) -> SummaryStatistics {
        let history = self.get_history(hours);
        
        if history.is_empty() {
            return SummaryStatistics::default();
        }

        let avg_cpu: f32 = history.iter().map(|m| m.avg_cpu_usage).sum::<f32>() / history.len() as f32;
        let avg_memory: f32 = history.iter().map(|m| m.avg_memory_usage).sum::<f32>() / history.len() as f32;
        let total_tasks: u32 = history.iter().map(|m| m.total_tasks_completed).sum();
        let avg_availability: f32 = history.iter().map(|m| m.availability_percentage).sum::<f32>() / history.len() as f32;
        
        SummaryStatistics {
            period_hours: hours,
            avg_cpu_usage: avg_cpu,
            avg_memory_usage: avg_memory,
            total_tasks_completed: total_tasks,
            avg_availability: avg_availability,
            peak_cpu_usage: history.iter().map(|m| m.max_cpu_usage).fold(0.0, f32::max),
            peak_memory_usage: history.iter().map(|m| m.max_memory_usage).fold(0.0, f32::max),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SummaryStatistics {
    pub period_hours: u32,
    pub avg_cpu_usage: f32,
    pub avg_memory_usage: f32,
    pub total_tasks_completed: u32,
    pub avg_availability: f32,
    pub peak_cpu_usage: f32,
    pub peak_memory_usage: f32,
}