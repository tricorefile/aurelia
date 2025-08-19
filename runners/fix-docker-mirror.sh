#!/bin/bash
# 修复 Docker 镜像拉取问题

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  修复 Docker 镜像拉取问题${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 1. 测试当前网络
echo -e "${YELLOW}[1/6] 测试网络连接...${NC}"

test_mirror() {
    local mirror=$1
    local name=$2
    echo -n "  测试 $name... "
    if timeout 5 curl -s -o /dev/null -w "%{http_code}" "$mirror" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ 可用${NC}"
        return 0
    else
        echo -e "${RED}✗ 不可用${NC}"
        return 1
    fi
}

# 测试各个镜像源
WORKING_MIRRORS=()

test_mirror "https://docker.m.daocloud.io" "DaoCloud" && WORKING_MIRRORS+=("https://docker.m.daocloud.io")
test_mirror "https://hub-mirror.c.163.com" "网易" && WORKING_MIRRORS+=("https://hub-mirror.c.163.com")
test_mirror "https://registry.docker-cn.com" "Docker中国" && WORKING_MIRRORS+=("https://registry.docker-cn.com")
test_mirror "https://mirror.ccs.tencentyun.com" "腾讯云" && WORKING_MIRRORS+=("https://mirror.ccs.tencentyun.com")
test_mirror "https://docker.mirrors.ustc.edu.cn" "中科大" && WORKING_MIRRORS+=("https://docker.mirrors.ustc.edu.cn")

echo ""
echo -e "${GREEN}可用镜像源: ${#WORKING_MIRRORS[@]} 个${NC}"

# 2. 停止 Docker
echo ""
echo -e "${YELLOW}[2/6] 停止 Docker 服务...${NC}"
sudo systemctl stop docker || true
sudo systemctl stop docker.socket || true
echo -e "${GREEN}✓ Docker 已停止${NC}"

# 3. 清理 Docker 配置
echo ""
echo -e "${YELLOW}[3/6] 清理 Docker 配置...${NC}"

# 备份原配置
if [ -f /etc/docker/daemon.json ]; then
    sudo cp /etc/docker/daemon.json /etc/docker/daemon.json.backup.$(date +%Y%m%d_%H%M%S)
    echo "  原配置已备份"
fi

# 4. 配置新的镜像源
echo ""
echo -e "${YELLOW}[4/6] 配置 Docker 镜像源...${NC}"

if [ ${#WORKING_MIRRORS[@]} -gt 0 ]; then
    # 使用可用的镜像源
    MIRRORS_JSON=$(printf '"%s",' "${WORKING_MIRRORS[@]}" | sed 's/,$//')
    
    sudo tee /etc/docker/daemon.json > /dev/null << EOF
{
  "registry-mirrors": [$MIRRORS_JSON],
  "insecure-registries": ["docker.m.daocloud.io"],
  "max-concurrent-downloads": 10,
  "max-concurrent-uploads": 5,
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "100m",
    "max-file": "3"
  },
  "storage-driver": "overlay2",
  "debug": false
}
EOF
    echo -e "${GREEN}✓ 镜像源配置完成${NC}"
else
    echo -e "${YELLOW}⚠ 没有可用的镜像源，使用直连${NC}"
    
    # 不使用镜像源，直接连接
    sudo tee /etc/docker/daemon.json > /dev/null << EOF
{
  "max-concurrent-downloads": 10,
  "max-concurrent-uploads": 5,
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "100m",
    "max-file": "3"
  },
  "storage-driver": "overlay2"
}
EOF
fi

# 5. 重启 Docker
echo ""
echo -e "${YELLOW}[5/6] 重启 Docker...${NC}"

sudo systemctl daemon-reload
sudo systemctl start docker

# 等待 Docker 启动
for i in {1..10}; do
    if docker info > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Docker 启动成功${NC}"
        break
    fi
    echo -n "."
    sleep 1
done

# 验证配置
echo ""
echo "当前 Docker 配置:"
docker info | grep -A 5 "Registry Mirrors" || echo "未配置镜像加速"

# 6. 测试拉取镜像
echo ""
echo -e "${YELLOW}[6/6] 测试镜像拉取...${NC}"

# 先清理可能损坏的镜像
docker image prune -f > /dev/null 2>&1 || true

# 尝试拉取测试镜像
echo -n "测试拉取 ubuntu:22.04... "
if timeout 60 docker pull ubuntu:22.04 > /dev/null 2>&1; then
    echo -e "${GREEN}✓ 成功${NC}"
else
    echo -e "${RED}✗ 失败${NC}"
    
    echo ""
    echo -e "${YELLOW}尝试备选方案...${NC}"
    
    # 方案1: 使用代理
    echo ""
    echo -e "${YELLOW}方案1: 使用 HTTP 代理${NC}"
    echo "如果你有代理，可以配置："
    echo "  export HTTP_PROXY=http://你的代理:端口"
    echo "  export HTTPS_PROXY=http://你的代理:端口"
    echo "  docker pull ubuntu:22.04"
    
    # 方案2: 手动下载
    echo ""
    echo -e "${YELLOW}方案2: 手动导入镜像${NC}"
    echo "在可以访问 Docker Hub 的机器上："
    echo "  docker pull ubuntu:22.04"
    echo "  docker save ubuntu:22.04 -o ubuntu-22.04.tar"
    echo "  scp ubuntu-22.04.tar 到这台服务器"
    echo "  docker load -i ubuntu-22.04.tar"
    
    # 方案3: 使用其他基础镜像
    echo ""
    echo -e "${YELLOW}方案3: 使用其他基础镜像${NC}"
    echo "修改 Dockerfile，使用其他镜像："
    echo "  FROM alpine:latest  # 更小的镜像"
    echo "  FROM centos:7       # CentOS 镜像"
fi

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}  配置完成${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "后续步骤:"
echo "1. 如果镜像拉取成功，重新构建："
echo "   docker-compose build --no-cache"
echo ""
echo "2. 如果仍然失败，使用离线镜像："
echo "   见上述方案2"
echo ""
echo "3. 查看 Docker 日志："
echo "   journalctl -u docker -f"