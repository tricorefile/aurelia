# Aurelia 智能体自主部署测试指南

## 概述

Aurelia 是一个具有自我复制、自主决策和目标发现能力的智能体系统。本指南描述如何测试其在 Ubuntu 服务器上的自主部署和运行能力。

## 系统架构

```
┌─────────────────────────────────────────────┐
│                   Kernel                     │
│         (核心协调器 - main.rs)               │
└──────┬──────────────────────┬────────────────┘
       │                      │
       ▼                      ▼
┌──────────────┐      ┌──────────────────────┐
│ Perception   │      │  Reasoning Engine    │
│    Core      │◄────►│   (推理引擎)         │
└──────────────┘      └──────────────────────┘
       │                      │
       ▼                      ▼
┌──────────────┐      ┌──────────────────────┐
│  Strategy    │      │ Execution Engine     │
│   Engine     │◄────►│   (执行引擎)         │
└──────────────┘      └──────────────────────┘
       │                      │
       ▼                      ▼
┌──────────────┐      ┌──────────────────────┐
│  Survival    │      │  Metamorphosis       │
│  Protocol    │◄────►│     Engine           │
└──────────────┘      └──────────────────────┘
```

## 测试环境准备

### 1. Ubuntu 服务器要求

- **操作系统**: Ubuntu 20.04 LTS 或更高版本
- **最低配置**: 2 CPU核心, 4GB RAM, 20GB 磁盘空间
- **网络**: SSH访问 (端口22), WebSocket支持 (端口8080)
- **依赖软件**: 
  - 基础工具: `curl`, `wget`, `git`
  - 监控工具: `htop`, `netstat`, `ps`

### 2. 本地环境准备

```bash
# 编译项目
cargo build --release

# 设置环境变量
cp .env.example .env
# 编辑 .env 文件，设置必要的 API 密钥

# 配置测试环境
# 编辑 test_env.json，设置目标服务器信息
vim test_env.json
```

### 3. SSH 密钥配置

```bash
# 生成 SSH 密钥对（如果还没有）
ssh-keygen -t rsa -b 4096

# 复制公钥到目标服务器
ssh-copy-id ubuntu@<server-ip>

# 测试连接
ssh ubuntu@<server-ip> "echo 'SSH connection successful'"
```

## 测试执行步骤

### 阶段 1: 基础部署测试

```bash
# 运行自动化部署脚本
./test_deployment.sh

# 脚本将执行以下步骤：
# 1. 预部署检查（SSH连接、文件完整性）
# 2. 构建部署包
# 3. 部署到目标服务器
# 4. 启动智能体
# 5. 验证运行状态
```

### 阶段 2: 自我复制测试

测试智能体的自主复制能力：

```bash
# 在主服务器上创建部署触发文件
ssh ubuntu@192.168.1.100 'cat > /home/ubuntu/aurelia_agent/deploy_trigger.json << EOF
{
    "ip": "192.168.1.101",
    "remote_user": "ubuntu",
    "private_key_path": "~/.ssh/id_rsa",
    "remote_path": "/home/ubuntu/aurelia_replica",
    "local_exe_path": "./kernel"
}
EOF'

# 监控复制过程
python3 monitor_validation.py --test replication
```

### 阶段 3: 自主行为验证

```bash
# 运行完整验证套件
python3 monitor_validation.py

# 持续监控（60分钟）
python3 monitor_validation.py --continuous 60
```

验证项目包括：
- ✅ 进程运行状态
- ✅ 资源使用情况（CPU、内存）
- ✅ 日志活动
- ✅ 自主决策行为
- ✅ 网络通信
- ✅ 自我复制能力

### 阶段 4: 故障恢复测试

```bash
# 模拟进程崩溃
ssh ubuntu@192.168.1.100 "pkill -f kernel"

# 等待自动恢复
sleep 30

# 验证恢复状态
python3 monitor_validation.py --test running
```

## 监控和验证

### 实时日志监控

```bash
# 主服务器日志
ssh ubuntu@192.168.1.100 "tail -f /home/ubuntu/aurelia_agent/aurelia.log"

# 副本服务器日志
ssh ubuntu@192.168.1.101 "tail -f /home/ubuntu/aurelia_replica/aurelia.log"
```

### 性能监控

```bash
# CPU 和内存使用
ssh ubuntu@192.168.1.100 "htop"

# 网络连接
ssh ubuntu@192.168.1.100 "netstat -an | grep ESTABLISHED"

# 进程详情
ssh ubuntu@192.168.1.100 "ps aux | grep kernel"
```

### 验证指标

| 测试项 | 预期结果 | 验证方法 |
|--------|----------|----------|
| 进程运行 | kernel进程持续运行 | `ps aux \| grep kernel` |
| CPU使用 | < 80% | `top -bn1` |
| 内存使用 | < 1GB | `pmap <pid>` |
| 日志生成 | 持续产生日志 | `tail -f aurelia.log` |
| 自主决策 | 策略决策事件 | 日志中包含 `StrategyDecision` |
| 网络通信 | WebSocket连接活跃 | `netstat -an \| grep 8080` |
| 自我复制 | 成功部署到副本服务器 | 副本服务器上存在kernel进程 |

## 测试结果分析

验证结果保存在 `validation_results.json`：

```json
{
  "start_time": "2024-01-01T10:00:00",
  "tests": [
    {
      "name": "agent_running",
      "server": "192.168.1.100",
      "passed": true
    },
    ...
  ],
  "summary": {
    "total_tests": 12,
    "passed": 11,
    "failed": 1,
    "success_rate": 91.7
  }
}
```

## 故障排查

### 常见问题

1. **SSH连接失败**
   ```bash
   # 检查SSH服务
   systemctl status ssh
   # 检查防火墙
   ufw status
   ```

2. **部署失败**
   ```bash
   # 检查目标目录权限
   ls -la /home/ubuntu/
   # 检查磁盘空间
   df -h
   ```

3. **智能体未启动**
   ```bash
   # 查看错误日志
   tail -n 100 aurelia.log | grep ERROR
   # 检查依赖
   ldd kernel
   ```

## 安全注意事项

1. **测试环境隔离**: 建议在隔离的测试网络中进行
2. **API密钥管理**: 使用测试专用的API密钥
3. **资源限制**: 设置适当的资源限制防止失控
4. **日志清理**: 定期清理测试日志避免磁盘满

## 清理测试环境

```bash
# 停止所有智能体进程
ssh ubuntu@192.168.1.100 "pkill -f kernel"
ssh ubuntu@192.168.1.101 "pkill -f kernel"

# 清理部署文件
ssh ubuntu@192.168.1.100 "rm -rf /home/ubuntu/aurelia_*"
ssh ubuntu@192.168.1.101 "rm -rf /home/ubuntu/aurelia_*"

# 清理本地测试文件
rm -f validation_results.json test_deployment.log
```

## 高级测试场景

### 1. 多节点集群测试

编辑 `test_env.json` 添加更多服务器节点，测试大规模部署。

### 2. 网络分区测试

模拟网络故障，测试智能体的容错能力。

### 3. 负载测试

增加市场数据频率，测试高负载下的表现。

### 4. 热更新测试

测试 Metamorphosis Engine 的代码热更新能力。

## 总结

通过以上测试流程，可以全面验证 Aurelia 智能体的：
- 🚀 自主部署能力
- 🔄 自我复制能力
- 🧠 自主决策能力
- 📊 资源管理能力
- 🛡️ 故障恢复能力

测试通过后，系统即可在生产环境中自主运行。