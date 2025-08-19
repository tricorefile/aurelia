#!/bin/bash
# 终极修复方案 - 解决所有 Docker 和网络问题

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  GitHub Runner 终极修复方案${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 检测问题类型
echo -e "${YELLOW}正在诊断问题...${NC}"
echo ""

# 测试 Docker
DOCKER_OK=false
if command -v docker &> /dev/null && docker info &> /dev/null; then
    DOCKER_OK=true
    echo -e "${GREEN}✓ Docker 运行正常${NC}"
else
    echo -e "${RED}✗ Docker 有问题${NC}"
fi

# 测试网络
NETWORK_OK=false
if ping -c 1 github.com &> /dev/null; then
    NETWORK_OK=true
    echo -e "${GREEN}✓ 可以访问 GitHub${NC}"
else
    echo -e "${YELLOW}⚠ 无法直接访问 GitHub${NC}"
fi

# 测试 Docker Hub
DOCKERHUB_OK=false
if curl -s -o /dev/null -w "%{http_code}" https://hub.docker.com &> /dev/null; then
    DOCKERHUB_OK=true
    echo -e "${GREEN}✓ 可以访问 Docker Hub${NC}"
else
    echo -e "${YELLOW}⚠ 无法访问 Docker Hub${NC}"
fi

echo ""
echo -e "${YELLOW}选择修复方案:${NC}"
echo "1) 自动选择最佳方案（推荐）"
echo "2) 修复 Docker 镜像问题"
echo "3) 使用离线安装"
echo "4) 使用预构建镜像"
echo "5) 手动配置代理"
echo "6) 完全重装"

read -p "请选择 (1-6): " choice

case ${choice:-1} in
    1)
        echo ""
        echo -e "${GREEN}自动选择最佳方案...${NC}"
        
        if [ "$DOCKER_OK" = false ]; then
            echo "检测到 Docker 问题，修复中..."
            sudo systemctl restart docker || sudo service docker restart || {
                echo "重装 Docker..."
                curl -fsSL https://get.docker.com | sh
            }
        fi
        
        if [ "$DOCKERHUB_OK" = false ]; then
            echo "配置 Docker 镜像加速..."
            ./fix-docker-mirror.sh || {
                echo "使用离线方案..."
                ./offline-setup.sh
            }
        else
            echo "使用标准 Docker 构建..."
            docker-compose build --no-cache
            docker-compose up -d
        fi
        ;;
        
    2)
        echo -e "${GREEN}修复 Docker 镜像问题...${NC}"
        
        # 清理 Docker
        docker system prune -af --volumes || true
        
        # 修复镜像源
        if [ -f fix-docker-mirror.sh ]; then
            ./fix-docker-mirror.sh
        else
            curl -O https://raw.githubusercontent.com/tricorefile/aurelia/main/runners/fix-docker-mirror.sh
            chmod +x fix-docker-mirror.sh
            ./fix-docker-mirror.sh
        fi
        
        # 使用中国优化版
        if [ -f docker-compose.cn.yml ]; then
            docker-compose -f docker-compose.cn.yml build --no-cache
            docker-compose -f docker-compose.cn.yml up -d
        fi
        ;;
        
    3)
        echo -e "${GREEN}使用离线安装...${NC}"
        
        if [ -f offline-setup.sh ]; then
            ./offline-setup.sh
        else
            curl -O https://raw.githubusercontent.com/tricorefile/aurelia/main/runners/offline-setup.sh
            chmod +x offline-setup.sh
            ./offline-setup.sh
        fi
        ;;
        
    4)
        echo -e "${GREEN}使用预构建镜像...${NC}"
        
        # 创建简化的 docker-compose
        cat > docker-compose.simple.yml << 'EOF'
version: '3'
services:
  runner:
    image: myoung34/github-runner:latest
    container_name: github-runner
    environment:
      REPO_URL: https://github.com/tricorefile/aurelia
      RUNNER_NAME: docker-runner
      RUNNER_TOKEN: ${GITHUB_TOKEN}
      RUNNER_WORKDIR: /tmp/runner/work
      LABELS: self-hosted,linux,x64,docker
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./work:/tmp/runner/work
    restart: unless-stopped
EOF
        
        echo "请确保设置了 GITHUB_TOKEN"
        docker-compose -f docker-compose.simple.yml pull
        docker-compose -f docker-compose.simple.yml up -d
        ;;
        
    5)
        echo -e "${GREEN}配置代理...${NC}"
        
        read -p "输入 HTTP 代理地址 (例: http://127.0.0.1:7890): " HTTP_PROXY
        read -p "输入 HTTPS 代理地址 (回车使用 HTTP 代理): " HTTPS_PROXY
        HTTPS_PROXY=${HTTPS_PROXY:-$HTTP_PROXY}
        
        # 设置系统代理
        export HTTP_PROXY
        export HTTPS_PROXY
        export NO_PROXY="localhost,127.0.0.1"
        
        # 配置 Docker 代理
        sudo mkdir -p /etc/systemd/system/docker.service.d
        sudo tee /etc/systemd/system/docker.service.d/http-proxy.conf << EOF
[Service]
Environment="HTTP_PROXY=$HTTP_PROXY"
Environment="HTTPS_PROXY=$HTTPS_PROXY"
Environment="NO_PROXY=localhost,127.0.0.1"
EOF
        
        sudo systemctl daemon-reload
        sudo systemctl restart docker
        
        echo -e "${GREEN}代理配置完成${NC}"
        
        # 重新构建
        docker-compose build --build-arg HTTP_PROXY=$HTTP_PROXY --build-arg HTTPS_PROXY=$HTTPS_PROXY
        docker-compose up -d
        ;;
        
    6)
        echo -e "${GREEN}完全重装...${NC}"
        
        # 停止所有容器
        docker stop $(docker ps -aq) 2>/dev/null || true
        docker rm $(docker ps -aq) 2>/dev/null || true
        
        # 清理 Docker
        docker system prune -af --volumes
        
        # 重装 Docker
        sudo apt-get remove docker docker-engine docker.io containerd runc || true
        curl -fsSL https://get.docker.com | sh
        
        # 重新开始
        echo "Docker 已重装，请重新运行部署脚本"
        ;;
        
    *)
        echo -e "${RED}无效选择${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}  修复流程完成${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "验证步骤:"
echo "1. 检查容器: docker ps"
echo "2. 查看日志: docker logs github-runner"
echo "3. GitHub: https://github.com/tricorefile/aurelia/settings/actions/runners"
echo ""
echo "如果仍有问题:"
echo "- 检查防火墙设置"
echo "- 确认 GitHub Token 有效"
echo "- 尝试使用 VPN"
echo "- 联系网络管理员"