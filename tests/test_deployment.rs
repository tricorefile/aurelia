use autonomy_core::{DeploymentCommander, SshDeployer, AuthMethod};
use std::path::PathBuf;
use tracing::{info, error, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n========================================");
    println!("  Aurelia 纯Rust SSH部署测试");
    println!("========================================\n");

    // 目标服务器信息（从配置文件）
    let target_server = "server-pwd"; // 194.146.13.14
    let binary_path = PathBuf::from("./target/release/kernel");

    // 检查二进制文件是否存在
    if !binary_path.exists() {
        error!("二进制文件不存在: {:?}", binary_path);
        println!("请先运行: cargo build --release");
        return Err(anyhow::anyhow!("Binary not found"));
    }

    println!("📦 二进制文件: {:?}", binary_path);
    println!("🎯 目标服务器: {}\n", target_server);

    // 创建部署指挥官
    let commander = DeploymentCommander::new(binary_path.clone());

    // 步骤1: 获取当前部署状态
    println!("1️⃣ 检查当前部署状态...");
    println!("----------------------------");
    let status = commander.get_deployment_status().await;
    for (server_id, deploy_status) in &status {
        println!("  {} - 状态: {:?}", server_id, deploy_status.status);
        if let Some(last_attempt) = &deploy_status.last_attempt {
            println!("    最后尝试: {}", last_attempt);
        }
        if let Some(error) = &deploy_status.error_message {
            println!("    错误: {}", error);
        }
    }

    // 步骤2: 测试SSH连接
    println!("\n2️⃣ 测试SSH连接...");
    println!("----------------------------");
    
    // 直接测试SSH连接
    let mut test_deployer = SshDeployer::new();
    match test_deployer.connect_with_password("194.146.13.14", 22, "root", "Tricorelife@123") {
        Ok(_) => {
            println!("✅ SSH连接成功!");
            
            // 测试执行命令
            match test_deployer.execute_command("hostname && whoami && pwd") {
                Ok(output) => {
                    println!("📋 服务器信息:");
                    println!("{}", output);
                }
                Err(e) => {
                    warn!("命令执行失败: {}", e);
                }
            }
            
            // 检查目标目录
            match test_deployer.execute_command("ls -la /opt/ 2>/dev/null || echo '目录不存在'") {
                Ok(output) => {
                    println!("📁 /opt 目录内容:");
                    println!("{}", output);
                }
                Err(e) => {
                    warn!("无法列出目录: {}", e);
                }
            }
        }
        Err(e) => {
            error!("❌ SSH连接失败: {}", e);
            println!("请检查:");
            println!("  - 服务器IP是否正确 (194.146.13.14)");
            println!("  - 端口22是否开放");
            println!("  - 用户名和密码是否正确");
            return Err(e);
        }
    }

    // 步骤3: 部署到服务器
    println!("\n3️⃣ 开始部署到服务器...");
    println!("----------------------------");
    
    match commander.deploy_to_server(target_server).await {
        Ok(_) => {
            println!("✅ 部署成功!");
        }
        Err(e) => {
            error!("❌ 部署失败: {}", e);
            
            // 尝试直接部署
            println!("\n尝试直接部署方式...");
            let mut deployer = SshDeployer::new();
            
            // 连接
            deployer.connect_with_password("194.146.13.14", 22, "root", "Tricorelife@123")?;
            
            // 部署
            println!("📤 上传文件...");
            deployer.deploy_kernel(
                &binary_path,
                "/opt/aurelia",
                Some(vec![PathBuf::from("config/target_servers.json")]),
            )?;
            
            // 启动
            println!("🚀 启动kernel...");
            deployer.start_kernel("/opt/aurelia")?;
            
            println!("✅ 直接部署完成!");
        }
    }

    // 步骤4: 检查服务状态
    println!("\n4️⃣ 检查服务状态...");
    println!("----------------------------");
    
    match commander.check_server_status(target_server).await {
        Ok(is_running) => {
            if is_running {
                println!("✅ Kernel正在运行!");
            } else {
                println!("⚠️ Kernel未运行");
            }
        }
        Err(e) => {
            warn!("无法检查状态: {}", e);
            
            // 直接检查
            let mut deployer = SshDeployer::new();
            deployer.connect_with_password("194.146.13.14", 22, "root", "Tricorelife@123")?;
            
            let ps_output = deployer.execute_command("ps aux | grep kernel | grep -v grep")?;
            if !ps_output.trim().is_empty() {
                println!("✅ 进程检查: Kernel正在运行");
                println!("{}", ps_output);
            } else {
                println!("⚠️ 进程检查: Kernel未运行");
            }
        }
    }

    // 步骤5: 获取日志
    println!("\n5️⃣ 获取最新日志...");
    println!("----------------------------");
    
    match commander.get_server_logs(target_server, 20).await {
        Ok(logs) => {
            println!("📜 最新日志:");
            println!("{}", logs);
        }
        Err(e) => {
            warn!("无法获取日志: {}", e);
            
            // 直接获取
            let mut deployer = SshDeployer::new();
            deployer.connect_with_password("194.146.13.14", 22, "root", "Tricorelife@123")?;
            
            let logs = deployer.get_logs("/opt/aurelia", 20)?;
            println!("📜 日志内容:");
            println!("{}", logs);
        }
    }

    // 步骤6: 执行测试命令
    println!("\n6️⃣ 执行测试命令...");
    println!("----------------------------");
    
    let test_commands = vec![
        ("df -h /opt", "磁盘使用情况"),
        ("free -h", "内存使用情况"),
        ("ss -tlnp | grep -E '(8080|3030)' || echo '端口未监听'", "监听端口"),
        ("systemctl status aurelia --no-pager 2>/dev/null || echo 'systemd服务未配置'", "systemd服务状态"),
    ];

    for (cmd, desc) in test_commands {
        println!("\n📌 {}:", desc);
        match commander.execute_on_all(cmd).await {
            Ok(results) => {
                for (server_id, result) in results {
                    if server_id == target_server {
                        match result {
                            Ok(output) => println!("{}", output.trim()),
                            Err(e) => println!("错误: {}", e),
                        }
                    }
                }
            }
            Err(e) => {
                warn!("命令执行失败: {}", e);
            }
        }
    }

    // 最终状态
    println!("\n========================================");
    println!("  部署测试完成");
    println!("========================================");
    
    let final_status = commander.get_deployment_status().await;
    if let Some(status) = final_status.get(target_server) {
        println!("\n最终状态:");
        println!("  服务器: {} ({})", status.server_id, status.ip);
        println!("  状态: {:?}", status.status);
        if let Some(last_success) = &status.last_success {
            println!("  最后成功: {}", last_success);
        }
        
        match status.status {
            autonomy_core::deployment_commander::DeploymentState::Running => {
                println!("\n✅ 部署成功! Kernel正在远程服务器上运行。");
                println!("\n你可以通过以下方式访问:");
                println!("  SSH: ssh root@194.146.13.14");
                println!("  API: http://194.146.13.14:8080");
                println!("  监控: http://194.146.13.14:3030");
            }
            _ => {
                println!("\n⚠️ 部署可能未完全成功，请检查上述日志。");
            }
        }
    }

    Ok(())
}