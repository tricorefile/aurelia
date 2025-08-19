#!/bin/bash
# GitHub Actions Runner 诊断脚本

set -e

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  GitHub Actions Runner 诊断工具${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 读取配置
if [ -f .env ]; then
    source .env
else
    echo -e "${RED}错误: .env 文件不存在${NC}"
    exit 1
fi

GITHUB_OWNER=${GITHUB_OWNER:-tricorefile}
GITHUB_REPOSITORY=${GITHUB_REPOSITORY:-aurelia}

echo -e "${YELLOW}配置信息:${NC}"
echo "  Owner: $GITHUB_OWNER"
echo "  Repository: $GITHUB_REPOSITORY"
echo ""

# 1. 检查 Token 格式
echo -e "${YELLOW}[1/5] 检查 Token 格式...${NC}"

if [[ -z "$GITHUB_TOKEN" ]]; then
    echo -e "${RED}✗ GITHUB_TOKEN 未设置${NC}"
    exit 1
fi

# 检查 Token 中的特殊字符
if echo "$GITHUB_TOKEN" | grep -q $'\n'; then
    echo -e "${RED}✗ Token 包含换行符${NC}"
    echo "  请移除 Token 中的换行符"
    exit 1
fi

if echo "$GITHUB_TOKEN" | grep -q ' '; then
    echo -e "${RED}✗ Token 包含空格${NC}"
    echo "  请移除 Token 中的空格"
    exit 1
fi

TOKEN_LENGTH=${#GITHUB_TOKEN}
echo "  Token 长度: $TOKEN_LENGTH 字符"

if [[ "$GITHUB_TOKEN" == ghp_* ]]; then
    echo -e "${GREEN}✓ 检测到 Personal Access Token (PAT)${NC}"
    TOKEN_TYPE="PAT"
elif [[ $TOKEN_LENGTH -gt 100 ]] && [[ "$GITHUB_TOKEN" == *"AA"* ]]; then
    echo -e "${GREEN}✓ 检测到 Registration Token${NC}"
    TOKEN_TYPE="REGISTRATION"
else
    echo -e "${YELLOW}⚠ 未知的 Token 类型${NC}"
    TOKEN_TYPE="UNKNOWN"
fi

# 2. 验证 PAT 权限
if [[ "$TOKEN_TYPE" == "PAT" ]]; then
    echo ""
    echo -e "${YELLOW}[2/5] 验证 PAT 权限...${NC}"
    
    # 测试基本认证
    echo -n "  基本认证: "
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
        -H "Authorization: token ${GITHUB_TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/user")
    
    if [[ "$HTTP_CODE" == "200" ]]; then
        echo -e "${GREEN}✓ 成功 (HTTP $HTTP_CODE)${NC}"
    else
        echo -e "${RED}✗ 失败 (HTTP $HTTP_CODE)${NC}"
        echo -e "${RED}  Token 无效或已过期${NC}"
        exit 1
    fi
    
    # 获取用户信息
    USER_INFO=$(curl -s \
        -H "Authorization: token ${GITHUB_TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/user")
    
    USERNAME=$(echo "$USER_INFO" | jq -r .login)
    echo "  认证用户: $USERNAME"
    
    # 检查仓库访问权限
    echo -n "  仓库访问: "
    REPO_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
        -H "Authorization: token ${GITHUB_TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPOSITORY}")
    
    if [[ "$REPO_CODE" == "200" ]]; then
        echo -e "${GREEN}✓ 成功${NC}"
    else
        echo -e "${RED}✗ 失败 (HTTP $REPO_CODE)${NC}"
        echo -e "${RED}  无法访问仓库 ${GITHUB_OWNER}/${GITHUB_REPOSITORY}${NC}"
        exit 1
    fi
    
    # 检查管理员权限
    echo -n "  管理员权限: "
    REPO_INFO=$(curl -s \
        -H "Authorization: token ${GITHUB_TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPOSITORY}")
    
    PERMISSIONS=$(echo "$REPO_INFO" | jq -r .permissions)
    IS_ADMIN=$(echo "$PERMISSIONS" | jq -r .admin)
    
    if [[ "$IS_ADMIN" == "true" ]]; then
        echo -e "${GREEN}✓ 有管理员权限${NC}"
    else
        echo -e "${YELLOW}⚠ 无管理员权限${NC}"
        echo "    注意: 需要管理员权限才能注册 Runner"
    fi
else
    echo ""
    echo -e "${YELLOW}[2/5] 跳过 PAT 验证 (使用 Registration Token)${NC}"
fi

