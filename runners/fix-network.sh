#!/bin/bash
# 快速修复 Docker 构建网络问题

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  修复 Docker 构建网络问题${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 1. 停止当前容器
echo -e "${YELLOW}[1/5] 停止现有容器...${NC}"
docker-compose down || true
docker stop $(docker ps -q --filter "name=runner") 2>/dev/null || true
echo -e "${GREEN}✓ 完成${NC}"

# 2. 配置 Docker 使用镜像加速
echo ""
echo -e "${YELLOW}[2/5] 配置 Docker 镜像加速...${NC}"

sudo tee /etc/docker/daemon.json > /dev/null << 'EOF'
{
  "registry-mirrors": [
    "https://mirror.ccs.tencentyun.com",
    "https://docker.mirrors.ustc.edu.cn",
    "https://hub-mirror.c.163.com",
    "https://registry.docker-cn.com"
  ],
  "dns": ["8.8.8.8", "114.114.114.114"],
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "100m",
    "max-file": "3"
  }
}
EOF

# 重启 Docker
sudo systemctl restart docker
echo -e "${GREEN}✓ Docker 镜像加速配置完成${NC}"

# 3. 创建优化版 Dockerfile
echo ""
echo -e "${YELLOW}[3/5] 创建网络优化版 Dockerfile...${NC}"

cat > Dockerfile.fixed << 'EOF'
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV RUNNER_ALLOW_RUNASROOT=1

# 强制使用镜像源
RUN echo "deb http://mirrors.aliyun.com/ubuntu/ jammy main restricted universe multiverse" > /etc/apt/sources.list && \
    echo "deb http://mirrors.aliyun.com/ubuntu/ jammy-security main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb http://mirrors.aliyun.com/ubuntu/ jammy-updates main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb http://mirrors.aliyun.com/ubuntu/ jammy-backports main restricted universe multiverse" >> /etc/apt/sources.list

# 更新并安装（添加重试）
RUN for i in 1 2 3; do \
        apt-get update && \
        apt-get install -y --no-install-recommends \
            curl wget git jq build-essential \
            libssl-dev python3 python3-pip \
            openssh-client ca-certificates && \
        break || \
        (echo "Retry $i failed, waiting..." && sleep 10); \
    done && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /home/runner

# 下载 Runner（添加重试和超时）
ARG RUNNER_VERSION=2.311.0
RUN for i in 1 2 3 4 5; do \
        echo "Downloading runner (attempt $i)..." && \
        curl --connect-timeout 30 --max-time 300 -L -o runner.tar.gz \
            "https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz" && \
        tar xzf runner.tar.gz && \
        rm runner.tar.gz && \
        break || \
        (echo "Download failed, using proxy mirror..." && \
         wget -T 300 -O runner.tar.gz \
            "https://ghproxy.com/https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz" && \
         tar xzf runner.tar.gz && \
         rm runner.tar.gz && \
         break) || \
        (echo "Attempt $i failed" && sleep 30); \
    done

# 安装依赖
RUN ./bin/installdependencies.sh || true

# 复制入口脚本
COPY entrypoint.sh /home/runner/
RUN chmod +x /home/runner/entrypoint.sh

ENTRYPOINT ["/home/runner/entrypoint.sh"]
EOF

echo -e "${GREEN}✓ Dockerfile 创建完成${NC}"

# 4. 创建简化的 docker-compose
echo ""
echo -e "${YELLOW}[4/5] 创建简化的 docker-compose.yml...${NC}"

cat > docker-compose.fixed.yml << 'EOF'
version: '3.8'

services:
  runner:
    build:
      context: .
      dockerfile: Dockerfile.fixed
      args:
        - RUNNER_VERSION=2.311.0
      # 使用主机网络避免 DNS 问题
      network: host
    container_name: aurelia-runner-fixed
    network_mode: host
    environment:
      - GITHUB_TOKEN=${GITHUB_TOKEN}
      - GITHUB_OWNER=tricorefile
      - GITHUB_REPOSITORY=aurelia
      - RUNNER_NAME=docker-runner-fixed
      - RUNNER_LABELS=self-hosted,linux,x64,docker,aurelia,china
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./work:/home/runner/work
    restart: unless-stopped
EOF

echo -e "${GREEN}✓ docker-compose.yml 创建完成${NC}"

# 5. 构建并启动
echo ""
echo -e "${YELLOW}[5/5] 构建并启动 Runner...${NC}"

# 检查 .env
if [ ! -f .env ]; then
    echo -e "${RED}警告: .env 文件不存在${NC}"
    read -p "请输入 GitHub Token: " GITHUB_TOKEN
    echo "GITHUB_TOKEN=$GITHUB_TOKEN" > .env
fi

# 清理旧镜像
docker rmi $(docker images -q --filter "dangling=true") 2>/dev/null || true

# 构建（使用 BuildKit）
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

echo "开始构建（可能需要 5-10 分钟）..."
if timeout 600 docker-compose -f docker-compose.fixed.yml build --no-cache; then
    echo -e "${GREEN}✓ 构建成功${NC}"
else
    echo -e "${RED}✗ 构建失败或超时${NC}"
    echo ""
    echo -e "${YELLOW}备选方案:${NC}"
    echo "1. 使用预构建镜像:"
    echo "   docker pull ghcr.io/actions/actions-runner:latest"
    echo "2. 使用代理:"
    echo "   export HTTP_PROXY=http://your-proxy:port"
    echo "3. 手动下载 runner 并构建"
    exit 1
fi

# 启动
docker-compose -f docker-compose.fixed.yml up -d

# 检查
sleep 5
docker-compose -f docker-compose.fixed.yml ps
docker-compose -f docker-compose.fixed.yml logs --tail=20

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}  修复完成！${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "查看日志: docker-compose -f docker-compose.fixed.yml logs -f"
echo "验证: https://github.com/tricorefile/aurelia/settings/actions/runners"

# 可选：设置系统代理
echo ""
echo -e "${YELLOW}提示: 如果仍有网络问题，可以设置系统代理:${NC}"
echo "export HTTP_PROXY=http://127.0.0.1:7890"
echo "export HTTPS_PROXY=http://127.0.0.1:7890"
echo "export NO_PROXY=localhost,127.0.0.1,mirrors.aliyun.com"