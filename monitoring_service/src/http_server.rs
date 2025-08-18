use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub agent_id: String,
    pub hostname: String,
    pub ip_address: String,
    pub status: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub uptime_seconds: u64,
    pub last_heartbeat: DateTime<Utc>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub total_agents: usize,
    pub healthy_agents: usize,
    pub degraded_agents: usize,
    pub offline_agents: usize,
    pub total_cpu_usage: f32,
    pub total_memory_usage: f32,
    pub cluster_health: String,
    pub agents: Vec<AgentStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage_mb: f64,
    pub memory_total_mb: f64,
    pub memory_percentage: f32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStatus {
    pub active: bool,
    pub last_price: HashMap<String, f64>,
    pub total_trades: u32,
    pub successful_trades: u32,
    pub failed_trades: u32,
    pub pnl: f64,
}

#[derive(Clone)]
pub struct MonitoringHttpService {
    pub agents: Arc<RwLock<HashMap<String, AgentStatus>>>,
    pub system_metrics: Arc<RwLock<SystemMetrics>>,
    pub trading_status: Arc<RwLock<TradingStatus>>,
    pub port: u16,
}

impl MonitoringHttpService {
    pub fn new(port: u16) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            system_metrics: Arc::new(RwLock::new(SystemMetrics {
                cpu_usage: 0.0,
                memory_usage_mb: 0.0,
                memory_total_mb: 0.0,
                memory_percentage: 0.0,
                timestamp: Utc::now(),
            })),
            trading_status: Arc::new(RwLock::new(TradingStatus {
                active: false,
                last_price: HashMap::new(),
                total_trades: 0,
                successful_trades: 0,
                failed_trades: 0,
                pnl: 0.0,
            })),
            port,
        }
    }

    pub fn start_server(self) {
        let service_data = web::Data::new(self.clone());

        // 启动后台指标收集
        let metrics_service = self.clone();
        tokio::spawn(async move {
            metrics_service.collect_metrics_loop().await;
        });

        println!("🚀 Rust监控API启动在: http://0.0.0.0:{}", self.port);
        println!("📊 API端点:");
        println!("   GET /");
        println!("   GET /api/status");
        println!("   GET /api/agents");
        println!("   GET /api/cluster/status");
        println!("   GET /api/metrics");
        println!("   GET /api/trading");
        println!("   GET /health");

        let port = self.port;

        // 在独立线程中运行Actix系统
        std::thread::spawn(move || {
            let rt = actix_web::rt::System::new();
            rt.block_on(async move {
                // 尝试初始化env_logger，如果已经初始化则忽略错误
                let _ =
                    env_logger::try_init_from_env(env_logger::Env::new().default_filter_or("info"));
                HttpServer::new(move || {
                    let cors = Cors::default()
                        .allow_any_origin()
                        .allow_any_method()
                        .allow_any_header();

                    App::new()
                        .app_data(service_data.clone())
                        .wrap(cors)
                        .wrap(middleware::Logger::default())
                        .route("/", web::get().to(root_handler))
                        .route("/api/status", web::get().to(get_status))
                        .route("/api/agents", web::get().to(get_agents))
                        .route("/api/cluster/status", web::get().to(get_cluster_status))
                        .route("/api/metrics", web::get().to(get_metrics))
                        .route("/api/trading", web::get().to(get_trading_status))
                        .route("/health", web::get().to(health_check))
                })
                .bind(("0.0.0.0", port))
                .expect("Failed to bind server")
                .run()
                .await
                .expect("Failed to run server");
            });
        });
    }

    async fn collect_metrics_loop(&self) {
        let mut sys = System::new_all();

        loop {
            sys.refresh_all();

            let cpu_usage = sys.global_cpu_info().cpu_usage();
            let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0;
            let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0;
            let memory_percentage = (used_memory / total_memory) * 100.0;

            let metrics = SystemMetrics {
                cpu_usage,
                memory_usage_mb: used_memory,
                memory_total_mb: total_memory,
                memory_percentage: memory_percentage as f32,
                timestamp: Utc::now(),
            };

            *self.system_metrics.write().await = metrics;

            // 更新本地agent状态
            let mut agents = self.agents.write().await;
            let hostname = hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "localhost".to_string());

            agents.insert(
                "local".to_string(),
                AgentStatus {
                    agent_id: "local".to_string(),
                    hostname: hostname.clone(),
                    ip_address: "127.0.0.1".to_string(),
                    status: "Running".to_string(),
                    cpu_usage,
                    memory_usage: memory_percentage as f32,
                    disk_usage: 0.0,
                    uptime_seconds: System::uptime(),
                    last_heartbeat: Utc::now(),
                    version: "0.1.0".to_string(),
                },
            );

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    pub async fn update_trading_status(
        &self,
        active: bool,
        symbol: Option<String>,
        price: Option<f64>,
    ) {
        let mut status = self.trading_status.write().await;
        status.active = active;

        if let (Some(sym), Some(p)) = (symbol, price) {
            status.last_price.insert(sym, p);
        }
    }

    pub async fn record_trade(&self, success: bool) {
        let mut status = self.trading_status.write().await;
        status.total_trades += 1;
        if success {
            status.successful_trades += 1;
        } else {
            status.failed_trades += 1;
        }
    }

    pub async fn update_pnl(&self, pnl: f64) {
        let mut status = self.trading_status.write().await;
        status.pnl = pnl;
    }
}

