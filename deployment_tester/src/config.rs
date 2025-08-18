use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    #[default]
    Key,
    Password,
    KeyWithPassphrase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub test_environments: Vec<ServerConfig>,
    pub test_settings: TestSettings,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default)]
    pub auth_method: AuthMethod,
    pub remote_deploy_path: PathBuf,
    pub role: ServerRole,
}

impl ServerConfig {
    /// 创建使用密钥认证的配置(兼容旧代码)
    pub fn with_key(
        name: String,
        ip: String,
        port: u16,
        user: String,
        ssh_key_path: PathBuf,
        remote_deploy_path: PathBuf,
        role: ServerRole,
    ) -> Self {
        Self {
            name,
            ip,
            port,
            user,
            ssh_key_path: Some(ssh_key_path),
            password: None,
            auth_method: AuthMethod::Key,
            remote_deploy_path,
            role,
        }
    }

    /// 创建使用密码认证的配置
    pub fn with_password(
        name: String,
        ip: String,
        port: u16,
        user: String,
        password: String,
        remote_deploy_path: PathBuf,
        role: ServerRole,
    ) -> Self {
        Self {
            name,
            ip,
            port,
            user,
            ssh_key_path: None,
            password: Some(password),
            auth_method: AuthMethod::Password,
            remote_deploy_path,
            role,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerRole {
    Primary,
    Replica,
    Monitor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSettings {
    pub initial_funds: f64,
    pub test_duration_minutes: u64,
    pub health_check_interval_seconds: u64,
    pub auto_deploy_threshold: f64,
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_percent: f64,
    pub max_memory_mb: u64,
    pub max_disk_gb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_port: u16,
    pub log_level: String,
    pub alert_endpoints: Vec<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            test_environments: vec![
                ServerConfig {
                    name: "ubuntu-test-server-1".to_string(),
                    ip: "192.168.1.100".to_string(),
                    port: 22,
                    user: "ubuntu".to_string(),
                    ssh_key_path: Some(PathBuf::from("~/.ssh/id_rsa")),
                    password: None,
                    auth_method: AuthMethod::Key,
                    remote_deploy_path: PathBuf::from("/home/ubuntu/aurelia_agent"),
                    role: ServerRole::Primary,
                },
                ServerConfig {
                    name: "ubuntu-test-server-2".to_string(),
                    ip: "192.168.1.101".to_string(),
                    port: 22,
                    user: "ubuntu".to_string(),
                    ssh_key_path: Some(PathBuf::from("~/.ssh/id_rsa")),
                    password: None,
                    auth_method: AuthMethod::Key,
                    remote_deploy_path: PathBuf::from("/home/ubuntu/aurelia_agent"),
                    role: ServerRole::Replica,
                },
            ],
            test_settings: TestSettings {
                initial_funds: 1000.0,
                test_duration_minutes: 60,
                health_check_interval_seconds: 30,
                auto_deploy_threshold: 0.8,
                resource_limits: ResourceLimits {
                    max_cpu_percent: 80.0,
                    max_memory_mb: 1024,
                    max_disk_gb: 10,
                },
            },
            monitoring: MonitoringConfig {
                metrics_port: 9090,
                log_level: "debug".to_string(),
                alert_endpoints: vec!["http://localhost:8080/alerts".to_string()],
            },
        }
    }
}

impl TestConfig {
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get_primary_server(&self) -> Option<&ServerConfig> {
        self.test_environments
            .iter()
            .find(|s| matches!(s.role, ServerRole::Primary))
    }

    pub fn get_replica_servers(&self) -> Vec<&ServerConfig> {
        self.test_environments
            .iter()
            .filter(|s| matches!(s.role, ServerRole::Replica))
            .collect()
    }
}
