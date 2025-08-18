use autonomy_core::AutonomousAgent;
use std::path::PathBuf;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("===========================================");
    println!("Aurelia Autonomous Agent Test");
    println!("===========================================\n");
    
    // Create autonomous agent
    let binary_path = PathBuf::from("./target/release/kernel");
    let agent = AutonomousAgent::new(binary_path);
    
    println!("✅ Autonomous agent created");
    
    // Initialize the agent
    println!("Initializing agent...");
    agent.initialize().await?;
    println!("✅ Agent initialized");
    
    // Get initial status
    let status = agent.get_status().await;
    println!("\n📊 Initial Status:");
    println!("  - Running: {}", status.is_running);
    println!("  - Health: {}", status.health_status);
    println!("  - Active Replicas: {}", status.active_replicas);
    println!("  - Pending Tasks: {}", status.pending_tasks);
    
    // Run for a short period
    println!("\n🚀 Starting autonomous operations...");
    println!("The agent will now:");
    println!("  1. Monitor its own health");
    println!("  2. Make autonomous decisions");
    println!("  3. Schedule and execute tasks");
    println!("  4. Attempt self-replication (if configured)");
    println!("\nPress Ctrl+C to stop\n");
    
    // Start the agent
    let agent_handle = {
        let agent_clone = agent.clone();
        tokio::spawn(async move {
            if let Err(e) = agent_clone.run().await {
                eprintln!("Agent error: {}", e);
            }
        })
    };
    
    // Monitor the agent
    let monitor_handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            
            let status = agent.get_status().await;
            println!("\n📊 Status Update:");
            println!("  - Health: {}", status.health_status);
            println!("  - Active Replicas: {}", status.active_replicas);
            println!("  - Running Tasks: {}", status.running_tasks);
            println!("  - Recovery Rate: {:.1}%", status.recovery_success_rate);
        }
    });
    
    // Wait for termination
    tokio::select! {
        _ = agent_handle => println!("Agent stopped"),
        _ = monitor_handle => println!("Monitor stopped"),
        _ = tokio::signal::ctrl_c() => {
            println!("\n\n⚠️  Shutting down...");
        }
    }
    
    println!("✅ Test completed");
    Ok(())
}