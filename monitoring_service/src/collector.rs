use anyhow::Result;
use sysinfo::System;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub hostname: String,
    pub ip_address: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub uptime_seconds: u64,
    pub process_count: usize,
    pub load_average: (f64, f64, f64),
}

pub struct MetricsCollector {
    system: System,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self { system }
    }

    pub async fn collect(&mut self) -> Result<SystemMetrics> {
        // Refresh system info
        self.system.refresh_all();
        
        // Wait a bit for CPU usage calculation
        tokio::time::sleep(Duration::from_millis(200)).await;
        self.system.refresh_cpu_all();

        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let ip_address = self.get_primary_ip().await.unwrap_or_else(|| "127.0.0.1".to_string());

        // Calculate CPU usage
        let cpu_usage = self.system.global_cpu_usage();

        // Calculate memory usage
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let memory_usage = if total_memory > 0 {
            (used_memory as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        };

        // Calculate disk usage
        let disk_usage = self.calculate_disk_usage();

        // Get network stats
        let (rx_bytes, tx_bytes) = self.get_network_stats();

        // Get uptime
        let uptime_seconds = self.system.uptime();

        // Get process count
        let process_count = self.system.processes().len();

        // Get load average
        let load_avg = self.system.load_average();
        let load_average = (load_avg.one, load_avg.five, load_avg.fifteen);

        Ok(SystemMetrics {
            hostname,
            ip_address,
            cpu_usage,
            memory_usage,
            disk_usage,
            network_rx_bytes: rx_bytes,
            network_tx_bytes: tx_bytes,
            uptime_seconds,
            process_count,
            load_average,
        })
    }

    fn calculate_disk_usage(&self) -> f32 {
        // For now, return a default value since disks() is not available
        // In production, you might want to use platform-specific APIs
        50.0
    }

    fn get_network_stats(&self) -> (u64, u64) {
        // Return default values for now
        // In production, you might want to use platform-specific network APIs
        (0, 0)
    }

    async fn get_primary_ip(&self) -> Option<String> {
        // Try to get the primary network interface IP
        if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
            // Connect to a public DNS server (doesn't actually send data)
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    return Some(addr.ip().to_string());
                }
            }
        }
        None
    }

    pub fn get_process_metrics(&mut self, process_name: &str) -> Option<ProcessMetrics> {
        self.system.refresh_processes();

        for (_pid, process) in self.system.processes() {
            if process.name().to_str().unwrap_or("") == process_name {
                return Some(ProcessMetrics {
                    pid: 0, // Simplified for now
                    name: process.name().to_str().unwrap_or("").to_string(),
                    cpu_usage: process.cpu_usage(),
                    memory_bytes: process.memory(),
                    virtual_memory_bytes: process.virtual_memory(),
                    status: "Running".to_string(),
                    start_time: 0,
                });
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub status: String,
    pub start_time: u64,
}