use autonomy_core::SshDeployer;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n====================================");
    println!("  测试纯Rust SSH部署");
    println!("====================================\n");

    let host = "194.146.13.14";
    let port = 22;
    let username = "root";
    let password = "A8vd0Cz400fCC1b";

    println!("🎯 目标服务器: {}:{}", host, port);
    println!("👤 用户名: {}", username);
    println!("\n正在连接...\n");

    // 创建SSH部署器
    let mut deployer = SshDeployer::new();

    // 测试连接
    match deployer.connect_with_password(host, port, username, password) {
        Ok(_) => {
            println!("✅ SSH连接成功！\n");

            // 执行一些测试命令
            println!("执行测试命令...");
            println!("{}", "-".repeat(40));

            // 1. 获取主机名
            match deployer.execute_command("hostname") {
                Ok(output) => println!("主机名: {}", output.trim()),
                Err(e) => println!("获取主机名失败: {}", e),
            }

            // 2. 检查当前目录
            match deployer.execute_command("pwd") {
                Ok(output) => println!("当前目录: {}", output.trim()),
                Err(e) => println!("获取目录失败: {}", e),
            }

            // 3. 检查/opt目录
            match deployer.execute_command("ls -la /opt/ 2>/dev/null | head -5") {
                Ok(output) => {
                    println!("\n/opt目录内容:");
                    println!("{}", output);
                }
                Err(e) => println!("列出目录失败: {}", e),
            }

            // 4. 检查kernel进程
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

            // 5. 检查磁盘空间
            match deployer.execute_command("df -h /opt") {
                Ok(output) => {
                    println!("\n磁盘空间:");
                    println!("{}", output);
                }
                Err(e) => println!("检查磁盘失败: {}", e),
            }

            println!("\n{}", "-".repeat(40));
            println!("\n现在测试部署功能...\n");

            // 检查本地二进制文件
            let binary_path = PathBuf::from("./target/release/kernel");
            if !binary_path.exists() {
                println!("⚠️ 本地二进制文件不存在: {:?}", binary_path);
                println!("请先运行: cargo build --release");
            } else {
                println!("📦 准备部署: {:?}", binary_path);

                // 询问是否继续部署
                println!("\n是否部署到服务器? (这将上传并启动kernel)");
                println!("注意: 这将覆盖 /opt/aurelia 目录");
                println!("\n按Enter继续，或Ctrl+C取消...");

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).ok();

                println!("开始部署...");

                // 部署kernel
                match deployer.deploy_kernel(
                    &binary_path,
                    "/opt/aurelia",
                    Some(vec![PathBuf::from("config/target_servers.json")]),
                ) {
                    Ok(_) => {
                        println!("✅ 文件上传成功！");

                        // 启动kernel
                        println!("\n启动kernel...");
                        match deployer.start_kernel("/opt/aurelia") {
                            Ok(_) => {
                                println!("✅ kernel已启动！");

                                // 检查状态
                                std::thread::sleep(std::time::Duration::from_secs(2));
                                match deployer.check_kernel_status() {
                                    Ok(running) => {
                                        if running {
                                            println!("✅ kernel正在运行！");
                                        } else {
                                            println!("⚠️ kernel未检测到运行");
                                        }
                                    }
                                    Err(e) => println!("状态检查失败: {}", e),
                                }

                                // 获取日志
                                match deployer.get_logs("/opt/aurelia", 10) {
                                    Ok(logs) => {
                                        println!("\n最新日志:");
                                        println!("{}", logs);
                                    }
                                    Err(e) => println!("获取日志失败: {}", e),
                                }
                            }
                            Err(e) => println!("❌ 启动失败: {}", e),
                        }
                    }
                    Err(e) => println!("❌ 部署失败: {}", e),
                }
            }

            println!("\n✅ 测试完成！");
        }
        Err(e) => {
            println!("❌ SSH连接失败: {}", e);
            println!("\n可能的原因:");
            println!("  1. 服务器IP或端口不正确");
            println!("  2. 用户名或密码错误");
            println!("  3. 服务器防火墙阻止连接");
            println!("  4. SSH服务未运行");
            return Err(format!("SSH connection failed: {}", e).into());
        }
    }

    Ok(())
}
