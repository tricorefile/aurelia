use deployment_tester::{TestConfig, TestRunner};
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "aurelia-test")]
#[command(about = "Aurelia Agent Deployment and Testing Tool", long_about = None)]
struct Cli {
    /// Path to test configuration file
    #[arg(short, long, default_value = "test_env.json")]
    config: PathBuf,
    
    /// Path to kernel binary
    #[arg(short, long, default_value = "target/release/kernel")]
    binary: PathBuf,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the complete test suite
    Full,
    
    /// Test SSH connections only
    Connection,
    
    /// Deploy agents to all servers
    Deploy,
    
    /// Test self-replication capability
    Replication,
    
    /// Run validation tests
    Validate,
    
    /// Start continuous monitoring
    Monitor {
        /// Duration in minutes
        #[arg(short, long, default_value = "60")]
        duration: u64,
    },
    
    /// Cleanup all deployments
    Cleanup,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    let cli = Cli::parse();
    
    // Load or create default configuration
    let config = if cli.config.exists() {
        TestConfig::from_file(&cli.config)?
    } else {
        tracing::warn!("Config file not found, using default configuration");
        let default_config = TestConfig::default();
        default_config.save_to_file(&cli.config)?;
        default_config
    };
    
    let binary_path = cli.binary.clone();
    let runner = TestRunner::new(config.clone(), binary_path.clone());
    
    match cli.command {
        Commands::Full => {
            runner.run_complete_test_suite().await?;
        }
        Commands::Connection => {
            runner.run_specific_test("connection").await?;
        }
        Commands::Deploy => {
            runner.run_specific_test("deploy").await?;
        }
        Commands::Replication => {
            runner.run_specific_test("replication").await?;
        }
        Commands::Validate => {
            runner.run_specific_test("validation").await?;
        }
        Commands::Monitor { duration } => {
            let mut config = config;
            config.test_settings.test_duration_minutes = duration;
            let runner = TestRunner::new(config, binary_path);
            runner.run_specific_test("monitor").await?;
        }
        Commands::Cleanup => {
            runner.cleanup().await?;
        }
    }
    
    Ok(())
}