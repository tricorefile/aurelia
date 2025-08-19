#!/bin/bash
# 中国优化版 Runner 构建脚本
# 解决网络连接问题和镜像源问题

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  GitHub Runner 中国优化版构建${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 检查 Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}错误: Docker 未安装${NC}"
    exit 1
fi

# 检查 docker-compose
if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}错误: Docker Compose 未安装${NC}"
    exit 1
fi

# 选择构建方式
echo -e "${YELLOW}选择构建方式:${NC}"
echo "1) 使用中国镜像源（推荐）"
echo "2) 使用代理"
echo "3) 标准构建（可能失败）"
read -p "请选择 (1-3): " choice

case $choice in
    1)
        echo -e "${GREEN}使用中国镜像源构建...${NC}"
        DOCKERFILE="Dockerfile.cn"
        COMPOSE_FILE="docker-compose.cn.yml"
        ;;
    2)
        echo -e "${YELLOW}使用代理构建...${NC}"
        read -p "请输入 HTTP 代理地址 (例如 http://127.0.0.1:7890): " HTTP_PROXY
        read -p "请输入 HTTPS 代理地址 (留空使用 HTTP 代理): " HTTPS_PROXY
        HTTPS_PROXY=${HTTPS_PROXY:-$HTTP_PROXY}
        
        export HTTP_PROXY
        export HTTPS_PROXY
        export NO_PROXY="localhost,127.0.0.1,mirrors.aliyun.com"
        
        DOCKERFILE="Dockerfile"
        COMPOSE_FILE="docker-compose.yml"
        
        # 为 Docker 设置代理
        mkdir -p ~/.docker
        cat > ~/.docker/config.json << EOF
{
  "proxies": {
    "default": {
      "httpProxy": "$HTTP_PROXY",
      "httpsProxy": "$HTTPS_PROXY",
      "noProxy": "$NO_PROXY"
    }
  }
}
EOF
        echo -e "${GREEN}代理配置完成${NC}"
        ;;
    3)
        echo -e "${YELLOW}使用标准构建...${NC}"
        DOCKERFILE="Dockerfile"
        COMPOSE_FILE="docker-compose.yml"
        ;;
    *)
        echo -e "${RED}无效选择${NC}"
        exit 1
        ;;
esac

# 检查文件
if [ ! -f "$DOCKERFILE" ]; then
    echo -e "${RED}错误: $DOCKERFILE 不存在${NC}"
    
    if [ "$DOCKERFILE" == "Dockerfile.cn" ]; then
        echo -e "${YELLOW}正在下载中国优化版 Dockerfile...${NC}"
        curl -L -o Dockerfile.cn https://raw.githubusercontent.com/tricorefile/aurelia/main/runners/Dockerfile.cn
        
        if [ $? -ne 0 ]; then
            echo -e "${RED}下载失败，创建本地版本...${NC}"
            # 这里会创建一个简化版的 Dockerfile.cn
            cat > Dockerfile.cn << 'EOF'
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive
ENV RUNNER_ALLOW_RUNASROOT=1

# 使用阿里云镜像
RUN sed -i 's/archive.ubuntu.com/mirrors.aliyun.com/g' /etc/apt/sources.list && \
    sed -i 's/security.ubuntu.com/mirrors.aliyun.com/g' /etc/apt/sources.list

# 安装基础包
RUN apt-get update && apt-get install -y \
    curl wget git jq build-essential libssl-dev \
    python3 python3-pip openssh-client \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /home/runner

# 下载 runner
ARG RUNNER_VERSION=2.311.0
RUN curl -L -o runner.tar.gz \
    https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz && \
    tar xzf runner.tar.gz && rm runner.tar.gz && \
    ./bin/installdependencies.sh

COPY entrypoint.sh /home/runner/
RUN chmod +x entrypoint.sh

ENTRYPOINT ["/home/runner/entrypoint.sh"]
EOF
        fi
    fi
fi

if [ ! -f "$COMPOSE_FILE" ] && [ "$COMPOSE_FILE" == "docker-compose.cn.yml" ]; then
    echo -e "${YELLOW}创建中国优化版 docker-compose 文件...${NC}"
    cat > docker-compose.cn.yml << 'EOF'
version: '3.8'

services:
  runner:
    build:
      context: .
      dockerfile: Dockerfile.cn
      network: host
    container_name: aurelia-runner
    environment:
      - GITHUB_TOKEN=${GITHUB_TOKEN}
      - GITHUB_OWNER=tricorefile
      - GITHUB_REPOSITORY=aurelia
      - RUNNER_NAME=docker-runner-cn
      - RUNNER_LABELS=self-hosted,linux,x64,docker,aurelia,china
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./work:/home/runner/work
    restart: unless-stopped
EOF
fi

# 检查 .env 文件
if [ ! -f .env ]; then
    echo -e "${RED}错误: .env 文件不存在${NC}"
    echo -e "${YELLOW}请创建 .env 文件并添加:${NC}"
    echo "GITHUB_TOKEN=你的token"
    exit 1
fi

# 清理旧容器
echo ""
echo -e "${YELLOW}清理旧容器...${NC}"
docker-compose -f $COMPOSE_FILE down 2>/dev/null || true

# 构建镜像
echo ""
echo -e "${YELLOW}开始构建 Docker 镜像...${NC}"
echo -e "${YELLOW}这可能需要几分钟，请耐心等待...${NC}"

# 使用 buildkit 加速构建
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

# 构建
if docker-compose -f $COMPOSE_FILE build --progress=plain; then
    echo -e "${GREEN}✓ 镜像构建成功${NC}"
else
    echo -e "${RED}✗ 镜像构建失败${NC}"
    echo ""
    echo -e "${YELLOW}可能的解决方案:${NC}"
    echo "1. 检查网络连接"
    echo "2. 使用代理: export HTTP_PROXY=http://your-proxy:port"
    echo "3. 手动修改 Dockerfile 使用其他镜像源"
    echo "4. 尝试使用 VPN"
    exit 1
fi

# 启动容器
echo ""
echo -e "${YELLOW}启动 Runner 容器...${NC}"
if docker-compose -f $COMPOSE_FILE up -d; then
    echo -e "${GREEN}✓ Runner 启动成功${NC}"
else
    echo -e "${RED}✗ Runner 启动失败${NC}"
    exit 1
fi

# 等待启动
sleep 5

# 检查状态
echo ""
echo -e "${YELLOW}检查 Runner 状态...${NC}"
docker-compose -f $COMPOSE_FILE ps

# 显示日志
echo ""
echo -e "${YELLOW}最新日志:${NC}"
docker-compose -f $COMPOSE_FILE logs --tail=30

# 清理代理配置
if [ -n "$HTTP_PROXY" ]; then
    echo ""
    echo -e "${YELLOW}清理代理配置...${NC}"
    rm -f ~/.docker/config.json
fi

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}  构建完成！${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "查看日志: docker-compose -f $COMPOSE_FILE logs -f"
echo "停止服务: docker-compose -f $COMPOSE_FILE down"
echo "重启服务: docker-compose -f $COMPOSE_FILE restart"
echo ""
echo "验证 Runner: https://github.com/tricorefile/aurelia/settings/actions/runners"