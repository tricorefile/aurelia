use common::{AppEvent, DeploymentInfo, EventReceiver, EventSender, StrategyDecision};
use dotenvy::dotenv;
use ssh2::Session;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use tokio::sync::broadcast::error::RecvError;
use tracing::{error, info, warn};

/// A trait for deploying the agent.
pub trait Deployer: Send + Sync {
    fn deploy(&self, info: DeploymentInfo) -> Result<(), Box<dyn std::error::Error>>;
}

/// The real deployment handler that uses ssh2.
struct SshDeployer;

impl Deployer for SshDeployer {
    fn deploy(&self, info: DeploymentInfo) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            ip = %info.ip,
            user = %info.remote_user,
            "[Deployment] Starting deployment with ssh2."
        );

        let tcp = TcpStream::connect(format!("{}:22", info.ip))?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;

        sess.userauth_pubkey_file(
            &info.remote_user,
            None,
            Path::new(&info.private_key_path),
            None,
        )?;

        if !sess.authenticated() {
            return Err("SSH authentication failed.".into());
        }

        info!("[Deployment] SSH connection and authentication successful.");

        // 1. Create remote directories
        let remote_config_path = format!("{}/config", info.remote_path);
        let cmd = format!("mkdir -p {}", remote_config_path);
        self.exec_command(&mut sess, &cmd)?;

        // 2. Upload files
        self.upload_files(&mut sess, &info)?;

        // 3. Make kernel executable
        let remote_kernel_path = format!("{}/kernel", info.remote_path);
        let chmod_cmd = format!("chmod +x {}", remote_kernel_path);
        self.exec_command(&mut sess, &chmod_cmd)?;

        // 4. Start the remote agent
        let remote_log_path = format!("{}/aurelia.log", info.remote_path);
        let start_cmd = format!(
            "cd {} && nohup ./kernel > {} 2>&1 &",
            info.remote_path, remote_log_path
        );
        self.exec_command(&mut sess, &start_cmd)?;

        info!("[Deployment] Deployment process completed successfully.");
        Ok(())
    }
}

impl SshDeployer {
    fn exec_command(
        &self,
        sess: &mut Session,
        cmd: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut channel = sess.channel_session()?;
        channel.exec(cmd)?;
        // Reading the output is important to ensure the command has finished.
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;
        Ok(())
    }

    fn upload_files(
        &self,
        sess: &mut Session,
        info: &DeploymentInfo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let current_exe = env::current_exe()?;
        let base_path = current_exe
            .parent()
            .and_then(|p| p.parent()) // Move up from target/debug
            .ok_or("Could not determine project root")?;

        let files_to_upload = vec![
            ("kernel", "kernel"),
            (".env", ".env"),
            ("config/strategy.json", "config/strategy.json"),
            ("config/state.json", "config/state.json"),
        ];

        for (local_suffix, remote_suffix) in files_to_upload {
            let local_path = base_path.join(local_suffix);
            let remote_path_str = format!("{}/{}", info.remote_path, remote_suffix);

            if !local_path.exists() {
                warn!("[Deployment] File not found, skipping: {:?}", local_path);
                continue;
            }

            info!(
                "[Deployment] Uploading {:?} to {}",
                local_path, remote_path_str
            );
            let data = fs::read(&local_path)?;
            let mut remote_file =
                sess.scp_send(Path::new(&remote_path_str), 0o644, data.len() as u64, None)?;
            remote_file.write_all(&data)?;
        }
        Ok(())
    }
}

pub struct ExecutionEngine {
    tx: EventSender,
    rx: EventReceiver,
    api_key: String,
    api_secret: String,
    client: reqwest::Client,
    deployer: Box<dyn Deployer>,
}

impl ExecutionEngine {
    pub fn new(tx: EventSender, rx: EventReceiver, deployer: Box<dyn Deployer>) -> Self {
        // Try to load .env file but don't panic if it doesn't exist
        let _ = dotenv();

        // Use default values for tests or when env vars are not set
        let api_key = env::var("BINANCE_API_KEY").unwrap_or_else(|_| {
            warn!("BINANCE_API_KEY not set, using placeholder");
            "test_api_key".to_string()
        });
        let api_secret = env::var("BINANCE_API_SECRET").unwrap_or_else(|_| {
            warn!("BINANCE_API_SECRET not set, using placeholder");
            "test_api_secret".to_string()
        });

        info!("[Execution Engine] Initialized.");

        Self {
            tx,
            rx,
            api_key,
            api_secret,
            client: reqwest::Client::new(),
            deployer,
        }
    }

    pub async fn run(&mut self) {
        info!("[Execution Engine] Starting...");
        loop {
            match self.rx.recv().await {
                Ok(AppEvent::StrategyDecision(decision)) => self.handle_decision(decision).await,
                Ok(AppEvent::Deploy(info)) => {
                    if let Err(e) = self.deployer.deploy(info) {
                        error!("[Execution Engine] Deployment failed: {}", e);
                    }
                }
                Ok(_) => {}
                Err(RecvError::Lagged(n)) => {
                    warn!("[Execution Engine] Lagged by {} messages", n)
                }
                Err(RecvError::Closed) => {
                    error!("[Execution Engine] Event channel closed.");
                    break;
                }
            }
        }
    }

    async fn handle_decision(&mut self, decision: StrategyDecision) {
        match decision {
            StrategyDecision::Buy(symbol, price) => {
                info!(
                    symbol = symbol,
                    price = price,
                    "[Execution Engine] PREPARING REAL BUY ORDER"
                );
                // self.place_order(symbol, "BUY", 1.0, price).await;
            }
            StrategyDecision::Sell(symbol, price) => {
                info!(
                    symbol = symbol,
                    price = price,
                    "[Execution Engine] PREPARING REAL SELL ORDER"
                );
                // self.place_order(symbol, "SELL", 1.0, price).await;
            }
            StrategyDecision::Hold(_) => {}
        }
    }

    // This function is ready but commented out for safety.
    // To enable, you would uncomment the calls in handle_decision.
    /*
    async fn place_order(&self, symbol: String, side: &str, quantity: f64, price: f64) {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let params = format!(
            "symbol={}&side={}&type=LIMIT&timeInForce=GTC&quantity={}&price={}&timestamp={}",
            symbol, side, quantity, price, timestamp
        );

        let signature = {
            use hmac::{Hmac, Mac};
            use sha2::Sha256;
            type HmacSha256 = Hmac<Sha256>;
            let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes()).unwrap();
            mac.update(params.as_bytes());
            hex::encode(mac.finalize().into_bytes())
        };

        let request_url = format!("https://api.binance.com/api/v3/order?{}&signature={}", params, signature);

        match self.client.post(&request_url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await {
                Ok(response) => {
                    info!("Order placement response: {:?}", response.text().await);
                    // Here you would update the funds based on the real trade outcome
                }
                Err(e) => {
                    error!("Failed to place order: {}", e);
                }
            }
    }
    */
}
