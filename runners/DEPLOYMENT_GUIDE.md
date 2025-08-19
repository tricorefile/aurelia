# 在指定服务器上部署 GitHub Actions Runner 详细指南

## 前置准备

### 1. 服务器要求
- Ubuntu 20.04+ 或 CentOS 7+
- 最少 2 CPU, 4GB RAM
- 20GB+ 可用磁盘空间
- Docker 和 Docker Compose
- 稳定的网络连接

### 2. 获取 GitHub Token

#### 方法一：使用 Personal Access Token (PAT) - 推荐
```bash
# 1. 访问 GitHub Settings
https://github.com/settings/tokens

# 2. 创建 Fine-grained personal access token
# 3. 设置权限:
#    - Actions: Read
#    - Administration: Read & Write  
#    - Metadata: Read
# 4. 复制生成的 token (格式: ghp_xxxxxxxxxxxx)
```

#### 方法二：使用 Registration Token
```bash
# 访问仓库的 Actions 设置
https://github.com/tricorefile/aurelia/settings/actions/runners

# 点击 "New self-hosted runner"
# 复制 token (格式很长，包含 AAAA)
```

## 部署步骤

### 步骤 1: 登录目标服务器

```bash
# 使用 SSH 登录到你的服务器
ssh root@你的服务器IP

# 例如：
ssh root@106.54.1.130
```

### 步骤 2: 安装 Docker（如果未安装）

```bash
# 更新系统
apt-get update && apt-get upgrade -y

# 安装 Docker
curl -fsSL https://get.docker.com | bash

# 添加当前用户到 docker 组
usermod -aG docker $USER

# 安装 Docker Compose
curl -L "https://github.com/docker/compose/releases/download/v2.23.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
chmod +x /usr/local/bin/docker-compose

# 验证安装
docker --version
docker-compose --version

# 启动 Docker 服务
systemctl start docker
systemctl enable docker
```

### 步骤 3: 克隆项目并准备 Runner 文件

```bash
# 创建工作目录
mkdir -p /opt/github-runners
cd /opt/github-runners

# 克隆仓库（或只复制 runners 目录）
git clone https://github.com/tricorefile/aurelia.git
cd aurelia/runners

# 或者直接下载 runners 目录
wget https://github.com/tricorefile/aurelia/archive/refs/heads/main.zip
unzip main.zip
cd aurelia-main/runners
```

### 步骤 4: 配置环境变量

```bash
# 复制环境变量模板
cp .env.example .env

# 编辑 .env 文件
nano .env

# 添加你的 GitHub Token
GITHUB_TOKEN=ghp_你的token这里
# 或
GITHUB_TOKEN=AAAAB3NzaC1yc2EAAAADAQABAAACAQC...（registration token）

# 保存并退出 (Ctrl+X, Y, Enter)
```

### 步骤 5: 配置 SSH 密钥（用于部署）

```bash
# 创建 SSH 目录
mkdir -p ssh

# 如果你已有部署密钥，复制过来
# 从本地复制：
# scp ~/.ssh/aurelia_deploy root@服务器IP:/opt/github-runners/aurelia/runners/ssh/

# 或者生成新的密钥
ssh-keygen -t rsa -b 4096 -f ssh/aurelia_deploy -N ""

# 设置权限
chmod 600 ssh/aurelia_deploy
chmod 644 ssh/aurelia_deploy.pub

# 将公钥添加到目标部署服务器
cat ssh/aurelia_deploy.pub
# 复制输出，添加到目标服务器的 ~/.ssh/authorized_keys
```

### 步骤 6: 构建并启动 Runner

```bash
# 给脚本执行权限
chmod +x deploy-runner.sh entrypoint.sh

# 执行初始化设置
./deploy-runner.sh setup

# 启动所有 runners
./deploy-runner.sh start all

# 或只启动生产环境 runners
./deploy-runner.sh start prod

# 或只启动单个 runner
./deploy-runner.sh start single
```

### 步骤 7: 验证 Runner 状态

```bash
# 查看本地容器状态
docker-compose ps

# 查看 runner 日志
./deploy-runner.sh logs runner-1

# 检查 runner 状态
./deploy-runner.sh status
```

在 GitHub 上验证：
1. 访问 https://github.com/tricorefile/aurelia/settings/actions/runners
2. 应该能看到你的 runner 显示为 "Idle" (空闲) 状态

## 高级配置

### 自定义 Runner 标签

编辑 `docker-compose.yml`：

```yaml
services:
  runner-1:
    environment:
      - RUNNER_NAME=tencent-cloud-runner
      - RUNNER_LABELS=self-hosted,linux,x64,docker,aurelia,tencent,production
```

### 配置多个 Runner

```yaml
# docker-compose.yml
services:
  runner-gpu:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: aurelia-runner-gpu
    environment:
      - GITHUB_TOKEN=${GITHUB_TOKEN}
      - RUNNER_NAME=gpu-runner
      - RUNNER_LABELS=self-hosted,linux,x64,docker,aurelia,gpu
    # ... 其他配置
```

### 设置为系统服务（开机自启）

```bash
# 创建 systemd 服务文件
cat > /etc/systemd/system/github-runner.service << 'EOF'
[Unit]
Description=GitHub Actions Runner for Aurelia
Requires=docker.service
After=docker.service

[Service]
Type=forking
RemainAfterExit=yes
WorkingDirectory=/opt/github-runners/aurelia/runners
ExecStart=/usr/local/bin/docker-compose up -d
ExecStop=/usr/local/bin/docker-compose down
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# 重新加载 systemd
systemctl daemon-reload

# 启用服务
systemctl enable github-runner

# 启动服务
systemctl start github-runner

# 查看服务状态
systemctl status github-runner
```

