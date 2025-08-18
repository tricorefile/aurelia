# Aurelia 服务器配置管理系统

## 概述

Aurelia 现在支持通过配置文件管理目标服务器，用于自主复制和部署。该系统提供了灵活的服务器管理功能，支持动态添加、删除、更新服务器配置。

## 系统架构

```
┌─────────────────────────────────────────┐
│           server_manager.py             │
│         (Python 管理工具)                │
└─────────────┬───────────────────────────┘
              │ 读写
              ▼
┌─────────────────────────────────────────┐
│      config/target_servers.json         │
│         (JSON 配置文件)                  │
└─────────────┬───────────────────────────┘
              │ 读取
              ▼
┌─────────────────────────────────────────┐
│        server_config.rs                 │
│      (Rust 配置模块)                     │
└─────────────┬───────────────────────────┘
              │ 使用
              ▼
┌─────────────────────────────────────────┐
│       self_replicator.rs                │
│     (自主复制模块)                       │
└─────────────────────────────────────────┘
```

## 配置文件格式

配置文件位于 `config/target_servers.json`：

```json
{
  "target_servers": [
    {
      "id": "server-1",
      "name": "Production Server 1",
      "ip": "192.168.1.101",
      "port": 22,
      "username": "ubuntu",
      "ssh_key_path": "~/.ssh/id_rsa",
      "remote_path": "/home/ubuntu/aurelia",
      "enabled": true,
      "priority": 1,
      "tags": ["production", "primary"],
      "max_retries": 3,
      "retry_delay_seconds": 60
    }
  ],
  "default_settings": {
    "port": 22,
    "username": "ubuntu",
    "ssh_key_path": "~/.ssh/id_rsa",
    "remote_path": "/home/ubuntu/aurelia"
  },
  "deployment_strategy": {
    "parallel_deployments": 2,
    "deployment_timeout_seconds": 300,
    "health_check_interval_seconds": 30
  }
}
```

## 使用方法

### 1. Python 管理工具 (server_manager.py)

#### 列出所有服务器
```bash
python3 server_manager.py list
```

#### 添加新服务器
```bash
python3 server_manager.py add <id> <name> <ip> <username> \
  --port 22 \
  --ssh-key ~/.ssh/id_rsa \
  --remote-path /home/ubuntu/aurelia \
  --priority 10 \
  --tags "production,primary" \
  --max-retries 3 \
  --retry-delay 60
```

#### 更新服务器配置
```bash
python3 server_manager.py update <id> \
  --name "New Name" \
  --ip "192.168.1.200" \
  --priority 5 \
  --tags "production,backup"
```

#### 启用/禁用服务器
```bash
python3 server_manager.py enable <id>
python3 server_manager.py disable <id>
```

#### 删除服务器
```bash
python3 server_manager.py remove <id>
```

#### 显示服务器详情
```bash
python3 server_manager.py show <id>
```

#### 测试服务器连接
```bash
python3 server_manager.py test <id>
```

### 2. Rust 代码集成

在 Rust 代码中使用配置：

```rust
use autonomy_core::{ServerConfig, TargetServer};

// 加载配置
let config = ServerConfig::from_file("config/target_servers.json")?;

// 获取启用的服务器
let enabled_servers = config.get_enabled_servers();

// 按优先级获取服务器
let prioritized = config.get_servers_by_priority();

// 添加新服务器
let new_server = TargetServer {
    id: "new-server".to_string(),
    name: "New Server".to_string(),
    ip: "192.168.1.200".to_string(),
    port: 22,
    username: "ubuntu".to_string(),
    ssh_key_path: "~/.ssh/id_rsa".to_string(),
    remote_path: "/home/ubuntu/aurelia".to_string(),
    enabled: true,
    priority: 10,
    tags: vec!["test".to_string()],
    max_retries: 3,
    retry_delay_seconds: 60,
};
config.add_server(new_server)?;

// 保存配置
config.save_to_file("config/target_servers.json")?;
```

### 3. 自主复制器集成

SelfReplicator 会自动从配置文件加载服务器：

```rust
use autonomy_core::SelfReplicator;
use std::path::PathBuf;

// 创建自主复制器，自动加载配置
let replicator = SelfReplicator::new(PathBuf::from("target/release/kernel"));

// 获取配置的服务器
let servers = replicator.get_configured_servers();

// 重新加载配置
replicator.reload_targets_from_config().await?;

// 添加服务器到配置并保存
replicator.add_server_to_config(new_server).await?;
```

## 服务器配置字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | string | 是 | 服务器唯一标识符 |
| name | string | 是 | 服务器名称 |
| ip | string | 是 | 服务器IP地址 |
| port | number | 否 | SSH端口，默认22 |
| username | string | 是 | SSH用户名 |
| ssh_key_path | string | 是 | SSH密钥路径 |
| remote_path | string | 是 | 远程部署路径 |
| enabled | boolean | 否 | 是否启用，默认true |
| priority | number | 否 | 优先级(越小越高)，默认100 |
| tags | array | 否 | 标签列表 |
| max_retries | number | 否 | 最大重试次数，默认3 |
| retry_delay_seconds | number | 否 | 重试延迟秒数，默认60 |

## 部署策略配置

| 字段 | 说明 |
|------|------|
| parallel_deployments | 并行部署数量 |
| deployment_timeout_seconds | 部署超时时间 |
| health_check_interval_seconds | 健康检查间隔 |

## 注意事项

1. **SSH密钥路径**: 支持 `~` 符号，会自动展开为用户主目录
2. **优先级**: 数字越小优先级越高，系统会优先选择高优先级服务器
3. **标签**: 可用于分组管理服务器，如 "production", "development", "backup" 等
4. **启用状态**: 只有 `enabled: true` 的服务器才会被用于部署
5. **配置持久化**: 所有修改都会自动保存到配置文件

## 测试工具

运行集成测试：
```bash
# 测试服务器管理功能
python3 test_server_config.py

# 测试 Rust 集成
python3 test_config_integration.py
```

## 故障排除

1. **SSH连接失败**: 确保SSH密钥文件存在且有正确权限 (chmod 600)
2. **配置文件损坏**: 备份在 `config/target_servers.json.bak`
3. **服务器不可达**: 使用 `test` 命令测试连接性
4. **权限问题**: 确保远程路径有写入权限

## 最佳实践

1. 定期测试服务器连接性
2. 为不同环境使用标签管理
3. 设置合理的重试次数和延迟
4. 保持配置文件的版本控制
5. 使用优先级控制部署顺序