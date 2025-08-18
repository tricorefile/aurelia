pub mod http_server;
pub mod simple_server;

pub use http_server::{
    AgentStatus, ClusterStatus, MonitoringHttpService, SystemMetrics, TradingStatus,
};
pub use simple_server::SimpleAgentStatus;
use simple_server::SimpleMonitoringService;

#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub port: u16,
    pub use_http: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            use_http: true,
        }
    }
}

pub struct MonitoringService {
    inner: SimpleMonitoringService,
    http_service: Option<MonitoringHttpService>,
    config: MonitoringConfig,
}

impl MonitoringService {
    pub fn new(config: MonitoringConfig) -> Self {
        let http_service = if config.use_http {
            Some(MonitoringHttpService::new(config.port))
        } else {
            None
        };

        Self {
            inner: SimpleMonitoringService::new(config.port),
            http_service,
            config,
        }
    }

    pub async fn start(self: std::sync::Arc<Self>) -> anyhow::Result<()> {
        if self.config.use_http {
            if let Some(http_service) = &self.http_service {
                // 启动HTTP服务器
                http_service.clone().start_server();
                println!("✅ Rust监控API已启动在端口 {}", self.config.port);
            }
        } else {
            // 使用简单监控服务
            let inner = std::sync::Arc::new(self.inner.clone());
            inner.start().await?;
        }
        Ok(())
    }

    pub fn get_http_service(&self) -> Option<&MonitoringHttpService> {
        self.http_service.as_ref()
    }
}
