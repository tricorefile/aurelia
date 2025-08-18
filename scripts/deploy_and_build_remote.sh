#!/bin/bash

# Deploy source code and build on remote server

set -e

HOST="106.54.1.130"
USER="ubuntu"
KEY_PATH="$HOME/.ssh/tencent.pem"
REMOTE_DIR="/home/ubuntu/aurelia"

echo "======================================"
echo "  远程编译部署脚本"
echo "======================================"
echo ""
echo "目标服务器: $USER@$HOST"
echo "部署目录: $REMOTE_DIR"
echo ""

# 1. 创建远程目录
echo "1. 创建远程目录..."
ssh -i "$KEY_PATH" "$USER@$HOST" "mkdir -p $REMOTE_DIR"

# 2. 同步源代码（排除目标文件和大文件）
echo "2. 同步源代码到服务器..."
rsync -avz --progress \
    --exclude 'target/' \
    --exclude '.git/' \
    --exclude '*.log' \
    --exclude 'py/' \
    --exclude 'docs/' \
    --exclude '.DS_Store' \
    -e "ssh -i $KEY_PATH" \
    ./ "$USER@$HOST:$REMOTE_DIR/"

# 3. 在远程服务器上安装Rust（如果需要）
echo ""
echo "3. 检查远程Rust环境..."
ssh -i "$KEY_PATH" "$USER@$HOST" << 'ENDSSH'
if ! command -v cargo &> /dev/null; then
    echo "安装Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✅ Rust已安装"
    rustc --version
    cargo --version
fi
ENDSSH

# 4. 在远程服务器上编译
echo ""
echo "4. 在远程服务器上编译kernel..."
ssh -i "$KEY_PATH" "$USER@$HOST" << ENDSSH
cd $REMOTE_DIR
source "\$HOME/.cargo/env"

# 安装依赖
sudo apt-get update
sudo apt-get install -y pkg-config libssl-dev

# 编译
echo "开始编译..."
cargo build --release --bin kernel

if [ -f "target/release/kernel" ]; then
    echo "✅ 编译成功！"
    ls -lh target/release/kernel
    
    # 设置执行权限
    chmod +x target/release/kernel
    
    # 测试运行
    echo ""
    echo "测试运行kernel..."
    ./target/release/kernel --version || echo "kernel版本信息不可用"
else
    echo "❌ 编译失败"
    exit 1
fi
ENDSSH

echo ""
echo "======================================"
echo "  部署完成"
echo "======================================"
echo ""
echo "连接到服务器："
echo "  ssh -i $KEY_PATH $USER@$HOST"
echo ""
echo "在服务器上运行："
echo "  cd $REMOTE_DIR"
echo "  ./target/release/kernel"