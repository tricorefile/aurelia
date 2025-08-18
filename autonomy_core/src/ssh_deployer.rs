use anyhow::{Context, Result};
use ssh2::{Session, Sftp};
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

/// Pure Rust SSH deployment capability
/// Allows the kernel to deploy itself to remote servers without external scripts
pub struct SshDeployer {
    session: Session,
    sftp: Option<Sftp>,
    connected: bool,
}

impl SshDeployer {
    /// Create a new SSH deployer
    pub fn new() -> Self {
        Self {
            session: Session::new().unwrap(),
            sftp: None,
            connected: false,
        }
    }

    /// Connect to a remote server using SSH key authentication
    pub fn connect_with_key(
        &mut self,
        host: &str,
        port: u16,
        username: &str,
        private_key_path: &Path,
        passphrase: Option<&str>,
    ) -> Result<()> {
        info!("Connecting to {}:{} as user {}", host, port, username);

        // Establish TCP connection
        let tcp = TcpStream::connect(format!("{}:{}", host, port))
            .context("Failed to establish TCP connection")?;

        self.session.set_tcp_stream(tcp);
        self.session.handshake().context("SSH handshake failed")?;

        // Try public key authentication
        if private_key_path.exists() {
            info!("Authenticating with SSH key: {:?}", private_key_path);

            if let Some(pass) = passphrase {
                self.session
                    .userauth_pubkey_file(username, None, private_key_path, Some(pass))
                    .context("SSH key authentication with passphrase failed")?;
            } else {
                self.session
                    .userauth_pubkey_file(username, None, private_key_path, None)
                    .context("SSH key authentication failed")?;
            }
        } else {
            return Err(anyhow::anyhow!(
                "SSH key file not found: {:?}",
                private_key_path
            ));
        }

        if !self.session.authenticated() {
            return Err(anyhow::anyhow!("Authentication failed"));
        }

        self.connected = true;
        info!("Successfully connected and authenticated to {}", host);
        Ok(())
    }

    /// Connect to a remote server using password authentication
    pub fn connect_with_password(
        &mut self,
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Result<()> {
        info!(
            "Connecting to {}:{} as user {} with password",
            host, port, username
        );

        // Establish TCP connection
        let tcp = TcpStream::connect(format!("{}:{}", host, port))
            .context("Failed to establish TCP connection")?;

        self.session.set_tcp_stream(tcp);
        self.session.handshake().context("SSH handshake failed")?;

        // Password authentication
        self.session
            .userauth_password(username, password)
            .context("Password authentication failed")?;

        if !self.session.authenticated() {
            return Err(anyhow::anyhow!("Authentication failed"));
        }

        self.connected = true;
        info!("Successfully connected and authenticated to {}", host);
        Ok(())
    }

    /// Execute a command on the remote server
    pub fn execute_command(&self, command: &str) -> Result<String> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to remote server"));
        }

        let mut channel = self
            .session
            .channel_session()
            .context("Failed to create SSH channel")?;

        channel.exec(command).context("Failed to execute command")?;

        let mut output = String::new();
        channel
            .read_to_string(&mut output)
            .context("Failed to read command output")?;

        channel.wait_close()?;
        let exit_status = channel.exit_status()?;

        if exit_status != 0 {
            warn!("Command '{}' exited with status {}", command, exit_status);
        }

