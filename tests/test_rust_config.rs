use autonomy_core::{SelfReplicator, ServerConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();
    
    println!("测试 Rust 服务器配置集成\n");
    
    // 1. 直接加载配置文件
    println!("1. 从配置文件加载服务器列表:");
    match ServerConfig::from_file("config/target_servers.json") {
        Ok(config) => {
            println!("   ✅ 成功加载配置文件");
            println!("   • 服务器总数: {}", config.target_servers.len());
            println!("   • 启用的服务器: {}", config.get_enabled_servers().len());
            
            println!("\n   按优先级排序的服务器:");
            for server in config.get_servers_by_priority() {
                if server.enabled {
                    println!("     - {} ({}): 优先级 {}", 
                        server.name, server.ip, server.priority);
                }
            }
        }
        Err(e) => {
            println!("   ❌ 加载配置失败: {}", e);
        }
    }
    
    // 2. 测试 SelfReplicator 集成
    println!("\n2. 测试 SelfReplicator 集成:");
    let binary_path = PathBuf::from("target/release/kernel");
    let replicator = SelfReplicator::new(binary_path);
    
    let configured_servers = replicator.get_configured_servers();
    println!("   • SelfReplicator 加载的服务器: {}", configured_servers.len());
    
    for server in configured_servers.iter().take(3) {
        println!("     - {}: {} (启用: {})", 
            server.id, server.ip, if server.enabled { "是" } else { "否" });
    }
    
    // 3. 测试状态获取
    println!("\n3. 获取复制状态:");
    let status = replicator.get_status().await;
    println!("   • 活跃副本: {}", status.active_replicas);
    println!("   • 可用目标: {}", status.total_targets);
    println!("   • 最近失败: {}", status.recent_failures);
    println!("   • 策略:");
    println!("     - 最大副本: {}", status.strategy.max_replicas);
    println!("     - 最小副本: {}", status.strategy.min_replicas);
    println!("     - 自动扩展: {}", status.strategy.auto_scale);
    
    println!("\n✅ 配置系统集成测试完成!");
}