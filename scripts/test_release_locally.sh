#!/bin/bash

# 本地测试发布构建脚本

set -e

echo "======================================"
echo "  本地测试Release构建"
echo "======================================"
echo ""

# 检查是否安装了必要的工具
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 未安装"
        return 1
    else
        echo "✅ $1 已安装"
        return 0
    fi
}

echo "1. 检查工具..."
check_tool cargo
check_tool rustc
echo ""

# 获取当前系统信息
OS=$(uname -s)
ARCH=$(uname -m)

echo "2. 系统信息："
echo "   OS: $OS"
echo "   架构: $ARCH"
echo ""

# 确定构建目标
if [[ "$OS" == "Darwin" ]]; then
    if [[ "$ARCH" == "arm64" ]]; then
        TARGET="aarch64-apple-darwin"
    else
        TARGET="x86_64-apple-darwin"
    fi
elif [[ "$OS" == "Linux" ]]; then
    if [[ "$ARCH" == "aarch64" ]]; then
        TARGET="aarch64-unknown-linux-gnu"
    else
        TARGET="x86_64-unknown-linux-gnu"
    fi
else
    echo "不支持的操作系统: $OS"
    exit 1
fi

echo "3. 构建目标: $TARGET"
echo ""

# 添加目标（如果需要）
echo "4. 检查Rust目标..."
if ! rustup target list | grep -q "$TARGET (installed)"; then
    echo "   添加目标: $TARGET"
    rustup target add $TARGET
else
    echo "   ✅ 目标已安装"
fi
echo ""

# 构建
echo "5. 开始构建..."
echo "   执行: cargo build --release --target $TARGET --bin kernel"
cargo build --release --target $TARGET --bin kernel

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ 构建成功！"
    
    # 检查二进制文件
    BINARY_PATH="target/$TARGET/release/kernel"
    if [[ "$OS" == "Windows" ]]; then
        BINARY_PATH="target/$TARGET/release/kernel.exe"
    fi
    
    echo ""
    echo "6. 二进制文件信息："
    ls -lh $BINARY_PATH
    file $BINARY_PATH || true
    
    # 创建发布包
    echo ""
    echo "7. 创建发布包..."
    RELEASE_NAME="aurelia-${TARGET}"
    
    cd target/$TARGET/release
    if [[ "$OS" == "Windows" ]]; then
        zip ../../../${RELEASE_NAME}.zip kernel.exe
        echo "   ✅ 创建: ${RELEASE_NAME}.zip"
    else
        tar czf ../../../${RELEASE_NAME}.tar.gz kernel
        echo "   ✅ 创建: ${RELEASE_NAME}.tar.gz"
    fi
    cd ../../../
    
    # 生成SHA256
    echo ""
    echo "8. 生成SHA256校验和..."
    if [[ "$OS" == "Darwin" ]]; then
        shasum -a 256 ${RELEASE_NAME}.tar.gz > ${RELEASE_NAME}.tar.gz.sha256
    elif [[ "$OS" == "Linux" ]]; then
        sha256sum ${RELEASE_NAME}.tar.gz > ${RELEASE_NAME}.tar.gz.sha256
    fi
    
    if [ -f "${RELEASE_NAME}.tar.gz.sha256" ]; then
        echo "   ✅ SHA256: $(cat ${RELEASE_NAME}.tar.gz.sha256)"
    fi
    
    echo ""
    echo "======================================"
    echo "  构建完成"
    echo "======================================"
    echo ""
    echo "发布文件："
    ls -lh aurelia-*.tar.gz aurelia-*.zip 2>/dev/null || true
    echo ""
    echo "下一步："
    echo "1. 测试二进制文件: ./$BINARY_PATH --help"
    echo "2. 创建Git标签: git tag v1.0.0"
    echo "3. 推送标签触发自动发布: git push origin v1.0.0"
else
    echo ""
    echo "❌ 构建失败"
    exit 1
fi