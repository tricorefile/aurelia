use autonomy_core::SshDeployer;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n====================================");
    println!("  测试腾讯云服务器SSH部署");
    println!("====================================\n");
    
    // 腾讯云服务器信息（硬编码，后续放入配置文件）
    let host = "106.54.1.130";
    let port = 22;
    let username = "ubuntu";
    let private_key_path = PathBuf::from(
        std::env::var("HOME").unwrap_or_default() + "/.ssh/tencent.pem"
    );
    
    println!("🎯 目标服务器: {}:{}", host, port);
    println!("👤 用户名: {}", username);
    println!("🔑 私钥文件: {:?}", private_key_path);
    println!("\n正在使用私钥连接...\n");
    
    // 创建SSH部署器
    let mut deployer = SshDeployer::new();
    
    // 使用私钥连接
    match deployer.connect_with_key(host, port, username, &private_key_path, None) {
        Ok(_) => {
            println!("✅ SSH私钥连接成功！\n");
            
            // 执行测试命令
            println!("执行测试命令...");
            println!("{}", "-".repeat(40));
            
            // 1. 获取系统信息
            match deployer.execute_command("hostname && uname -a") {
                Ok(output) => {
                    println!("系统信息:");
                    println!("{}", output);
                }
                Err(e) => println!("获取系统信息失败: {}", e),
            }
            
            // 2. 检查当前用户和目录
            match deployer.execute_command("whoami && pwd") {
                Ok(output) => {
                    println!("当前用户和目录:");
                    println!("{}", output);
                }
                Err(e) => println!("获取用户信息失败: {}", e),
            }
            
            // 3. 检查磁盘空间
            match deployer.execute_command("df -h /") {
                Ok(output) => {
                    println!("\n磁盘空间:");
                    println!("{}", output);
                }
                Err(e) => println!("检查磁盘失败: {}", e),
            }
            
            // 4. 检查/opt目录（需要创建或使用sudo）
            match deployer.execute_command("ls -la /opt 2>/dev/null | head -5 || echo '需要创建/opt/aurelia'") {
                Ok(output) => {
                    println!("\n/opt目录状态:");
                    println!("{}", output);
                }
                Err(e) => println!("检查/opt失败: {}", e),
            }
            
            // 5. 创建部署目录（使用用户目录，避免权限问题）
            let deploy_path = "/home/ubuntu/aurelia";
            println!("\n准备部署目录: {}", deploy_path);
            
            match deployer.create_remote_directory(deploy_path) {
                Ok(_) => println!("✅ 创建部署目录成功"),
                Err(e) => println!("创建目录失败: {}", e),
            }
            
            // 6. 检查kernel进程
            match deployer.execute_command("ps aux | grep kernel | grep -v grep") {
                Ok(output) => {
                    if output.trim().is_empty() {
                        println!("\nkernel进程: 未运行");
                    } else {
                        println!("\nkernel进程:");
                        println!("{}", output);
                    }
                }
                Err(e) => println!("检查进程失败: {}", e),
            }
            
            println!("\n{}", "-".repeat(40));
            println!("\n准备部署测试...\n");
            
            // 检查本地二进制文件
            let binary_path = PathBuf::from("./target/release/kernel");
            if !binary_path.exists() {
                println!("⚠️ 本地二进制文件不存在: {:?}", binary_path);
                println!("请先运行: cargo build --release");
            } else {
                println!("📦 准备部署: {:?}", binary_path);
                println!("📍 部署路径: {}", deploy_path);
                
                // 询问是否继续部署
                println!("\n是否部署到腾讯云服务器? (这将上传并启动kernel)");
                println!("注意: 将部署到 {}", deploy_path);
                println!("\n按Enter继续，或Ctrl+C取消...");
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).ok();
                
                println!("开始部署...");
                
                // 部署kernel到用户目录
                match deployer.deploy_kernel(
                    &binary_path,
                    deploy_path,
                    Some(vec![PathBuf::from("config/target_servers.json")]),
                ) {
                    Ok(_) => {
                        println!("✅ 文件上传成功！");
                        
                        // 启动kernel
                        println!("\n启动kernel...");
                        match deployer.start_kernel(deploy_path) {
                            Ok(_) => {
                                println!("✅ kernel已启动！");
                                
                                // 检查状态
                                std::thread::sleep(std::time::Duration::from_secs(2));
                                match deployer.check_kernel_status() {
                                    Ok(running) => {
                                        if running {
                                            println!("✅ kernel正在运行！");
                                            
                                            // 显示进程信息
                                            if let Ok(ps_output) = deployer.execute_command("ps aux | grep kernel | grep -v grep") {
                                                println!("\n进程信息:");
                                                println!("{}", ps_output);
                                            }
                                        } else {
                                            println!("⚠️ kernel未检测到运行");
                                        }
                                    }
                                    Err(e) => println!("状态检查失败: {}", e),
                                }
                                
                                // 获取日志
                                match deployer.get_logs(deploy_path, 10) {
                                    Ok(logs) => {
                                        println!("\n最新日志:");
                                        println!("{}", logs);
                                    }
                                    Err(e) => println!("获取日志失败: {}", e),
                                }
                                
                                // 检查端口监听
                                match deployer.execute_command("ss -tlnp 2>/dev/null | grep -E '(8080|3030)' || netstat -tlnp 2>/dev/null | grep -E '(8080|3030)' || echo '端口未监听'") {
                                    Ok(output) => {
                                        println!("\n端口监听状态:");
                                        println!("{}", output);
                                    }
                                    Err(e) => println!("检查端口失败: {}", e),
                                }
                            }
                            Err(e) => println!("❌ 启动失败: {}", e),
                        }
                    }
                    Err(e) => println!("❌ 部署失败: {}", e),
                }
            }
            
            println!("\n✅ 测试完成！");
            println!("\n访问方式:");
            println!("  SSH: ssh -i ~/.ssh/tencent.pem ubuntu@{}", host);
            println!("  API: http://{}:8080", host);
            println!("  监控: http://{}:3030", host);
        }
        Err(e) => {
            println!("❌ SSH连接失败: {}", e);
            println!("\n可能的原因:");
            println!("  1. 私钥文件不存在或权限错误");
            println!("  2. 服务器IP或端口不正确");
            println!("  3. 用户名错误（应该是ubuntu而不是root）");
            println!("  4. 安全组未开放SSH端口");
            
            // 检查私钥文件
            if !private_key_path.exists() {
                println!("\n⚠️ 私钥文件不存在: {:?}", private_key_path);
                println!("请确保私钥文件在正确位置");
            } else {
                println!("\n私钥文件存在，检查权限:");
                println!("  chmod 400 {:?}", private_key_path);
            }
            
            return Err(format!("SSH connection failed: {}", e).into());
        }
    }
    
    Ok(())
}