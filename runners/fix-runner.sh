#!/bin/bash
# 修复 GitHub Actions Runner 404 错误

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  GitHub Runner 404 错误修复工具${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 步骤 1: 停止现有容器
echo -e "${YELLOW}[1/5] 停止现有容器...${NC}"
docker-compose down || true
echo -e "${GREEN}✓ 完成${NC}"

# 步骤 2: 备份当前配置
echo ""
echo -e "${YELLOW}[2/5] 备份当前配置...${NC}"
if [ -f .env ]; then
    cp .env .env.backup.$(date +%Y%m%d_%H%M%S)
    echo -e "${GREEN}✓ .env 已备份${NC}"
fi

# 步骤 3: 验证并修复 Token
echo ""
echo -e "${YELLOW}[3/5] 验证 Token...${NC}"

# 读取现有 token 或请求新的
if [ -f .env ]; then
    source .env
fi

if [ -z "$GITHUB_TOKEN" ]; then
    echo -e "${RED}未找到 GITHUB_TOKEN${NC}"
    echo ""
    echo "请选择 Token 类型:"
    echo "1) Personal Access Token (PAT) - 推荐"
    echo "2) Registration Token (一次性)"
    read -p "选择 (1 或 2): " choice
    
    case $choice in
        1)
            echo ""
            echo "创建 PAT 的步骤:"
            echo "1. 访问: https://github.com/settings/tokens/new"
            echo "2. 设置权限:"
            echo "   - repo (Full control)"
            echo "   - admin:org (如果是组织仓库)"
            echo "3. 生成并复制 Token"
            echo ""
            read -p "请输入 PAT (ghp_...): " GITHUB_TOKEN
            ;;
        2)
            echo ""
            echo "获取 Registration Token:"
            echo "1. 访问: https://github.com/tricorefile/aurelia/settings/actions/runners"
            echo "2. 点击 'New self-hosted runner'"
            echo "3. 复制 --token 后面的值"
            echo ""
            read -p "请输入 Registration Token: " GITHUB_TOKEN
            ;;
        *)
            echo -e "${RED}无效选择${NC}"
            exit 1
            ;;
    esac
fi

# 清理 Token（移除换行和空格）
GITHUB_TOKEN=$(echo "$GITHUB_TOKEN" | tr -d '\n\r ')

# 验证 Token 格式
if [[ "$GITHUB_TOKEN" == ghp_* ]]; then
    echo -e "${GREEN}✓ 检测到 PAT${NC}"
    
    # 验证 PAT
    echo -n "验证 PAT... "
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
        -H "Authorization: token ${GITHUB_TOKEN}" \
        "https://api.github.com/user")
    
    if [[ "$HTTP_CODE" == "200" ]]; then
        echo -e "${GREEN}✓ 有效${NC}"
    else
        echo -e "${RED}✗ 无效 (HTTP $HTTP_CODE)${NC}"
        exit 1
    fi
    
    # 检查仓库权限
    echo -n "检查仓库权限... "
    REPO_RESPONSE=$(curl -s \
        -H "Authorization: token ${GITHUB_TOKEN}" \
        "https://api.github.com/repos/tricorefile/aurelia")
    
    if echo "$REPO_RESPONSE" | jq -e .permissions.admin > /dev/null 2>&1; then
        IS_ADMIN=$(echo "$REPO_RESPONSE" | jq -r .permissions.admin)
        if [[ "$IS_ADMIN" == "true" ]]; then
            echo -e "${GREEN}✓ 管理员权限${NC}"
        else
            echo -e "${YELLOW}⚠ 非管理员${NC}"
            echo "  警告: 需要管理员权限才能注册 Runner"
            echo "  请联系仓库管理员添加权限"
        fi
    else
        echo -e "${RED}✗ 无法访问仓库${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠ 使用 Registration Token${NC}"
fi

# 保存清理后的 Token
echo "GITHUB_TOKEN=${GITHUB_TOKEN}" > .env
echo -e "${GREEN}✓ Token 已保存${NC}"

# 步骤 4: 使用修复版 entrypoint.sh
echo ""
echo -e "${YELLOW}[4/5] 更新 entrypoint.sh...${NC}"

cat > entrypoint.sh << 'EOF'
#!/bin/bash
set -e

GITHUB_OWNER=${GITHUB_OWNER:-tricorefile}
GITHUB_REPOSITORY=${GITHUB_REPOSITORY:-aurelia}
RUNNER_NAME=${RUNNER_NAME:-docker-runner-$(hostname)}
RUNNER_LABELS=${RUNNER_LABELS:-self-hosted,linux,x64,docker,aurelia}

echo "Configuring runner for ${GITHUB_OWNER}/${GITHUB_REPOSITORY}"

# Get registration token
if [[ "$GITHUB_TOKEN" == ghp_* ]]; then
    echo "Using PAT to get registration token..."
    REG_TOKEN=$(curl -sX POST \
        -H "Authorization: token ${GITHUB_TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPOSITORY}/actions/runners/registration-token" \
        | jq -r .token)
    
    if [[ -z "$REG_TOKEN" ]] || [[ "$REG_TOKEN" == "null" ]]; then
        echo "Failed to get registration token"
        exit 1
    fi
else
    echo "Using provided registration token"
    REG_TOKEN="$GITHUB_TOKEN"
fi

# Configure runner
./config.sh \
    --url "https://github.com/${GITHUB_OWNER}/${GITHUB_REPOSITORY}" \
    --token "${REG_TOKEN}" \
    --name "${RUNNER_NAME}" \
    --labels "${RUNNER_LABELS}" \
    --work "_work" \
    --unattended \
    --replace

# Start runner
exec ./run.sh
EOF

chmod +x entrypoint.sh
echo -e "${GREEN}✓ entrypoint.sh 已更新${NC}"

# 步骤 5: 重建并启动
echo ""
echo -e "${YELLOW}[5/5] 重建并启动 Runner...${NC}"

# 重建镜像
docker-compose build --no-cache

# 启动容器
docker-compose up -d

# 等待启动
sleep 5

# 检查状态
echo ""
echo -e "${GREEN}检查 Runner 状态:${NC}"
docker-compose ps

echo ""
echo -e "${GREEN}查看日志:${NC}"
docker-compose logs --tail=50

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  修复完成${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "后续步骤:"
echo "1. 查看实时日志: docker-compose logs -f"
echo "2. 检查 GitHub: https://github.com/tricorefile/aurelia/settings/actions/runners"
echo "3. 如果仍有问题，运行: ./diagnose.sh"