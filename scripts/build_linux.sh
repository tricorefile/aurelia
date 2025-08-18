#!/bin/bash

# 交叉编译Linux二进制文件脚本

set -e

echo "======================================"
echo "  构建Linux二进制文件"
echo "======================================"
echo ""

# 检查目标架构
echo "1. 检查目标架构..."
if ! rustup target list | grep -q "x86_64-unknown-linux-gnu (installed)"; then
    echo "   安装Linux目标..."
    rustup target add x86_64-unknown-linux-gnu
else
    echo "   ✅ Linux目标已安装"
fi

# 安装交叉编译工具（如果需要）
echo ""
echo "2. 检查交叉编译工具..."
if ! command -v x86_64-linux-gnu-gcc &> /dev/null; then
    echo "   ⚠️ 需要安装交叉编译工具链"
    echo "   运行: brew install filosottile/musl-cross/musl-cross"
    echo "   或使用musl目标（静态链接）"
fi

# 使用musl目标编译（静态链接，更兼容）
echo ""
echo "3. 编译Linux二进制文件（使用musl静态链接）..."
echo "   目标: x86_64-unknown-linux-musl"

# 设置环境变量
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="x86_64-linux-musl-gcc"

# 编译
cargo build --release --target x86_64-unknown-linux-musl --bin kernel

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ 编译成功！"
    echo "   二进制文件: target/x86_64-unknown-linux-musl/release/kernel"
    
    # 检查文件
    file target/x86_64-unknown-linux-musl/release/kernel
    
    # 显示大小
    ls -lh target/x86_64-unknown-linux-musl/release/kernel
else
    echo ""
    echo "❌ 编译失败"
    echo ""
    echo "尝试使用gnu目标："
    echo "cargo build --release --target x86_64-unknown-linux-gnu --bin kernel"
fi

echo ""
echo "======================================"
echo "  部署说明"
echo "======================================"
echo ""
echo "1. 上传到服务器："
echo "   scp target/x86_64-unknown-linux-musl/release/kernel ubuntu@106.54.1.130:~/aurelia/"
echo ""
echo "2. 在服务器上运行："
echo "   ssh ubuntu@106.54.1.130"
echo "   cd ~/aurelia"
echo "   chmod +x kernel"
echo "   ./kernel"
echo ""