## 在腾讯云服务器上的特别配置

### 1. 安全组配置

在腾讯云控制台设置安全组规则：
- 入站：允许 22 端口 (SSH)
- 出站：允许所有（Runner 需要访问 GitHub）

### 2. 使用腾讯云容器镜像服务加速

```bash
# 配置 Docker 镜像加速
mkdir -p /etc/docker
cat > /etc/docker/daemon.json << 'EOF'
{
  "registry-mirrors": [
    "https://mirror.ccs.tencentyun.com",
    "https://docker.mirrors.ustc.edu.cn"
  ],
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "100m",
    "max-file": "3"
  }
}
EOF

# 重启 Docker
systemctl restart docker
```

### 3. 监控和日志

```bash
# 实时查看 runner 日志
docker logs -f aurelia-runner-1

# 查看资源使用情况
docker stats

# 设置日志轮转
cat > /etc/logrotate.d/docker-runner << 'EOF'
/var/lib/docker/containers/*/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0644 root root
}
EOF
```

## 使用 Runner 运行工作流

### 修改 GitHub Actions 工作流

```yaml
# .github/workflows/deploy.yml
jobs:
  deploy:
    # 指定使用你的自托管 runner
    runs-on: [self-hosted, linux, x64, docker, aurelia, tencent]
    
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --release
```

### 测试 Runner

创建测试工作流 `.github/workflows/test-runner.yml`：

```yaml
name: Test Self-Hosted Runner

on:
  workflow_dispatch:

jobs:
  test:
    runs-on: [self-hosted, linux, x64, docker, aurelia]
    steps:
      - name: Check runner info
        run: |
          echo "Runner name: $RUNNER_NAME"
          echo "Runner OS: $RUNNER_OS"
          echo "Runner Arch: $RUNNER_ARCH"
          uname -a
          docker --version
          
      - uses: actions/checkout@v4
      
      - name: Test Rust
        run: |
          rustc --version
          cargo --version
```

## 常见问题解决

### 1. Runner 无法注册

```bash
# 检查 token 是否正确
cat .env

# 检查网络连接
curl -I https://github.com

# 查看详细日志
docker logs aurelia-runner-1

# 重新生成 token 并更新
nano .env
docker-compose down
docker-compose up -d
```

### 2. 权限问题

```bash
# Docker socket 权限
chmod 666 /var/run/docker.sock

# 或添加用户到 docker 组
usermod -aG docker runner
```

### 3. 磁盘空间不足

```bash
# 清理 Docker 缓存
docker system prune -a --volumes

# 查看磁盘使用
df -h
du -sh /var/lib/docker
```

### 4. Runner 离线

```bash
# 重启 runner
docker-compose restart runner-1

# 或完全重建
./deploy-runner.sh rebuild
```

## 安全建议

1. **使用专用服务器**：不要在生产服务器上运行 Runner
2. **定期更新**：保持 Docker 和 Runner 镜像最新
3. **限制网络访问**：使用防火墙规则限制入站连接
4. **监控日志**：定期检查 Runner 日志异常活动
5. **轮换 Token**：定期更换 GitHub Token

## 性能优化

```bash
# 增加 Docker 资源限制
# 编辑 docker-compose.yml
services:
  runner-1:
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 8G
        reservations:
          cpus: '2'
          memory: 4G
```

## 监控脚本

创建监控脚本 `monitor.sh`：

```bash
#!/bin/bash
# 检查 runner 状态并自动重启

RUNNER_NAME="aurelia-runner-1"

if ! docker ps | grep -q $RUNNER_NAME; then
    echo "Runner $RUNNER_NAME is down, restarting..."
    docker-compose up -d runner-1
    
    # 发送通知（可选）
    # curl -X POST https://your-webhook-url -d "Runner restarted"
fi
```

添加到 crontab：
```bash
crontab -e
# 每5分钟检查一次
*/5 * * * * /opt/github-runners/aurelia/runners/monitor.sh
```

## 完整部署命令示例

```bash
# 一键部署脚本
ssh root@106.54.1.130 << 'ENDSSH'
# 安装 Docker
curl -fsSL https://get.docker.com | bash

# 克隆项目
cd /opt
git clone https://github.com/tricorefile/aurelia.git
cd aurelia/runners

# 配置
cp .env.example .env
echo "GITHUB_TOKEN=ghp_你的token" > .env

# 启动
chmod +x *.sh
./deploy-runner.sh setup
./deploy-runner.sh start prod

# 设置开机自启
systemctl enable docker
echo "@reboot cd /opt/aurelia/runners && docker-compose up -d" | crontab -
ENDSSH
```

## 验证部署成功

1. **本地验证**：
   ```bash
   docker ps | grep aurelia-runner
   ```

2. **GitHub 验证**：
   - 访问: https://github.com/tricorefile/aurelia/settings/actions/runners
   - 看到 runner 状态为 "Idle"

3. **运行测试工作流**：
   - 在 GitHub 上手动触发测试工作流
   - 查看是否在你的 runner 上执行

部署完成！你的服务器现在是一个功能完整的 GitHub Actions Runner。