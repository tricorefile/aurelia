#!/bin/bash
# 快速构建和启动 Runner

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  GitHub Runner 快速构建${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 检查当前目录
if [ ! -f "docker-compose.cn.yml" ]; then
    echo -e "${RED}错误: 请在 runners 目录下运行此脚本${NC}"
    exit 1
fi

# 检查 .env 文件
if [ ! -f ".env" ]; then
    echo -e "${YELLOW}创建 .env 文件...${NC}"
    echo -n "请输入 GitHub Token (ghp_...): "
    read GITHUB_TOKEN
    echo "GITHUB_TOKEN=$GITHUB_TOKEN" > .env
    echo -e "${GREEN}✓ .env 文件已创建${NC}"
else
    echo -e "${GREEN}✓ 找到 .env 文件${NC}"
fi

# 使用修复后的 Dockerfile
if [ -f "Dockerfile.cn-fixed" ]; then
    echo -e "${YELLOW}使用修复版 Dockerfile...${NC}"
    cp Dockerfile.cn-fixed Dockerfile.cn
elif [ ! -f "Dockerfile.cn" ]; then
    echo -e "${YELLOW}下载 Dockerfile.cn...${NC}"
    curl -L -o Dockerfile.cn https://raw.githubusercontent.com/tricorefile/aurelia/main/runners/Dockerfile.cn-fixed
fi

# 停止旧容器
echo ""
echo -e "${YELLOW}停止旧容器...${NC}"
docker compose -f docker-compose.cn.yml down 2>/dev/null || true
docker stop $(docker ps -q --filter "name=aurelia-runner") 2>/dev/null || true
echo -e "${GREEN}✓ 清理完成${NC}"

# 构建镜像
echo ""
echo -e "${YELLOW}开始构建镜像...${NC}"
echo -e "${YELLOW}这可能需要 5-10 分钟，请耐心等待...${NC}"

# 使用 BuildKit 加速
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

# 构建
if docker compose -f docker-compose.cn.yml build --no-cache; then
    echo -e "${GREEN}✓ 镜像构建成功${NC}"
else
    echo -e "${RED}✗ 构建失败${NC}"
    echo ""
    echo -e "${YELLOW}尝试备用方案...${NC}"
    
    # 尝试使用预构建镜像
    echo "使用预构建镜像..."
    cat > docker-compose.prebuilt.yml << 'EOF'
version: '3.8'

services:
  runner:
    image: myoung34/github-runner:latest
    container_name: aurelia-runner-prebuilt
    environment:
      - REPO_URL=https://github.com/tricorefile/aurelia
      - RUNNER_NAME=prebuilt-runner
      - ACCESS_TOKEN=${GITHUB_TOKEN}
      - RUNNER_WORKDIR=/tmp/runner/work
      - LABELS=self-hosted,linux,x64,docker
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./work:/tmp/runner/work
    restart: unless-stopped
EOF
    
    docker compose -f docker-compose.prebuilt.yml pull
    docker compose -f docker-compose.prebuilt.yml up -d
    
    echo -e "${GREEN}✓ 使用预构建镜像启动${NC}"
    exit 0
fi

# 启动容器
echo ""
echo -e "${YELLOW}启动 Runner...${NC}"

# 只启动一个 runner 以测试
docker compose -f docker-compose.cn.yml up -d runner-1

# 等待启动
sleep 5

# 检查状态
echo ""
echo -e "${YELLOW}检查状态...${NC}"
docker compose -f docker-compose.cn.yml ps

# 显示日志
echo ""
echo -e "${YELLOW}最新日志:${NC}"
docker compose -f docker-compose.cn.yml logs --tail=30 runner-1

# 检查是否运行正常
if docker ps | grep -q "aurelia-runner-1"; then
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  ✓ Runner 启动成功！${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo "后续操作:"
    echo "1. 查看实时日志: docker compose -f docker-compose.cn.yml logs -f runner-1"
    echo "2. 启动更多 runners: docker compose -f docker-compose.cn.yml up -d"
    echo "3. 验证注册: https://github.com/tricorefile/aurelia/settings/actions/runners"
else
    echo ""
    echo -e "${RED}========================================${NC}"
    echo -e "${RED}  ✗ Runner 启动可能有问题${NC}"
    echo -e "${RED}========================================${NC}"
    echo ""
    echo "调试步骤:"
    echo "1. 查看详细日志: docker compose -f docker-compose.cn.yml logs runner-1"
    echo "2. 检查 Token: cat .env"
    echo "3. 手动调试: docker compose -f docker-compose.cn.yml run runner-1 bash"
fi