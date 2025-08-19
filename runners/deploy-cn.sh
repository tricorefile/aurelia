#!/bin/bash
# 针对中国服务器优化的 GitHub Actions Runner 部署脚本
# 包含镜像加速、代理配置等优化
set -e

# 配置
SERVER_IP=${1:-"106.54.1.130"}
GITHUB_TOKEN=${2:-""}
SSH_USER=${SSH_USER:-"root"}
SSH_KEY=${SSH_KEY:-"~/.ssh/id_rsa"}
RUNNER_NAME=${3:-"tencent-cloud-runner"}

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=================================${NC}"
echo -e "${GREEN}  Aurelia Runner 中国区部署脚本  ${NC}"
echo -e "${GREEN}=================================${NC}"

# 检查 Token
if [ -z "$GITHUB_TOKEN" ]; then
    echo -e "${RED}错误: 请提供 GitHub Token${NC}"
    echo "用法: $0 <服务器IP> <GitHub-Token> [Runner名称]"
    echo "示例: $0 106.54.1.130 ghp_xxxxxxxxxxxxx tencent-runner"
    exit 1
fi

echo -e "${YELLOW}目标服务器: $SERVER_IP${NC}"
echo -e "${YELLOW}Runner 名称: $RUNNER_NAME${NC}"

# SSH 命令封装
run_ssh() {
    ssh -o StrictHostKeyChecking=no -i $SSH_KEY $SSH_USER@$SERVER_IP "$@"
}

# SCP 命令封装
run_scp() {
    scp -o StrictHostKeyChecking=no -i $SSH_KEY "$@"
}

echo -e "${GREEN}[1/6] 准备服务器环境...${NC}"

run_ssh << 'ENDSSH'
# 更新系统
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq

# 安装基础工具
apt-get install -y -qq curl wget git jq unzip > /dev/null 2>&1

# 配置时区
timedatectl set-timezone Asia/Shanghai

echo "✓ 基础环境准备完成"
ENDSSH

echo -e "${GREEN}[2/6] 安装 Docker（使用国内镜像）...${NC}"

run_ssh << 'ENDSSH'
if ! command -v docker &> /dev/null; then
    # 使用阿里云镜像安装 Docker
    curl -fsSL https://get.docker.com | bash -s docker --mirror Aliyun
    
    # 配置 Docker 镜像加速
    mkdir -p /etc/docker
    cat > /etc/docker/daemon.json << 'EOF'
{
    "registry-mirrors": [
        "https://mirror.ccs.tencentyun.com",
        "https://docker.mirrors.ustc.edu.cn",
        "https://hub-mirror.c.163.com",
        "https://registry.docker-cn.com"
    ],
    "log-driver": "json-file",
    "log-opts": {
        "max-size": "100m",
        "max-file": "3"
    },
    "storage-driver": "overlay2",
    "exec-opts": ["native.cgroupdriver=systemd"]
}
EOF
    
    systemctl daemon-reload
    systemctl restart docker
    systemctl enable docker
    
    echo "✓ Docker 安装完成（使用镜像加速）"
else
    echo "✓ Docker 已安装"
fi

# 安装 Docker Compose
if ! command -v docker-compose &> /dev/null; then
    # 使用国内镜像下载
    curl -L "https://get.daocloud.io/docker/compose/releases/download/v2.23.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    chmod +x /usr/local/bin/docker-compose
    echo "✓ Docker Compose 安装完成"
else
    echo "✓ Docker Compose 已安装"
fi
ENDSSH

echo -e "${GREEN}[3/6] 创建 Runner 配置文件...${NC}"

# 创建临时目录
TEMP_DIR=$(mktemp -d)
cd $TEMP_DIR

# 创建优化后的 Dockerfile（使用国内镜像源）
cat > Dockerfile << 'EOF'
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV RUNNER_ALLOW_RUNASROOT=1

# 使用国内镜像源
RUN sed -i 's/archive.ubuntu.com/mirrors.aliyun.com/g' /etc/apt/sources.list && \
    sed -i 's/security.ubuntu.com/mirrors.aliyun.com/g' /etc/apt/sources.list