        Ok(output)
    }

    /// Create a directory on the remote server
    pub fn create_remote_directory(&self, path: &str) -> Result<()> {
        info!("Creating remote directory: {}", path);
        let command = format!("mkdir -p {}", path);
        self.execute_command(&command)?;
        Ok(())
    }

    /// Upload a file to the remote server
    pub fn upload_file(&mut self, local_path: &Path, remote_path: &str) -> Result<()> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to remote server"));
        }

        info!("Uploading {:?} to {}", local_path, remote_path);

        // Initialize SFTP if not already done
        if self.sftp.is_none() {
            self.sftp = Some(
                self.session
                    .sftp()
                    .context("Failed to create SFTP session")?,
            );
        }

        let sftp = self.sftp.as_ref().unwrap();

        // Read local file
        let mut local_file = File::open(local_path).context("Failed to open local file")?;
        let mut contents = Vec::new();
        local_file
            .read_to_end(&mut contents)
            .context("Failed to read local file")?;

        // Create remote file
        let mut remote_file = sftp
            .create(Path::new(remote_path))
            .context("Failed to create remote file")?;

        // Write contents
        remote_file
            .write_all(&contents)
            .context("Failed to write to remote file")?;

        info!("Successfully uploaded {} bytes", contents.len());
        Ok(())
    }

    /// Deploy the kernel binary to a remote server
    pub fn deploy_kernel(
        &mut self,
        local_binary: &Path,
        remote_path: &str,
        config_files: Option<Vec<PathBuf>>,
    ) -> Result<()> {
        info!("Starting kernel deployment to {}", remote_path);

        // Create remote directories
        self.create_remote_directory(remote_path)?;
        self.create_remote_directory(&format!("{}/config", remote_path))?;
        self.create_remote_directory(&format!("{}/logs", remote_path))?;
        self.create_remote_directory(&format!("{}/data", remote_path))?;

        // Upload binary
        let remote_binary = format!("{}/kernel", remote_path);
        self.upload_file(local_binary, &remote_binary)?;

        // Make binary executable
        self.execute_command(&format!("chmod +x {}", remote_binary))?;

        // Upload config files if provided
        if let Some(configs) = config_files {
            for config in configs {
                if config.exists() {
                    let filename = config.file_name().unwrap().to_str().unwrap();
                    let remote_config = format!("{}/config/{}", remote_path, filename);
                    self.upload_file(&config, &remote_config)?;
                }
            }
        }

        info!("Kernel deployment completed successfully");
        Ok(())
    }

    /// Start the kernel on the remote server
    pub fn start_kernel(&self, remote_path: &str) -> Result<()> {
        info!("Starting kernel at {}", remote_path);

        // Stop any existing instance
        let _ = self.execute_command("pkill -f kernel");

        // Start new instance in background
        let start_command = format!(
            "cd {} && nohup ./kernel > logs/aurelia.log 2>&1 &",
            remote_path
        );
        self.execute_command(&start_command)?;

        // Verify it started
        std::thread::sleep(std::time::Duration::from_secs(2));
        let check_output = self.execute_command("ps aux | grep kernel | grep -v grep")?;

        if check_output.trim().is_empty() {
            return Err(anyhow::anyhow!("Kernel failed to start"));
        }

        info!("Kernel started successfully");
        Ok(())
    }

    /// Stop the kernel on the remote server
    pub fn stop_kernel(&self) -> Result<()> {
        info!("Stopping kernel");
        self.execute_command("pkill -f kernel")?;
        Ok(())
    }

    /// Check if kernel is running on remote server
    pub fn check_kernel_status(&self) -> Result<bool> {
        let output = self.execute_command("ps aux | grep kernel | grep -v grep")?;
        Ok(!output.trim().is_empty())
    }

    /// Create systemd service for automatic startup
    pub fn setup_systemd_service(&self, remote_path: &str, username: &str) -> Result<()> {
        info!("Setting up systemd service");

        let service_content = format!(
            r#"[Unit]
Description=Aurelia Autonomous Trading System
After=network.target

[Service]
Type=simple
User={}
WorkingDirectory={}
ExecStart={}/kernel
Restart=always
RestartSec=10
StandardOutput=append:{}/logs/aurelia.log
StandardError=append:{}/logs/aurelia.error.log

[Install]
WantedBy=multi-user.target
"#,
            username, remote_path, remote_path, remote_path, remote_path
        );

        // Write service file
        let temp_service = "/tmp/aurelia.service";
        self.execute_command(&format!("echo '{}' > {}", service_content, temp_service))?;

        // Move to systemd directory and reload
        self.execute_command(&format!("sudo mv {} /etc/systemd/system/", temp_service))?;
        self.execute_command("sudo systemctl daemon-reload")?;
        self.execute_command("sudo systemctl enable aurelia")?;

        info!("Systemd service setup completed");
        Ok(())
    }

    /// Get logs from remote server
    pub fn get_logs(&self, remote_path: &str, lines: usize) -> Result<String> {
        let command = format!(
            "tail -n {} {}/logs/aurelia.log 2>/dev/null || echo 'No logs found'",
            lines, remote_path
        );
        self.execute_command(&command)
    }

    /// Perform a complete deployment with all steps
    pub fn full_deploy(
        &mut self,
        host: &str,
        port: u16,
        username: &str,
        auth: AuthMethod,
        local_binary: &Path,
        remote_path: &str,
        config_files: Option<Vec<PathBuf>>,
        setup_service: bool,
    ) -> Result<()> {
        // Connect
        match auth {
            AuthMethod::Password(password) => {
                self.connect_with_password(host, port, username, &password)?;
            }
            AuthMethod::Key { path, passphrase } => {
                self.connect_with_key(host, port, username, &path, passphrase.as_deref())?;
            }
        }

        // Deploy
        self.deploy_kernel(local_binary, remote_path, config_files)?;

        // Setup systemd service if requested
        if setup_service {
            self.setup_systemd_service(remote_path, username)?;
            self.execute_command("sudo systemctl start aurelia")?;
        } else {
            self.start_kernel(remote_path)?;
        }

        // Verify deployment
        if self.check_kernel_status()? {
            info!("Deployment successful - kernel is running");
        } else {
            return Err(anyhow::anyhow!("Deployment failed - kernel is not running"));
        }

        Ok(())
    }

    /// Disconnect from the remote server
    pub fn disconnect(&mut self) {
        if self.connected {
            self.sftp = None;
            self.connected = false;
            info!("Disconnected from remote server");
        }
    }
}

/// Authentication method for SSH connection
pub enum AuthMethod {
    Password(String),
    Key {
        path: PathBuf,
        passphrase: Option<String>,
    },
}

impl Drop for SshDeployer {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_deployer_creation() {
        let deployer = SshDeployer::new();
        assert!(!deployer.connected);
    }
}
