use autonomy_core::SshDeployer;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n====================================");
    println!("  测试SSH私钥登录");
    println!("====================================\n");
    
    let host = "194.146.13.14";
    let port = 22;
    let username = "root";
    
    // 私钥路径
    let private_key_path = PathBuf::from(
        std::env::var("HOME").unwrap_or_default() + "/.ssh/id_rsa"
    );
    
    println!("🎯 目标服务器: {}:{}", host, port);
    println!("👤 用户名: {}", username);
    println!("🔑 私钥文件: {:?}", private_key_path);
    
    // 创建SSH部署器
    let mut deployer = SshDeployer::new();
    
    // 检查私钥文件是否存在
    if !private_key_path.exists() {
        println!("\n⚠️ 私钥文件不存在: {:?}", private_key_path);
        println!("\n生成SSH密钥对的方法：");
        println!("  ssh-keygen -t rsa -b 4096 -f ~/.ssh/id_rsa");
        println!("\n将公钥添加到服务器：");
        println!("  ssh-copy-id {}@{}", username, host);
        println!("  或手动将公钥内容添加到服务器的 ~/.ssh/authorized_keys");
        return Ok(());
    }
    
    println!("\n正在使用私钥连接...\n");
    
    // 尝试使用私钥连接（无密码短语）
    match deployer.connect_with_key(host, port, username, &private_key_path, None) {
        Ok(_) => {
            println!("✅ SSH私钥连接成功！\n");
            
            // 执行测试命令
            println!("执行测试命令...");
            println!("{}", "-".repeat(40));
            
            match deployer.execute_command("hostname && whoami && pwd") {
                Ok(output) => {
                    println!("服务器信息:");
                    println!("{}", output);
                }
                Err(e) => println!("命令执行失败: {}", e),
            }
            
            match deployer.execute_command("ls -la /opt/ | head -5") {
                Ok(output) => {
                    println!("\n/opt目录:");
                    println!("{}", output);
                }
                Err(e) => println!("列出目录失败: {}", e),
            }
            
            println!("\n✅ 私钥认证测试成功！");
        }
        Err(e) => {
            println!("❌ 私钥连接失败: {}", e);
            
            // 如果私钥有密码短语，尝试询问
            println!("\n如果私钥有密码短语，请输入：");
            let mut passphrase = String::new();
            std::io::stdin().read_line(&mut passphrase).ok();
            let passphrase = passphrase.trim();
            
            if !passphrase.is_empty() {
                println!("\n使用密码短语重试...");
                
                let mut deployer2 = SshDeployer::new();
                match deployer2.connect_with_key(host, port, username, &private_key_path, Some(passphrase)) {
                    Ok(_) => {
                        println!("✅ 使用密码短语连接成功！");
                        
                        match deployer2.execute_command("hostname") {
                            Ok(output) => println!("主机名: {}", output.trim()),
                            Err(e) => println!("命令执行失败: {}", e),
                        }
                    }
                    Err(e2) => {
                        println!("❌ 使用密码短语也失败: {}", e2);
                    }
                }
            }
            
            println!("\n可能的解决方案：");
            println!("1. 确保私钥文件权限正确：");
            println!("   chmod 600 ~/.ssh/id_rsa");
            println!("\n2. 生成新的密钥对：");
            println!("   ssh-keygen -t rsa -b 4096");
            println!("\n3. 将公钥添加到服务器：");
            println!("   cat ~/.ssh/id_rsa.pub | ssh {}@{} 'cat >> ~/.ssh/authorized_keys'", username, host);
            println!("\n4. 或使用密码登录后添加公钥：");
            println!("   ssh-copy-id -i ~/.ssh/id_rsa.pub {}@{}", username, host);
        }
    }
    
    Ok(())
}