use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetServer {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_base64: Option<String>,  // 密码使用base64编码存储
    #[serde(default = "default_auth_method")]
    pub auth_method: AuthMethod,
    pub remote_path: String,
    pub enabled: bool,
    pub priority: u32,
    pub tags: Vec<String>,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
}

fn default_auth_method() -> AuthMethod {
    AuthMethod::Key
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    Key,
    Password,
    KeyWithPassphrase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultSettings {
    pub port: u16,
    pub username: String,
    pub ssh_key_path: String,
    pub remote_path: String,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
    pub connection_timeout_seconds: u64,
    pub deployment_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStrategy {
    #[serde(rename = "type")]
    pub strategy_type: String,
    pub parallel_deployments: usize,
    pub delay_between_deployments_seconds: u64,
    pub health_check_after_deployment: bool,
    pub rollback_on_failure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub strict_host_key_checking: bool,
    pub compression: bool,
    pub keepalive_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub target_servers: Vec<TargetServer>,
    pub default_settings: DefaultSettings,
    pub deployment_strategy: DeploymentStrategy,
    pub ssh_config: SshConfig,
}

impl ServerConfig {
    /// 从JSON文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
        
        let config: ServerConfig = serde_json::from_str(&content)
            .with_context(|| "Failed to parse config JSON")?;
        
        Ok(config)
    }
    
    /// 保存配置到JSON文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(path.as_ref(), content)
            .with_context(|| format!("Failed to write config file: {:?}", path.as_ref()))?;
        
        Ok(())
    }
    
    /// 获取所有启用的服务器
    pub fn get_enabled_servers(&self) -> Vec<&TargetServer> {
        self.target_servers
            .iter()
            .filter(|s| s.enabled)
            .collect()
    }
    
    /// 按优先级排序获取启用的服务器
    pub fn get_servers_by_priority(&self) -> Vec<&TargetServer> {
        let mut servers = self.get_enabled_servers();
        servers.sort_by_key(|s| s.priority);
        servers
    }
    
    /// 根据标签获取服务器
    pub fn get_servers_by_tag(&self, tag: &str) -> Vec<&TargetServer> {
        self.target_servers
            .iter()
            .filter(|s| s.enabled && s.tags.contains(&tag.to_string()))
            .collect()
    }
    
    /// 根据ID获取服务器
    pub fn get_server_by_id(&self, id: &str) -> Option<&TargetServer> {
        self.target_servers
            .iter()
            .find(|s| s.id == id)
    }
    
    /// 添加新服务器
    pub fn add_server(&mut self, server: TargetServer) -> Result<()> {
        // 检查ID是否已存在
        if self.get_server_by_id(&server.id).is_some() {
            return Err(anyhow::anyhow!("Server with ID '{}' already exists", server.id));
        }
        
        self.target_servers.push(server);
        Ok(())
    }
    
    /// 删除服务器
    pub fn remove_server(&mut self, id: &str) -> Result<()> {
        let original_len = self.target_servers.len();
        self.target_servers.retain(|s| s.id != id);
        
        if self.target_servers.len() == original_len {
            return Err(anyhow::anyhow!("Server with ID '{}' not found", id));
        }
        
        Ok(())
    }
    
    /// 更新服务器配置
    pub fn update_server(&mut self, id: &str, updater: impl FnOnce(&mut TargetServer)) -> Result<()> {
        let server = self.target_servers
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| anyhow::anyhow!("Server with ID '{}' not found", id))?;
        
        updater(server);
        Ok(())
    }
    
    /// 启用/禁用服务器
    pub fn set_server_enabled(&mut self, id: &str, enabled: bool) -> Result<()> {
        self.update_server(id, |s| s.enabled = enabled)
    }
    
    /// 展开SSH密钥路径（处理~符号）
    pub fn expand_ssh_key_path(path: &str) -> PathBuf {
        if path.starts_with("~") {
            if let Some(home) = dirs::home_dir() {
                return home.join(&path[2..]);
            }
        }
        PathBuf::from(path)
    }
}

impl TargetServer {
    /// 创建新的目标服务器
    pub fn new(id: String, name: String, ip: String, username: String) -> Self {
        Self {
            id,
            name,
            ip,
            port: 22,
            username,
            ssh_key_path: Some("~/.ssh/id_rsa".to_string()),
            password_base64: None,
            auth_method: AuthMethod::Key,
            remote_path: "/home/ubuntu/aurelia".to_string(),
            enabled: true,
            priority: 100,
            tags: Vec::new(),
            max_retries: 3,
            retry_delay_seconds: 60,
        }
    }
    
