# 🌐 Aurelia 智能体测试服务器配置

## 快速开始

### 方式1: 本地测试（推荐）

无需真实服务器，直接在本地运行：

```bash
# 1. 编译项目
cargo build --release

# 2. 运行测试
./scripts/test/run_local_test.sh
```

智能体将在本地演示其自主能力。

### 方式2: Docker 测试环境

使用Docker容器模拟多个Ubuntu服务器：

```bash
# 1. 启动Docker容器
cd scripts/docker && docker-compose up -d && cd ../..

# 2. 验证容器
docker ps

# 3. 部署到主节点
scp -P 2221 target/release/kernel root@localhost:/home/ubuntu/aurelia/

# 4. 启动智能体
ssh -p 2221 root@localhost 'cd /home/ubuntu/aurelia && ./kernel'
```

**Docker服务器配置：**
| 节点 | 访问地址 | SSH端口 | 内部IP | 角色 |
|-----|---------|--------|--------|-----|
| Primary | localhost | 2221 | 172.20.0.10 | 主节点 |
| Replica1 | localhost | 2222 | 172.20.0.11 | 副本1 |
| Replica2 | localhost | 2223 | 172.20.0.12 | 副本2 |
| Monitor | localhost | 2224 | 172.20.0.20 | 监控 |

### 方式3: 云服务器测试

如果你有真实的云服务器（AWS/阿里云/腾讯云等）：

#### 服务器要求
- **系统**: Ubuntu 20.04 LTS 或更高
- **配置**: 最低 1vCPU, 2GB RAM
- **网络**: 开放 22(SSH), 8080(WebSocket) 端口
- **权限**: SSH密钥登录

#### 配置文件示例

创建 `production_config.json`:

```json
{
  "test_environments": [
    {
      "name": "aws-primary",
      "ip": "YOUR_SERVER_IP",
      "port": 22,
      "user": "ubuntu",
      "ssh_key_path": "~/.ssh/your_key.pem",
      "remote_deploy_path": "/home/ubuntu/aurelia",
      "role": "primary"
    },
    {
      "name": "aws-replica",
      "ip": "YOUR_REPLICA_IP",
      "port": 22,
      "user": "ubuntu",
      "ssh_key_path": "~/.ssh/your_key.pem",
      "remote_deploy_path": "/home/ubuntu/aurelia",
      "role": "replica"
    }
  ],
  "test_settings": {
    "initial_funds": 1000.0,
    "test_duration_minutes": 60,
    "health_check_interval_seconds": 30,
    "auto_deploy_threshold": 0.8,
    "resource_limits": {
      "max_cpu_percent": 80.0,
      "max_memory_mb": 1024,
      "max_disk_gb": 10
    }
  }
}
```

#### 部署步骤

```bash
# 1. 测试连接
ssh ubuntu@YOUR_SERVER_IP

# 2. 部署智能体
cargo run --example run_test -- --config production_config.json deploy

# 3. 启动智能体
ssh ubuntu@YOUR_SERVER_IP 'cd /home/ubuntu/aurelia && nohup ./kernel > aurelia.log 2>&1 &'

# 4. 监控
cargo run --example run_test -- --config production_config.json monitor
```

## 🧪 测试场景

### 场景1: 自主健康监控
智能体会自动：
- 每30秒检查系统健康
- 监控CPU、内存、磁盘使用
- 生成健康报告

### 场景2: 自我复制
当满足条件时，智能体会：
- 扫描可用服务器
- 决定是否需要复制
- 自动部署到新节点

### 场景3: 故障恢复
模拟故障：
```bash
# 杀死进程
pkill kernel

# 观察自动恢复
tail -f aurelia.log
```

### 场景4: 任务调度
智能体会自主：
- 调度定期任务
- 管理任务优先级
- 处理任务依赖

## 📊 监控指标

查看智能体状态：

```bash
# 查看日志
tail -f aurelia.log

# 查看进程
ps aux | grep kernel

# 查看资源使用
htop

# 查看网络连接
netstat -an | grep 8080
```

## 🛠️ 故障排查

### 问题: 无法连接到Docker容器
```bash
# 检查容器状态
docker ps

# 重启容器
cd scripts/docker && docker-compose restart && cd ../..

# 查看容器日志
docker logs aurelia-primary
```

### 问题: SSH密钥权限
```bash
# 设置正确权限
chmod 600 ~/.ssh/id_rsa
chmod 644 ~/.ssh/id_rsa.pub
```

### 问题: 端口已占用
```bash
# 修改 docker-compose.yml 中的端口映射
# 例如: 2221:22 改为 2231:22
```

## 🎯 验证成功标志

智能体成功运行时，你应该看到：

```
✅ [INFO] Kernel starting...
✅ [INFO] Autonomous Agent initialized successfully
✅ [INFO] Starting autonomous health monitoring
✅ [INFO] Starting autonomous task scheduler
✅ [INFO] Starting autonomous replication management
✅ [INFO] Making autonomous decision based on current context
✅ [INFO] Executing task: System Health Check
```

## 🚀 高级测试

### 压力测试
```bash
# 增加负载
stress --cpu 2 --timeout 60s

# 观察智能体反应
watch -n 1 'ps aux | grep kernel'
```

### 网络分区测试
```bash
# 模拟网络故障
iptables -A INPUT -s 172.20.0.11 -j DROP

# 恢复网络
iptables -D INPUT -s 172.20.0.11 -j DROP
```

## 📝 测试报告

测试完成后，查看：
- `aurelia.log` - 运行日志
- `validation_results.json` - 验证结果
- `config/state.json` - 状态信息

## 🛡️ 安全提醒

1. **测试环境隔离** - 不要在生产环境测试
2. **API密钥** - 使用测试密钥，不要使用真实密钥
3. **资源限制** - 设置合理的资源限制
4. **监控告警** - 设置异常告警

## 📧 支持

如需帮助，请查看：
- [AUTONOMOUS_AGENT.md](./AUTONOMOUS_AGENT.md) - 系统架构
- [note.md](./note.md) - 实现说明
- [TEST_GUIDE.md](./TEST_GUIDE.md) - 详细测试指南