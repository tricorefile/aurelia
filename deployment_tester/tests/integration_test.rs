use deployment_tester::{TestConfig, DeploymentClient, AgentMonitor};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_config_loading() {
    let config = TestConfig::default();
    assert_eq!(config.test_environments.len(), 2);
    assert!(config.get_primary_server().is_some());
    assert_eq!(config.get_replica_servers().len(), 1);
}

#[tokio::test]
async fn test_config_serialization() {
    let config = TestConfig::default();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.json");
    
    config.save_to_file(&config_path).unwrap();
    assert!(config_path.exists());
    
    let loaded_config = TestConfig::from_file(&config_path).unwrap();
    assert_eq!(loaded_config.test_environments.len(), config.test_environments.len());
}

#[cfg(test)]
mod mock_tests {
    use super::*;
    use deployment_tester::config::{ServerConfig, ServerRole, AuthMethod};
    
    fn create_mock_server_config() -> ServerConfig {
        ServerConfig {
            name: "test-server".to_string(),
            ip: "127.0.0.1".to_string(),
            port: 22,
            user: "testuser".to_string(),
            ssh_key_path: Some(PathBuf::from("~/.ssh/id_rsa")),
            password: None,
            auth_method: AuthMethod::Key,
            remote_deploy_path: PathBuf::from("/tmp/aurelia_test"),
            role: ServerRole::Primary,
        }
    }
    
    #[test]
    fn test_server_config_creation() {
        let server = create_mock_server_config();
        assert_eq!(server.name, "test-server");
        assert_eq!(server.ip, "127.0.0.1");
        assert!(matches!(server.role, ServerRole::Primary));
    }
    
    #[test]
    fn test_deployment_client_creation() {
        let server = create_mock_server_config();
        let _client = DeploymentClient::new(server);
        // Client created successfully
    }
    
    #[test]
    fn test_monitor_creation() {
        let server = create_mock_server_config();
        let _monitor = AgentMonitor::new(server);
        // Monitor created successfully
    }
}