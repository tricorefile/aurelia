#!/bin/bash

# 测试腾讯云服务器连接

HOST="106.54.1.130"
USER="ubuntu"
KEY_PATH="$HOME/.ssh/tencent.pem"

echo "======================================"
echo "  腾讯云服务器连接测试"
echo "======================================"
echo ""
echo "服务器: $USER@$HOST"
echo "私钥: $KEY_PATH"
echo ""

# 1. 测试网络
echo "1. 测试网络连接..."
if ping -c 1 $HOST > /dev/null 2>&1; then
    echo "   ✅ 网络连通"
else
    echo "   ❌ 网络不通"
fi

# 2. 测试端口
echo "2. 测试SSH端口..."
if nc -zv $HOST 22 2>&1 | grep -q succeeded; then
    echo "   ✅ 端口22开放"
else
    echo "   ❌ 端口22关闭"
fi

# 3. 获取SSH指纹
echo "3. 获取SSH服务器指纹..."
ssh-keyscan -t rsa -H $HOST 2>/dev/null | head -1 | cut -d' ' -f2-3

# 4. 测试SSH连接
echo ""
echo "4. 测试SSH连接..."
echo "   执行: ssh -i $KEY_PATH $USER@$HOST 'hostname'"
echo ""

# 手动测试命令
echo "请手动执行以下命令进行测试："
echo ""
echo "# 基本连接测试"
echo "ssh -i $KEY_PATH $USER@$HOST 'hostname && whoami && pwd'"
echo ""
echo "# 详细调试模式"
echo "ssh -vvv -i $KEY_PATH $USER@$HOST"
echo ""
echo "# 检查私钥权限"
echo "ls -la $KEY_PATH"
echo "chmod 400 $KEY_PATH  # 如果权限不对"
echo ""