# 安装基础包
RUN apt-get update && apt-get install -y \
    curl wget git jq build-essential libssl-dev \
    python3 python3-pip openssh-client \
    && rm -rf /var/lib/apt/lists/*

# 安装 Rust（使用国内镜像）
ENV RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
ENV RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
ENV RUSTUP_HOME=/opt/rust
ENV CARGO_HOME=/opt/rust
ENV PATH=/opt/rust/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable && \
    mkdir -p $CARGO_HOME && \
    echo '[source.crates-io]' > $CARGO_HOME/config && \
    echo 'replace-with = "ustc"' >> $CARGO_HOME/config && \
    echo '[source.ustc]' >> $CARGO_HOME/config && \
    echo 'registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"' >> $CARGO_HOME/config

WORKDIR /home/runner

# 下载 GitHub Runner
ARG RUNNER_VERSION=2.311.0
RUN wget -q https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz && \
    tar xzf ./actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz && \
    rm actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz && \
    ./bin/installdependencies.sh

COPY entrypoint.sh /home/runner/entrypoint.sh
RUN chmod +x /home/runner/entrypoint.sh

ENTRYPOINT ["/home/runner/entrypoint.sh"]
EOF

# 创建 entrypoint.sh
cat > entrypoint.sh << 'EOF'
#!/bin/bash
set -e

GITHUB_OWNER=${GITHUB_OWNER:-tricorefile}
GITHUB_REPOSITORY=${GITHUB_REPOSITORY:-aurelia}
RUNNER_NAME=${RUNNER_NAME:-docker-runner}
RUNNER_LABELS=${RUNNER_LABELS:-self-hosted,linux,x64,docker,aurelia,china}

if [[ -z "$GITHUB_TOKEN" ]]; then
    echo "Error: GITHUB_TOKEN not set"
    exit 1
fi

# 获取注册 token
if [[ ${#GITHUB_TOKEN} -gt 50 ]]; then
    REG_TOKEN="$GITHUB_TOKEN"
else
    REG_TOKEN=$(curl -sX POST \
        -H "Authorization: token ${GITHUB_TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPOSITORY}/actions/runners/registration-token" \
        | jq -r .token)
fi

# 配置 runner
./config.sh \
    --url "https://github.com/${GITHUB_OWNER}/${GITHUB_REPOSITORY}" \
    --token "${REG_TOKEN}" \
    --name "${RUNNER_NAME}" \
    --labels "${RUNNER_LABELS}" \
    --work "_work" \
    --unattended \
    --replace

# 启动 runner
./run.sh
EOF

# 创建 docker-compose.yml
cat > docker-compose.yml << EOF
version: '3.8'

services:
  runner:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: ${RUNNER_NAME}
    environment:
      - GITHUB_TOKEN=${GITHUB_TOKEN}
      - GITHUB_OWNER=tricorefile
      - GITHUB_REPOSITORY=aurelia
      - RUNNER_NAME=${RUNNER_NAME}
      - RUNNER_LABELS=self-hosted,linux,x64,docker,aurelia,china,tencent
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - runner-work:/home/runner/_work
    restart: unless-stopped
    networks:
      - runner-net

volumes:
  runner-work:
    driver: local

networks:
  runner-net:
    driver: bridge
EOF

# 创建 .env 文件
cat > .env << EOF
GITHUB_TOKEN=${GITHUB_TOKEN}
EOF

echo -e "${GREEN}[4/6] 上传文件到服务器...${NC}"

# 上传文件
run_ssh "mkdir -p /opt/aurelia-runner"
run_scp -r $TEMP_DIR/* $SSH_USER@$SERVER_IP:/opt/aurelia-runner/

echo -e "${GREEN}[5/6] 构建并启动 Runner...${NC}"

run_ssh << 'ENDSSH'
cd /opt/aurelia-runner

# 构建镜像
echo "构建 Docker 镜像..."
docker-compose build --progress=plain

# 启动 runner
echo "启动 Runner..."
docker-compose up -d

# 等待启动
sleep 10

# 检查状态
echo ""
echo "Runner 状态:"
docker-compose ps
echo ""
echo "最近日志:"
docker-compose logs --tail=30
ENDSSH

echo -e "${GREEN}[6/6] 配置开机自启...${NC}"

run_ssh << 'ENDSSH'
# 创建 systemd 服务
cat > /etc/systemd/system/aurelia-runner.service << 'EOF'
[Unit]
Description=Aurelia GitHub Actions Runner
Requires=docker.service
After=docker.service network-online.target
Wants=network-online.target

[Service]
Type=forking
RemainAfterExit=yes
WorkingDirectory=/opt/aurelia-runner
ExecStart=/usr/local/bin/docker-compose up -d
ExecStop=/usr/local/bin/docker-compose down
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable aurelia-runner

echo "✓ 开机自启配置完成"
ENDSSH

# 清理临时文件
rm -rf $TEMP_DIR

echo ""
echo -e "${GREEN}=================================${NC}"
echo -e "${GREEN}     🎉 部署成功完成！ 🎉        ${NC}"
echo -e "${GREEN}=================================${NC}"
echo ""
echo -e "${YELLOW}Runner 信息:${NC}"
echo "  • 名称: $RUNNER_NAME"
echo "  • 位置: $SERVER_IP:/opt/aurelia-runner"
echo "  • 标签: self-hosted,linux,x64,docker,aurelia,china,tencent"
echo ""
echo -e "${YELLOW}管理命令:${NC}"
echo "  • 查看状态: ssh $SSH_USER@$SERVER_IP 'cd /opt/aurelia-runner && docker-compose ps'"
echo "  • 查看日志: ssh $SSH_USER@$SERVER_IP 'cd /opt/aurelia-runner && docker-compose logs -f'"
echo "  • 重启服务: ssh $SSH_USER@$SERVER_IP 'systemctl restart aurelia-runner'"
echo ""
echo -e "${YELLOW}验证步骤:${NC}"
echo "  1. 访问 GitHub Actions Runners 页面:"
echo "     https://github.com/tricorefile/aurelia/settings/actions/runners"
echo "  2. 确认 '$RUNNER_NAME' 显示为 Idle 状态"
echo ""
echo -e "${GREEN}提示: Runner 已配置为系统服务，会自动启动和故障恢复${NC}"