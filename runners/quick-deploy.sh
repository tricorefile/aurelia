#!/bin/bash
# 快速部署 GitHub Actions Runner 到指定服务器
set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_color() {
    echo -e "${1}${2}${NC}"
}

print_header() {
    echo ""
    print_color "$BLUE" "========================================"
    print_color "$BLUE" "$1"
    print_color "$BLUE" "========================================"
    echo ""
}

# 检查参数
if [ $# -lt 2 ]; then
    print_color "$RED" "Usage: $0 <server-ip> <github-token> [runner-name]"
    print_color "$YELLOW" "Example: $0 106.54.1.130 ghp_xxxxxxxxxxxxx tencent-runner"
    exit 1
fi

SERVER_IP=$1
GITHUB_TOKEN=$2
RUNNER_NAME=${3:-"aurelia-runner-$(date +%s)"}
SSH_USER=${SSH_USER:-root}
INSTALL_PATH="/opt/github-runners"

print_header "GitHub Actions Runner Quick Deploy"
print_color "$GREEN" "Server: $SERVER_IP"
print_color "$GREEN" "Runner Name: $RUNNER_NAME"
print_color "$GREEN" "Install Path: $INSTALL_PATH"

# 创建本地临时目录
TEMP_DIR=$(mktemp -d)
print_color "$YELLOW" "Creating temporary directory: $TEMP_DIR"

# 下载必要文件
print_header "Downloading Runner Files"
cd $TEMP_DIR

# 创建文件结构
mkdir -p runners

# 下载文件
print_color "$YELLOW" "Downloading Dockerfile..."
curl -sL https://raw.githubusercontent.com/tricorefile/aurelia/main/runners/Dockerfile -o runners/Dockerfile

print_color "$YELLOW" "Downloading docker-compose.yml..."
curl -sL https://raw.githubusercontent.com/tricorefile/aurelia/main/runners/docker-compose.yml -o runners/docker-compose.yml

print_color "$YELLOW" "Downloading entrypoint.sh..."
curl -sL https://raw.githubusercontent.com/tricorefile/aurelia/main/runners/entrypoint.sh -o runners/entrypoint.sh

print_color "$YELLOW" "Downloading deploy-runner.sh..."
curl -sL https://raw.githubusercontent.com/tricorefile/aurelia/main/runners/deploy-runner.sh -o runners/deploy-runner.sh

# 创建 .env 文件
cat > runners/.env << EOF
GITHUB_TOKEN=$GITHUB_TOKEN
GITHUB_OWNER=tricorefile
GITHUB_REPOSITORY=aurelia
EOF

# 创建 SSH 目录
mkdir -p runners/ssh

print_color "$GREEN" "✓ Files downloaded"

# 连接到服务器并部署
print_header "Deploying to Server"

ssh -o StrictHostKeyChecking=no $SSH_USER@$SERVER_IP << ENDSSH
set -e

# 安装 Docker（如果需要）
if ! command -v docker &> /dev/null; then
    echo "Installing Docker..."
    curl -fsSL https://get.docker.com | bash
    systemctl start docker
    systemctl enable docker
fi

# 安装 Docker Compose（如果需要）
if ! command -v docker-compose &> /dev/null; then
    echo "Installing Docker Compose..."
    curl -L "https://github.com/docker/compose/releases/download/v2.23.0/docker-compose-\$(uname -s)-\$(uname -m)" -o /usr/local/bin/docker-compose
    chmod +x /usr/local/bin/docker-compose
fi

# 创建安装目录
mkdir -p $INSTALL_PATH
cd $INSTALL_PATH

# 清理旧的 runner（如果存在）
if [ -d "runners" ]; then
    echo "Stopping existing runners..."
    cd runners
    docker-compose down || true
    cd ..
    rm -rf runners.backup
    mv runners runners.backup
fi

echo "Runner files will be copied to server..."
ENDSSH

# 复制文件到服务器
print_color "$YELLOW" "Copying files to server..."
scp -r -o StrictHostKeyChecking=no $TEMP_DIR/runners $SSH_USER@$SERVER_IP:$INSTALL_PATH/

# 在服务器上启动 Runner
print_header "Starting Runner on Server"

ssh -o StrictHostKeyChecking=no $SSH_USER@$SERVER_IP << ENDSSH
set -e
cd $INSTALL_PATH/runners

# 设置权限
chmod +x entrypoint.sh deploy-runner.sh

# 自定义 runner 名称
sed -i "s/RUNNER_NAME=docker-runner-prod-1/RUNNER_NAME=$RUNNER_NAME/" docker-compose.yml

# 构建镜像
echo "Building Docker image..."
docker-compose build

# 启动 runner
echo "Starting runner..."
docker-compose up -d runner-1

# 等待启动
sleep 10

# 检查状态
echo ""
echo "Checking runner status..."
docker-compose ps

# 显示日志
echo ""
echo "Recent logs:"
docker-compose logs --tail=20 runner-1

# 创建系统服务
cat > /etc/systemd/system/github-runner.service << 'EOF'
[Unit]
Description=GitHub Actions Runner
Requires=docker.service
After=docker.service

[Service]
Type=forking
RemainAfterExit=yes
WorkingDirectory=$INSTALL_PATH/runners
ExecStart=/usr/local/bin/docker-compose up -d
ExecStop=/usr/local/bin/docker-compose down
Restart=always

[Install]
WantedBy=multi-user.target
EOF

# 启用服务
systemctl daemon-reload
systemctl enable github-runner

echo ""
echo "✅ Runner deployed successfully!"
ENDSSH

# 清理临时文件
rm -rf $TEMP_DIR

print_header "Deployment Complete!"
print_color "$GREEN" "✅ Runner '$RUNNER_NAME' is now running on $SERVER_IP"
print_color "$GREEN" "✅ Systemd service 'github-runner' has been created and enabled"
echo ""
print_color "$YELLOW" "Next steps:"
print_color "$YELLOW" "1. Check runner status on GitHub:"
print_color "$BLUE" "   https://github.com/tricorefile/aurelia/settings/actions/runners"
print_color "$YELLOW" "2. SSH to server to manage runner:"
print_color "$BLUE" "   ssh $SSH_USER@$SERVER_IP"
print_color "$BLUE" "   cd $INSTALL_PATH/runners"
print_color "$BLUE" "   ./deploy-runner.sh status"
print_color "$YELLOW" "3. View runner logs:"
print_color "$BLUE" "   ssh $SSH_USER@$SERVER_IP 'cd $INSTALL_PATH/runners && docker-compose logs -f'"
echo ""
print_color "$GREEN" "🎉 Deployment successful!"