#!/bin/bash

# 手动SSH连接测试脚本
# 用于快速测试SSH连接

HOST="194.146.13.14"
PORT=22
USER="root"
PASSWORD="A8vd0VHDGlpQY3Vu37eCz400fCC1b"

echo "====================================="
echo "  SSH 连接手动测试"
echo "====================================="
echo ""
echo "目标服务器: $USER@$HOST:$PORT"
echo ""

# 1. 网络测试
echo "1. 测试网络连通性..."
if ping -c 1 $HOST > /dev/null 2>&1; then
    echo "   ✅ 网络连通"
else
    echo "   ❌ 网络不通"
    exit 1
fi

# 2. 端口测试
echo "2. 测试SSH端口..."
if nc -zv $HOST $PORT 2>&1 | grep -q succeeded; then
    echo "   ✅ 端口 $PORT 开放"
else
    echo "   ❌ 端口 $PORT 关闭"
    exit 1
fi

# 3. 获取SSH Banner
echo "3. 获取SSH服务信息..."
echo -n "   Banner: "
timeout 2 bash -c "echo '' | nc $HOST $PORT 2>/dev/null" | head -1

# 4. 尝试SSH连接
echo ""
echo "4. 测试SSH连接..."
echo "   注意: 需要手动输入密码"
echo "   密码: $PASSWORD"
echo ""
echo "   执行命令: ssh -o StrictHostKeyChecking=no $USER@$HOST"
echo ""
echo "-----------------------------------"
echo "请手动执行以下命令进行测试:"
echo ""
echo "# 基本连接（需要输入密码）"
echo "ssh $USER@$HOST"
echo ""
echo "# 跳过host key检查"
echo "ssh -o StrictHostKeyChecking=no $USER@$HOST"
echo ""
echo "# 详细调试模式"
echo "ssh -vvv $USER@$HOST"
echo ""
echo "# 如果安装了sshpass"
echo "sshpass -p '$PASSWORD' ssh -o StrictHostKeyChecking=no $USER@$HOST 'hostname'"
echo ""
echo "-----------------------------------"

# 5. 测试Rust SSH部署
echo ""
echo "5. 测试Rust SSH部署功能..."
if [ -f "./target/debug/examples/test_ssh_deploy" ]; then
    echo "   运行: ./target/debug/examples/test_ssh_deploy"
    echo "   或编译运行: cargo run --example test_ssh_deploy"
else
    echo "   先编译: cargo build --example test_ssh_deploy"
    echo "   再运行: ./target/debug/examples/test_ssh_deploy"
fi

echo ""
echo "====================================="
echo "  测试完成"
echo "====================================="
echo ""
echo "如果SSH连接失败，请检查:"
echo "  1. 密码是否正确: $PASSWORD"
echo "  2. 服务器是否允许root登录"
echo "  3. 服务器是否允许密码认证"
echo "  4. 防火墙是否有IP限制"
echo ""
echo "查看详细指南: cat SSH_TESTING_GUIDE.md"