    /// 创建使用密码认证的服务器
    pub fn new_with_password(id: String, name: String, ip: String, username: String, password: String) -> Self {
        let mut server = Self::new(id, name, ip, username);
        server.set_password(&password);
        server
    }
    
    /// 获取展开后的SSH密钥路径
    pub fn get_expanded_ssh_key_path(&self) -> PathBuf {
        self.ssh_key_path.as_ref()
            .map(|path| ServerConfig::expand_ssh_key_path(path))
            .unwrap_or_else(|| PathBuf::from("~/.ssh/id_rsa"))
    }
    
    /// 设置密码（自动编码为base64）
    pub fn set_password(&mut self, password: &str) {
        self.password_base64 = Some(BASE64.encode(password.as_bytes()));
        self.auth_method = AuthMethod::Password;
    }
    
    /// 获取解码后的密码
    pub fn get_password(&self) -> Option<String> {
        self.password_base64.as_ref().and_then(|encoded| {
            BASE64.decode(encoded).ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
        })
    }
    
    /// 生成部署信息
    pub fn to_deployment_info(&self) -> common::DeploymentInfo {
        common::DeploymentInfo {
            ip: self.ip.clone(),
            remote_user: self.username.clone(),
            private_key_path: self.get_expanded_ssh_key_path().to_string_lossy().to_string(),
            remote_path: self.remote_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_load_config() {
        let config_json = r#"{
            "target_servers": [
                {
                    "id": "test-1",
                    "name": "Test Server",
                    "ip": "192.168.1.100",
                    "port": 22,
                    "username": "test",
                    "ssh_key_path": "~/.ssh/id_rsa",
                    "remote_path": "/home/test",
                    "enabled": true,
                    "priority": 1,
                    "tags": ["test"],
                    "max_retries": 3,
                    "retry_delay_seconds": 60
                }
            ],
            "default_settings": {
                "port": 22,
                "username": "ubuntu",
                "ssh_key_path": "~/.ssh/id_rsa",
                "remote_path": "/home/ubuntu/aurelia",
                "max_retries": 3,
                "retry_delay_seconds": 60,
                "connection_timeout_seconds": 30,
                "deployment_timeout_seconds": 300
            },
            "deployment_strategy": {
                "type": "progressive",
                "parallel_deployments": 2,
                "delay_between_deployments_seconds": 30,
                "health_check_after_deployment": true,
                "rollback_on_failure": true
            },
            "ssh_config": {
                "strict_host_key_checking": false,
                "compression": true,
                "keepalive_interval_seconds": 60
            }
        }"#;
        
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.json");
        fs::write(&file_path, config_json).unwrap();
        
        let config = ServerConfig::from_file(&file_path).unwrap();
        assert_eq!(config.target_servers.len(), 1);
        assert_eq!(config.target_servers[0].id, "test-1");
    }
    
    #[test]
    fn test_server_management() {
        let mut config = ServerConfig {
            target_servers: vec![],
            default_settings: DefaultSettings {
                port: 22,
                username: "ubuntu".to_string(),
                ssh_key_path: "~/.ssh/id_rsa".to_string(),
                remote_path: "/home/ubuntu".to_string(),
                max_retries: 3,
                retry_delay_seconds: 60,
                connection_timeout_seconds: 30,
                deployment_timeout_seconds: 300,
            },
            deployment_strategy: DeploymentStrategy {
                strategy_type: "progressive".to_string(),
                parallel_deployments: 2,
                delay_between_deployments_seconds: 30,
                health_check_after_deployment: true,
                rollback_on_failure: true,
            },
            ssh_config: SshConfig {
                strict_host_key_checking: false,
                compression: true,
                keepalive_interval_seconds: 60,
            },
        };
        
        // 添加服务器
        let server = TargetServer::new(
            "test-1".to_string(),
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            "test".to_string(),
        );
        config.add_server(server).unwrap();
        
        assert_eq!(config.target_servers.len(), 1);
        
        // 更新服务器
        config.update_server("test-1", |s| s.priority = 5).unwrap();
        assert_eq!(config.get_server_by_id("test-1").unwrap().priority, 5);
        
        // 禁用服务器
        config.set_server_enabled("test-1", false).unwrap();
        assert_eq!(config.get_enabled_servers().len(), 0);
        
        // 删除服务器
        config.remove_server("test-1").unwrap();
        assert_eq!(config.target_servers.len(), 0);
    }
}