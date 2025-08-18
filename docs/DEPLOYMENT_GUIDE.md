# Aurelia 远程部署指南

## 快速开始

### 1. 基本部署命令

```bash
# 使用SSH密钥部署
./deploy.sh deploy 194.146.13.14 -u root -k ~/.ssh/id_rsa

# 使用密码部署（需要安装sshpass）
./deploy.sh deploy 194.146.13.14 -u root -P 'your_password'

# 启动服务
./deploy.sh start 194.146.13.14

# 查看状态
./deploy.sh status 194.146.13.14

# 查看日志
./deploy.sh logs 194.146.13.14
```

### 2. 使用远程执行助手

```bash
# 列出所有配置的服务器
./remote_exec.sh list

# 快速连接到服务器
./remote_exec.sh connect 1

# 在所有服务器上执行命令
./remote_exec.sh exec 'df -h'

# 监控所有服务器
./remote_exec.sh monitor

# 批量启动所有服务
./remote_exec.sh start-all

# 查看所有服务器日志
./remote_exec.sh logs-all
```

## 详细说明

### deploy.sh - 主部署脚本

#### 功能列表

| 命令 | 说明 | 示例 |
|------|------|------|
| deploy | 完整部署到服务器 | `./deploy.sh deploy 192.168.1.10` |
| start | 启动Aurelia服务 | `./deploy.sh start 192.168.1.10` |
| stop | 停止Aurelia服务 | `./deploy.sh stop 192.168.1.10` |
| status | 查看服务状态 | `./deploy.sh status 192.168.1.10` |
| logs | 查看服务日志 | `./deploy.sh logs 192.168.1.10` |
| update | 更新二进制文件 | `./deploy.sh update 192.168.1.10` |
| clean | 清理远程目录 | `./deploy.sh clean 192.168.1.10` |

#### 参数选项

- `-u <user>`: SSH用户名（默认: root）
- `-p <path>`: 远程部署路径（默认: /opt/aurelia）
- `-k <key_path>`: SSH密钥路径
- `-P <password>`: SSH密码（需要sshpass）

### remote_exec.sh - 批量管理工具

#### 功能特性

1. **服务器列表管理**
   - 从 `config/target_servers.json` 自动读取服务器配置
   - 支持密码和密钥两种认证方式

2. **批量操作**
   - 在所有启用的服务器上同时执行命令
   - 批量同步文件到所有服务器
   - 批量启动/停止服务

3. **实时监控**
   - 监控所有服务器的运行状态
   - 显示CPU和内存使用情况
   - 实时查看最新日志

## 部署流程

### 步骤1: 配置服务器信息

编辑 `config/target_servers.json`：

```json
{
  "servers": [
    {
      "name": "production-server",
      "ip": "194.146.13.14",
      "port": 22,
      "username": "root",
      "auth_method": "password",
      "password": "your_password",
      "remote_path": "/opt/aurelia",
      "enabled": true
    }
  ]
}
```

### 步骤2: 首次部署

```bash
# 编译并部署到服务器
./deploy.sh deploy 194.146.13.14 -P 'your_password'

# 或使用批量部署（部署到所有配置的服务器）
cargo build --release
./remote_exec.sh sync target/release/kernel /opt/aurelia/
```

### 步骤3: 启动服务

```bash
# 单个服务器
./deploy.sh start 194.146.13.14

# 所有服务器
./remote_exec.sh start-all
```

### 步骤4: 监控运行

```bash
# 实时监控面板
./remote_exec.sh monitor

# 查看日志
./deploy.sh logs 194.146.13.14

# 批量查看日志
./remote_exec.sh logs-all
```

## 故障排除

### 1. SSH连接问题

如果遇到SSH连接错误：

```bash
# 测试连接
ssh -p 22 root@194.146.13.14

# 使用密码时安装sshpass（macOS）
brew install hudochenkov/sshpass/sshpass
```

### 2. 服务启动失败

```bash
# 检查服务状态
./deploy.sh status 194.146.13.14

# 查看详细错误日志
ssh root@194.146.13.14 "journalctl -u aurelia -n 100"

# 手动启动调试
ssh root@194.146.13.14
cd /opt/aurelia
./kernel
```

### 3. 权限问题

```bash
# 确保二进制文件可执行
ssh root@194.146.13.14 "chmod +x /opt/aurelia/kernel"

# 检查目录权限
ssh root@194.146.13.14 "ls -la /opt/aurelia/"
```

## 生产环境建议

### 1. 安全配置

- 使用SSH密钥代替密码认证
- 配置防火墙规则，只开放必要端口
- 定期更新系统和依赖

### 2. 监控设置

```bash
# 设置定时监控任务
crontab -e
# 添加：
*/5 * * * * /path/to/remote_exec.sh exec 'systemctl status aurelia' >> /var/log/aurelia-monitor.log
```

### 3. 备份策略

```bash
# 备份配置和日志
./remote_exec.sh exec 'tar -czf /backup/aurelia-$(date +%Y%m%d).tar.gz /opt/aurelia/config /opt/aurelia/logs'
```

### 4. 性能优化

- 调整系统资源限制
- 配置日志轮转
- 优化网络参数

```bash
# 设置资源限制
./remote_exec.sh exec 'echo "* soft nofile 65536" >> /etc/security/limits.conf'
./remote_exec.sh exec 'echo "* hard nofile 65536" >> /etc/security/limits.conf'
```

## 常用命令组合

```bash
# 完整部署流程
cargo build --release && ./deploy.sh deploy 194.146.13.14 && ./deploy.sh start 194.146.13.14

# 批量更新所有服务器
cargo build --release && ./remote_exec.sh sync target/release/kernel && ./remote_exec.sh exec 'systemctl restart aurelia'

# 收集所有服务器日志
./remote_exec.sh exec 'tail -100 /opt/aurelia/logs/aurelia.log' > all-servers-logs.txt

# 检查所有服务器资源使用
./remote_exec.sh exec 'df -h && free -h && ps aux | grep kernel'
```

## 注意事项

1. **首次部署前**确保：
   - 目标服务器可以SSH访问
   - 有足够的磁盘空间
   - 必要的端口未被占用

2. **密码认证**需要安装 `sshpass`：
   ```bash
   # macOS
   brew install hudochenkov/sshpass/sshpass
   
   # Ubuntu/Debian
   apt-get install sshpass
   
   # CentOS/RHEL
   yum install sshpass
   ```

3. **监控端口**：
   - API端口: 8080
   - 监控面板: 3030
   - 确保防火墙允许这些端口

4. **日志管理**：
   - 日志默认位置: `/opt/aurelia/logs/`
   - 建议配置日志轮转避免磁盘占满

5. **更新策略**：
   - 更新前先备份配置
   - 使用蓝绿部署减少停机时间
   - 监控更新后的系统状态