#!/bin/bash
# GitHub Actions Runner 离线安装脚本
# 适用于网络受限环境

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  GitHub Runner 离线安装${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

RUNNER_VERSION="2.311.0"
WORK_DIR="/opt/github-runner"

# 1. 检查是否有离线包
echo -e "${YELLOW}[1/5] 检查离线安装包...${NC}"

if [ ! -f "runner-package.tar.gz" ]; then
    echo -e "${YELLOW}未找到离线包，尝试下载...${NC}"
    
    # 尝试多个下载源
    DOWNLOAD_URLS=(
        "https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz"
        "https://ghproxy.com/https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz"
        "https://download.fastgit.org/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz"
    )
    
    for url in "${DOWNLOAD_URLS[@]}"; do
        echo "尝试: $url"
        if curl -L -o runner-package.tar.gz "$url" --connect-timeout 30 --max-time 300; then
            echo -e "${GREEN}✓ 下载成功${NC}"
            break
        else
            echo -e "${RED}✗ 下载失败${NC}"
        fi
    done
    
    if [ ! -f "runner-package.tar.gz" ]; then
        echo -e "${RED}错误: 无法下载 Runner 包${NC}"
        echo ""
        echo "请手动下载并放置到当前目录:"
        echo "1. 在可以访问 GitHub 的机器上下载:"
        echo "   wget https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz"
        echo "2. 将文件重命名为 runner-package.tar.gz"
        echo "3. 传输到此服务器的当前目录"
        echo "4. 重新运行此脚本"
        exit 1
    fi
fi

echo -e "${GREEN}✓ 找到离线安装包${NC}"

# 2. 创建运行目录
echo ""
echo -e "${YELLOW}[2/5] 创建运行目录...${NC}"

sudo mkdir -p $WORK_DIR
cd $WORK_DIR

# 解压
tar xzf ~/runner-package.tar.gz || tar xzf /opt/github-runner/runner-package.tar.gz || tar xzf ./runner-package.tar.gz
echo -e "${GREEN}✓ 解压完成${NC}"

# 3. 安装依赖
echo ""
echo -e "${YELLOW}[3/5] 安装系统依赖...${NC}"

# 检查并安装必要的依赖
packages_to_install=""

for pkg in curl git jq libicu70; do
    if ! dpkg -l | grep -q "^ii  $pkg"; then
        packages_to_install="$packages_to_install $pkg"
    fi
done

if [ -n "$packages_to_install" ]; then
    echo "需要安装: $packages_to_install"
    
    # 尝试使用镜像源
    if [ -f /etc/apt/sources.list ]; then
        sudo cp /etc/apt/sources.list /etc/apt/sources.list.backup
        sudo sed -i 's/archive.ubuntu.com/mirrors.aliyun.com/g' /etc/apt/sources.list
        sudo sed -i 's/security.ubuntu.com/mirrors.aliyun.com/g' /etc/apt/sources.list
    fi
    
    sudo apt-get update || true
    sudo apt-get install -y $packages_to_install || echo "部分依赖安装失败，可能影响运行"
fi

echo -e "${GREEN}✓ 依赖检查完成${NC}"

# 4. 创建配置脚本
echo ""
echo -e "${YELLOW}[4/5] 创建配置脚本...${NC}"

cat > configure-runner.sh << 'EOF'
#!/bin/bash

# 配置参数
GITHUB_OWNER=${GITHUB_OWNER:-tricorefile}
GITHUB_REPOSITORY=${GITHUB_REPOSITORY:-aurelia}
RUNNER_NAME=${RUNNER_NAME:-offline-runner-$(hostname)}
RUNNER_LABELS=${RUNNER_LABELS:-self-hosted,linux,x64,offline}

# 检查 Token
if [ -z "$GITHUB_TOKEN" ]; then
    echo "错误: 请设置 GITHUB_TOKEN 环境变量"
    echo "export GITHUB_TOKEN=your_token_here"
    exit 1
fi

# 获取注册 Token
if [[ "$GITHUB_TOKEN" == ghp_* ]]; then
    echo "获取注册 Token..."
    REG_TOKEN=$(curl -sX POST \
        -H "Authorization: token ${GITHUB_TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPOSITORY}/actions/runners/registration-token" \
        | jq -r .token)
    
    if [ -z "$REG_TOKEN" ] || [ "$REG_TOKEN" == "null" ]; then
        echo "获取注册 Token 失败"
        exit 1
    fi
else
    REG_TOKEN="$GITHUB_TOKEN"
fi

# 配置 Runner
./config.sh \
    --url "https://github.com/${GITHUB_OWNER}/${GITHUB_REPOSITORY}" \
    --token "${REG_TOKEN}" \
    --name "${RUNNER_NAME}" \
    --labels "${RUNNER_LABELS}" \
    --work "_work" \
    --unattended \
    --replace

echo "Runner 配置完成!"
EOF

chmod +x configure-runner.sh
echo -e "${GREEN}✓ 配置脚本创建完成${NC}"

# 5. 创建 systemd 服务
echo ""
echo -e "${YELLOW}[5/5] 创建系统服务...${NC}"

sudo tee /etc/systemd/system/github-runner.service > /dev/null << EOF
[Unit]
Description=GitHub Actions Runner
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$WORK_DIR
ExecStart=$WORK_DIR/run.sh
Restart=always
RestartSec=10
Environment="RUNNER_ALLOW_RUNASROOT=1"

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
echo -e "${GREEN}✓ 系统服务创建完成${NC}"

# 完成
echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}  离线安装完成！${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "接下来的步骤:"
echo ""
echo "1. 设置 GitHub Token:"
echo "   export GITHUB_TOKEN=ghp_your_token_here"
echo ""
echo "2. 配置 Runner:"
echo "   cd $WORK_DIR"
echo "   ./configure-runner.sh"
echo ""
echo "3. 启动 Runner:"
echo "   方式1 (手动): ./run.sh"
echo "   方式2 (服务): sudo systemctl start github-runner"
echo ""
echo "4. 设置开机自启:"
echo "   sudo systemctl enable github-runner"
echo ""
echo "验证: https://github.com/tricorefile/aurelia/settings/actions/runners"