# 3. 测试获取 Registration Token
if [[ "$TOKEN_TYPE" == "PAT" ]]; then
    echo ""
    echo -e "${YELLOW}[3/5] 测试获取 Registration Token...${NC}"
    
    API_URL="https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPOSITORY}/actions/runners/registration-token"
    echo "  API URL: $API_URL"
    
    RESPONSE=$(curl -sX POST \
        -H "Authorization: token ${GITHUB_TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "$API_URL")
    
    # 检查响应
    if echo "$RESPONSE" | jq -e .token > /dev/null 2>&1; then
        REG_TOKEN=$(echo "$RESPONSE" | jq -r .token)
        echo -e "${GREEN}✓ 成功获取 Registration Token${NC}"
        echo "  Token 前10字符: ${REG_TOKEN:0:10}..."
        
        # 获取过期时间
        EXPIRES_AT=$(echo "$RESPONSE" | jq -r .expires_at)
        echo "  过期时间: $EXPIRES_AT"
    else
        echo -e "${RED}✗ 无法获取 Registration Token${NC}"
        echo "  响应: $RESPONSE"
        
        if echo "$RESPONSE" | grep -q "404"; then
            echo ""
            echo -e "${RED}错误原因分析:${NC}"
            echo "  1. 仓库不存在或名称错误"
            echo "  2. PAT 缺少必要权限:"
            echo "     - Actions: Read"
            echo "     - Administration: Read & Write"
            echo "     - Metadata: Read"
            echo "  3. 用户不是仓库管理员"
        fi
        exit 1
    fi
else
    echo ""
    echo -e "${YELLOW}[3/5] 跳过 Registration Token 测试${NC}"
    REG_TOKEN="$GITHUB_TOKEN"
fi

# 4. 检查现有 Runners
echo ""
echo -e "${YELLOW}[4/5] 检查现有 Runners...${NC}"

if [[ "$TOKEN_TYPE" == "PAT" ]]; then
    RUNNERS=$(curl -s \
        -H "Authorization: token ${GITHUB_TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPOSITORY}/actions/runners")
    
    if echo "$RUNNERS" | jq -e .runners > /dev/null 2>&1; then
        RUNNER_COUNT=$(echo "$RUNNERS" | jq '.runners | length')
        echo "  已注册 Runners: $RUNNER_COUNT"
        
        if [[ $RUNNER_COUNT -gt 0 ]]; then
            echo ""
            echo "  Runner 列表:"
            echo "$RUNNERS" | jq -r '.runners[] | "    - \(.name): \(.status) (\(.os)/\(.arch))"'
        fi
    else
        echo -e "${YELLOW}⚠ 无法获取 Runner 列表${NC}"
    fi
else
    echo "  需要 PAT 才能查看 Runner 列表"
fi

# 5. 测试 Runner 配置
echo ""
echo -e "${YELLOW}[5/5] 测试 Runner 配置...${NC}"

# 检查 Docker
if command -v docker &> /dev/null; then
    echo -e "${GREEN}✓ Docker 已安装${NC}"
    docker --version
else
    echo -e "${RED}✗ Docker 未安装${NC}"
fi

# 检查 docker-compose
if command -v docker-compose &> /dev/null; then
    echo -e "${GREEN}✓ Docker Compose 已安装${NC}"
    docker-compose --version
else
    echo -e "${RED}✗ Docker Compose 未安装${NC}"
fi

# 检查容器状态
if command -v docker &> /dev/null; then
    echo ""
    echo "  运行中的 Runner 容器:"
    docker ps --filter "name=runner" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | head -5
fi

# 总结
echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  诊断完成${NC}"
echo -e "${BLUE}========================================${NC}"

if [[ "$TOKEN_TYPE" == "PAT" ]] && [[ -n "$REG_TOKEN" ]]; then
    echo -e "${GREEN}✓ 所有检查通过${NC}"
    echo ""
    echo "建议下一步操作:"
    echo "1. 使用修复版 entrypoint.sh:"
    echo "   cp entrypoint-fixed.sh entrypoint.sh"
    echo ""
    echo "2. 重启 Runner:"
    echo "   docker-compose down"
    echo "   docker-compose up -d"
    echo ""
    echo "3. 查看日志:"
    echo "   docker-compose logs -f"
elif [[ "$TOKEN_TYPE" == "REGISTRATION" ]]; then
    echo -e "${YELLOW}⚠ 使用 Registration Token${NC}"
    echo "  Registration Token 只能使用一次"
    echo "  建议使用 PAT 以便自动续期"
else
    echo -e "${RED}✗ 存在问题需要解决${NC}"
fi