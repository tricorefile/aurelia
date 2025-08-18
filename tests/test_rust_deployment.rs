use autonomy_core::{DeploymentCommander, SshDeployer, AuthMethod};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Testing Pure Rust SSH Deployment");
    println!("=================================\n");

    // Test 1: Direct SSH Deployer
    println!("Test 1: Direct SSH Connection and Command Execution");
    println!("---------------------------------------------------");
    
    let mut deployer = SshDeployer::new();
    
    // Try to connect to localhost for testing
    match deployer.connect_with_password("127.0.0.1", 22, "test", "test") {
        Ok(_) => {
            println!("✅ Connected successfully");
            
            // Try to execute a command
            match deployer.execute_command("echo 'Hello from Rust SSH!'") {
                Ok(output) => println!("Command output: {}", output),
                Err(e) => println!("Command failed: {}", e),
            }
            
            // Check kernel status
            match deployer.check_kernel_status() {
                Ok(running) => println!("Kernel running: {}", running),
                Err(e) => println!("Status check failed: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Connection failed (expected for test): {}", e);
        }
    }

    println!("\nTest 2: Deployment Commander");
    println!("-----------------------------");
    
    // Create deployment commander
    let commander = DeploymentCommander::new(PathBuf::from("./target/release/kernel"));
    
    // Get deployment status
    let status = commander.get_deployment_status().await;
    println!("Current deployment status:");
    for (server_id, deploy_status) in status {
        println!("  {}: {:?}", server_id, deploy_status.status);
    }
    
    // Try to deploy to a specific server (if configured)
    println!("\nAttempting deployment to server-pwd...");
    match commander.deploy_to_server("server-pwd").await {
        Ok(_) => println!("✅ Deployment successful!"),
        Err(e) => println!("❌ Deployment failed: {}", e),
    }
    
    // Check server status
    println!("\nChecking server status...");
    match commander.check_server_status("server-pwd").await {
        Ok(running) => println!("Server running: {}", running),
        Err(e) => println!("Status check failed: {}", e),
    }
    
    // Get logs
    println!("\nFetching server logs...");
    match commander.get_server_logs("server-pwd", 10).await {
        Ok(logs) => println!("Recent logs:\n{}", logs),
        Err(e) => println!("Failed to get logs: {}", e),
    }

    println!("\nTest 3: Deployment Features");
    println!("----------------------------");
    
    // Test deploy to priority servers
    println!("Deploying to high-priority servers (priority <= 1)...");
    match commander.deploy_to_priority_servers(1).await {
        Ok(results) => {
            for (server_id, result) in results {
                match result {
                    Ok(_) => println!("  {} ✅ Success", server_id),
                    Err(e) => println!("  {} ❌ Failed: {}", server_id, e),
                }
            }
        }
        Err(e) => println!("Priority deployment failed: {}", e),
    }
    
    // Execute command on all servers
    println!("\nExecuting command on all servers...");
    match commander.execute_on_all("uname -a").await {
        Ok(results) => {
            for (server_id, result) in results {
                match result {
                    Ok(output) => println!("  {} Output: {}", server_id, output.trim()),
                    Err(e) => println!("  {} Error: {}", server_id, e),
                }
            }
        }
        Err(e) => println!("Command execution failed: {}", e),
    }

    println!("\n✅ All tests completed!");
    Ok(())
}