use crate::config::ServerConfig;
use anyhow::{Context, Result};
use ssh2::Session;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

pub struct DeploymentClient {
    config: ServerConfig,
}

impl DeploymentClient {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    pub fn connect(&self) -> Result<Session> {
        info!("Connecting to {}:{}...", self.config.ip, self.config.port);
        
        let tcp = TcpStream::connect(format!("{}:{}", self.config.ip, self.config.port))
            .context("Failed to establish TCP connection")?;
        
        let mut sess = Session::new().context("Failed to create SSH session")?;
        sess.set_tcp_stream(tcp);
        sess.handshake().context("SSH handshake failed")?;
        
        let key_path = self.expand_tilde(&self.config.ssh_key_path);
        sess.userauth_pubkey_file(
            &self.config.user,
            None,
            Path::new(&key_path),
            None,
        ).context("SSH authentication failed")?;
        
        if !sess.authenticated() {
            return Err(anyhow::anyhow!("SSH authentication failed"));
        }
        
        info!("Successfully connected to {}", self.config.ip);
        Ok(sess)
    }

    pub fn test_connection(&self) -> Result<bool> {
        let sess = self.connect()?;
        let mut channel = sess.channel_session()?;
        channel.exec("echo 'Connection test successful'")?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;
        Ok(output.contains("Connection test successful"))
    }

    pub fn deploy_agent(&self, local_binary_path: &Path) -> Result<()> {
        info!("Starting deployment to {}...", self.config.name);
        
        let sess = self.connect()?;
        
        // Create remote directory
        self.create_remote_directory(&sess)?;
        
        // Upload binary
        self.upload_file(&sess, local_binary_path, "kernel")?;
        
        // Upload configuration files
        self.upload_config_files(&sess)?;
        
        // Make binary executable
        self.make_executable(&sess, "kernel")?;
        
        // Create startup script
        self.create_startup_script(&sess)?;
        
        // Start the agent
        self.start_agent(&sess)?;
        
        info!("Deployment to {} completed successfully", self.config.name);
        Ok(())
    }

    pub fn execute_command(&self, sess: &Session, cmd: &str) -> Result<String> {
        let mut channel = sess.channel_session()?;
        channel.exec(cmd)?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;
        
        let exit_status = channel.exit_status()?;
        if exit_status != 0 {
            return Err(anyhow::anyhow!(
                "Command failed with exit code {}: {}",
                exit_status,
                output
            ));
        }
        
        Ok(output)
    }

    fn create_remote_directory(&self, sess: &Session) -> Result<()> {
        info!("Creating remote directory: {:?}", self.config.remote_deploy_path);
        let cmd = format!("mkdir -p {:?}", self.config.remote_deploy_path);
        self.execute_command(sess, &cmd)?;
        
        let config_dir = self.config.remote_deploy_path.join("config");
        let cmd = format!("mkdir -p {:?}", config_dir);
        self.execute_command(sess, &cmd)?;
        
        Ok(())
    }

    fn upload_file(&self, sess: &Session, local_path: &Path, remote_name: &str) -> Result<()> {
        if !local_path.exists() {
            return Err(anyhow::anyhow!("Local file not found: {:?}", local_path));
        }
        
        let remote_path = self.config.remote_deploy_path.join(remote_name);
        info!("Uploading {:?} to {:?}", local_path, remote_path);
        
        let data = fs::read(local_path)?;
        let mut remote_file = sess.scp_send(
            &remote_path,
            0o644,
            data.len() as u64,
            None,
        )?;
        remote_file.write_all(&data)?;
        
        Ok(())
    }

    fn upload_config_files(&self, sess: &Session) -> Result<()> {
        // Create .env file
        let env_content = format!(
            "BINANCE_API_KEY=test_api_key\n\
             BINANCE_API_SECRET=test_api_secret\n\
             DEPLOYMENT_MODE=test\n"
        );
        
        let remote_env_path = self.config.remote_deploy_path.join(".env");
        let mut remote_file = sess.scp_send(
            &remote_env_path,
            0o644,
            env_content.len() as u64,
            None,
        )?;
        remote_file.write_all(env_content.as_bytes())?;
        
        // Upload strategy.json
        let strategy_content = r#"{
            "strategy_type": "momentum",
            "symbol": "BTCUSDT",
            "interval": "1h",
            "lookback_periods": 20,
            "threshold": 0.02
        }"#;
        