// Handler functions
async fn root_handler() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "service": "Aurelia Monitoring API",
        "version": "0.1.0",
        "status": "running",
        "endpoints": [
            "/api/status",
            "/api/agents",
            "/api/cluster/status",
            "/api/metrics",
            "/api/trading",
            "/health"
        ]
    })))
}

async fn get_status(service: web::Data<MonitoringHttpService>) -> Result<HttpResponse> {
    let agents = service.agents.read().await;
    let metrics = service.system_metrics.read().await;
    let trading = service.trading_status.read().await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "timestamp": Utc::now(),
        "total_agents": agents.len(),
        "system_metrics": metrics.clone(),
        "trading_active": trading.active,
        "total_trades": trading.total_trades,
    })))
}

async fn get_agents(service: web::Data<MonitoringHttpService>) -> Result<HttpResponse> {
    let agents = service.agents.read().await;
    let agent_list: Vec<AgentStatus> = agents.values().cloned().collect();
    Ok(HttpResponse::Ok().json(agent_list))
}

async fn get_cluster_status(service: web::Data<MonitoringHttpService>) -> Result<HttpResponse> {
    let agents = service.agents.read().await;
    let metrics = service.system_metrics.read().await;

    let healthy = agents.values().filter(|a| a.status == "Running").count();
    let degraded = agents.values().filter(|a| a.status == "Degraded").count();
    let offline = agents.values().filter(|a| a.status == "Offline").count();

    let status = ClusterStatus {
        total_agents: agents.len(),
        healthy_agents: healthy,
        degraded_agents: degraded,
        offline_agents: offline,
        total_cpu_usage: metrics.cpu_usage,
        total_memory_usage: metrics.memory_percentage,
        cluster_health: if healthy == agents.len() {
            "Healthy"
        } else {
            "Degraded"
        }
        .to_string(),
        agents: agents.values().cloned().collect(),
    };

    Ok(HttpResponse::Ok().json(status))
}

async fn get_metrics(service: web::Data<MonitoringHttpService>) -> Result<HttpResponse> {
    let metrics = service.system_metrics.read().await;
    Ok(HttpResponse::Ok().json(metrics.clone()))
}

async fn get_trading_status(service: web::Data<MonitoringHttpService>) -> Result<HttpResponse> {
    let trading = service.trading_status.read().await;
    Ok(HttpResponse::Ok().json(trading.clone()))
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "timestamp": Utc::now(),
    })))
}
