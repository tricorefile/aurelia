# Aurelia 部署指南

## 概述

Aurelia支持智能部署，能够自动检测目标服务器的架构并从GitHub Release下载对应的二进制文件进行部署。

## 支持的平台

- **Linux x86_64** - 标准Linux发行版（Ubuntu, Debian, CentOS等）
- **Linux x86_64 (musl)** - Alpine Linux或需要静态链接的环境
- **Linux ARM64** - ARM架构服务器（树莓派、AWS Graviton等）
- **Windows x86_64** - Windows Server

## 快速部署

### 方法1：使用Bash脚本

```bash
# 部署最新版本
./scripts/smart_deploy.sh latest YOUR_SERVER_IP

# 部署特定版本
./scripts/smart_deploy.sh v1.0.0 YOUR_SERVER_IP ~/.ssh/custom_key root

# 完整参数
./scripts/smart_deploy.sh [release_tag] <server_ip> [ssh_key] [ssh_user]
```

### 方法2：使用Python脚本

```bash
# 部署最新版本
python3 py/smart_deploy.py YOUR_SERVER_IP

# 部署特定版本
python3 py/smart_deploy.py YOUR_SERVER_IP --tag v1.0.0

# 指定用户和密钥
python3 py/smart_deploy.py YOUR_SERVER_IP \
  --user root \
  --key ~/.ssh/deploy_key \
  --tag v1.0.0 \
  --path /opt/aurelia
```

### 方法3：通过GitHub Actions

1. 在仓库设置中添加Secrets：
   - `SSH_PRIVATE_KEY` - SSH私钥
   - `DEFAULT_SERVER_IP` - 默认服务器IP
   - `SERVER_USER` - SSH用户名（可选，默认ubuntu）

2. 触发部署：
   - 访问Actions页面
   - 选择"Deploy" workflow
   - 点击"Run workflow"
   - 选择环境和版本

## 智能部署流程

1. **架构检测** - 自动检测目标服务器的CPU架构和操作系统
2. **二进制选择** - 根据架构选择合适的预编译二进制文件
3. **下载** - 从GitHub Release下载对应版本
4. **部署** - 上传到服务器并配置
5. **服务管理** - 创建systemd服务并启动

## 部署脚本功能

### 自动架构检测

脚本会自动检测并选择正确的二进制文件：

| 服务器架构 | 操作系统 | 选择的二进制 |
|-----------|---------|-------------|
| x86_64 | Ubuntu/Debian | aurelia-linux-x86_64 |
| x86_64 | Alpine | aurelia-linux-x86_64-musl |
| aarch64 | Any Linux | aurelia-linux-aarch64 |

### Systemd服务配置

部署脚本会自动创建systemd服务：

```ini
[Unit]
Description=Aurelia Autonomous System
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/opt/aurelia
ExecStart=/opt/aurelia/kernel
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

## 手动部署步骤

如果需要手动部署，请按以下步骤操作：

### 1. 检测服务器架构

```bash
# SSH到服务器
ssh user@your-server

# 检查架构
uname -m

# 检查操作系统
cat /etc/os-release | grep "^ID="
```

### 2. 下载对应的二进制文件

访问 [Releases页面](https://github.com/tricorefile/aurelia/releases) 下载对应版本。

```bash
# Linux x86_64
wget https://github.com/tricorefile/aurelia/releases/latest/download/aurelia-linux-x86_64.tar.gz

# 解压
tar xzf aurelia-linux-x86_64.tar.gz
```

### 3. 安装和配置

```bash
# 创建目录
sudo mkdir -p /opt/aurelia

# 复制二进制文件
sudo cp kernel /opt/aurelia/
sudo chmod +x /opt/aurelia/kernel

# 创建systemd服务
sudo tee /etc/systemd/system/aurelia.service > /dev/null << 'EOF'
[Unit]
Description=Aurelia Autonomous System
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/opt/aurelia
ExecStart=/opt/aurelia/kernel
Restart=always

[Install]
WantedBy=multi-user.target
EOF

# 启动服务
sudo systemctl daemon-reload
sudo systemctl start aurelia
sudo systemctl enable aurelia
```

## 服务管理

### 查看状态

```bash
sudo systemctl status aurelia
```

### 查看日志

```bash
# 实时日志
sudo journalctl -u aurelia -f

# 最近100行
sudo journalctl -u aurelia -n 100

# 特定时间范围
sudo journalctl -u aurelia --since "1 hour ago"
```

### 重启服务

```bash
sudo systemctl restart aurelia
```

### 停止服务

```bash
sudo systemctl stop aurelia
```

## 配置文件

默认配置文件位置：`/opt/aurelia/config/`

主要配置文件：
- `target_servers.json` - 目标服务器列表
- `.env` - 环境变量（如API密钥）

## 故障排除

### 服务无法启动

1. 检查日志：
```bash
sudo journalctl -u aurelia -n 50
```

2. 检查文件权限：
```bash
ls -la /opt/aurelia/
```

3. 手动运行测试：
```bash
sudo -u ubuntu /opt/aurelia/kernel
```

### 架构不匹配

如果出现"cannot execute binary file"错误，说明二进制文件与服务器架构不匹配。使用智能部署脚本会自动选择正确的版本。

### 网络问题

确保服务器能访问GitHub：
```bash
curl -I https://github.com
```

## 安全建议

1. **使用专用部署密钥** - 不要使用个人SSH密钥
2. **限制权限** - 服务以非root用户运行
3. **配置防火墙** - 只开放必要端口
4. **定期更新** - 及时部署安全更新

## Python脚本API

```python
from py.smart_deploy import DeploymentManager

# 创建部署管理器
manager = DeploymentManager("tricorefile/aurelia")

# 部署到服务器
manager.deploy(
    host="192.168.1.100",
    user="ubuntu",
    key_path="/path/to/key",
    tag="v1.0.0",
    deploy_path="/opt/aurelia"
)
```

## 环境变量

部署脚本支持以下环境变量：

- `GITHUB_TOKEN` - GitHub API令牌（用于私有仓库）
- `DEPLOY_PATH` - 默认部署路径
- `SSH_USER` - 默认SSH用户
- `SSH_KEY` - 默认SSH密钥路径

## 更多信息

- [GitHub Actions文档](./GITHUB_ACTIONS.md)
- [系统架构文档](./ARCHITECTURE.md)
- [API文档](./API.md)