        let remote_strategy_path = self.config.remote_deploy_path.join("config/strategy.json");
        let mut remote_file = sess.scp_send(
            &remote_strategy_path,
            0o644,
            strategy_content.len() as u64,
            None,
        )?;
        remote_file.write_all(strategy_content.as_bytes())?;
        
        // Upload state.json
        let state_content = r#"{
            "funds": 1000.0,
            "positions": {},
            "last_update": null
        }"#;
        
        let remote_state_path = self.config.remote_deploy_path.join("config/state.json");
        let mut remote_file = sess.scp_send(
            &remote_state_path,
            0o644,
            state_content.len() as u64,
            None,
        )?;
        remote_file.write_all(state_content.as_bytes())?;
        
        Ok(())
    }

    fn make_executable(&self, sess: &Session, file_name: &str) -> Result<()> {
        let remote_path = self.config.remote_deploy_path.join(file_name);
        let cmd = format!("chmod +x {:?}", remote_path);
        self.execute_command(sess, &cmd)?;
        Ok(())
    }

    fn create_startup_script(&self, sess: &Session) -> Result<()> {
        let script_content = format!(
            "#!/bin/bash\n\
             cd {:?}\n\
             nohup ./kernel > aurelia.log 2>&1 &\n\
             echo $! > aurelia.pid\n\
             echo \"Agent started with PID: $(cat aurelia.pid)\"\n",
            self.config.remote_deploy_path
        );
        
        let remote_script_path = self.config.remote_deploy_path.join("start_agent.sh");
        let mut remote_file = sess.scp_send(
            &remote_script_path,
            0o755,
            script_content.len() as u64,
            None,
        )?;
        remote_file.write_all(script_content.as_bytes())?;
        
        Ok(())
    }

    fn start_agent(&self, sess: &Session) -> Result<()> {
        info!("Starting agent on {}", self.config.name);
        let cmd = format!(
            "cd {:?} && ./start_agent.sh",
            self.config.remote_deploy_path
        );
        let output = self.execute_command(sess, &cmd)?;
        info!("Agent start output: {}", output);
        Ok(())
    }

    pub fn stop_agent(&self) -> Result<()> {
        let sess = self.connect()?;
        let cmd = format!(
            "cd {:?} && if [ -f aurelia.pid ]; then kill $(cat aurelia.pid); rm aurelia.pid; fi",
            self.config.remote_deploy_path
        );
        self.execute_command(&sess, &cmd)?;
        info!("Agent stopped on {}", self.config.name);
        Ok(())
    }

    pub fn cleanup(&self) -> Result<()> {
        let sess = self.connect()?;
        let cmd = format!(
            "rm -rf {:?}",
            self.config.remote_deploy_path
        );
        self.execute_command(&sess, &cmd)?;
        info!("Cleaned up deployment on {}", self.config.name);
        Ok(())
    }

    fn expand_tilde(&self, path: &Path) -> PathBuf {
        if let Some(path_str) = path.to_str() {
            if path_str.starts_with("~") {
                if let Ok(home) = std::env::var("HOME") {
                    return PathBuf::from(path_str.replacen("~", &home, 1));
                }
            }
        }
        path.to_path_buf()
    }

    pub fn trigger_self_replication(&self, target_server: &ServerConfig) -> Result<()> {
        info!("Triggering self-replication from {} to {}", 
              self.config.name, target_server.name);
        
        let sess = self.connect()?;
        
        let deploy_info = serde_json::json!({
            "ip": target_server.ip,
            "remote_user": target_server.user,
            "private_key_path": target_server.ssh_key_path,
            "remote_path": target_server.remote_deploy_path,
            "local_exe_path": "./kernel"
        });
        
        let deploy_json = serde_json::to_string_pretty(&deploy_info)?;
        let trigger_path = self.config.remote_deploy_path.join("deploy_trigger.json");
        
        let mut remote_file = sess.scp_send(
            &trigger_path,
            0o644,
            deploy_json.len() as u64,
            None,
        )?;
        remote_file.write_all(deploy_json.as_bytes())?;
        
        info!("Self-replication trigger sent");
        Ok(())
    }
}