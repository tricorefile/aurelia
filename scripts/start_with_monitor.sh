#!/bin/bash

echo "======================================"
echo "   🚀 启动 Aurelia 智能体 + 监控面板"
echo "======================================"
echo ""

# 检查Python3
if ! command -v python3 &> /dev/null; then
    echo "❌ 需要 Python3 来运行监控面板"
    echo "请安装 Python3: brew install python3"
    exit 1
fi

# 清理旧进程
echo "🧹 清理旧进程..."
pkill -f kernel 2>/dev/null
pkill -f simple_monitor_server.py 2>/dev/null
sleep 2

# 编译项目
echo "🔨 编译项目..."
if cargo build --release 2>&1 | grep -q "Finished"; then
    echo "✅ 编译成功"
else
    echo "⚠️  正在编译，请稍候..."
    cargo build --release
fi

echo ""
echo "🤖 启动智能体..."
./target/release/kernel > aurelia_output.log 2>&1 &
KERNEL_PID=$!
echo "   PID: $KERNEL_PID"

sleep 3

# 验证智能体启动
if ps -p $KERNEL_PID > /dev/null; then
    echo "✅ 智能体运行正常"
else
    echo "❌ 智能体启动失败"
    exit 1
fi

echo ""
echo "📊 启动监控面板..."
python3 simple_monitor_server.py &
MONITOR_PID=$!
echo "   PID: $MONITOR_PID"

sleep 2

echo ""
echo "======================================"
echo "        ✨ 系统已启动完成 ✨"
echo "======================================"
echo ""
echo "📊 监控面板: http://localhost:3030"
echo "📝 查看日志: tail -f aurelia_output.log"
echo "🛑 停止系统: pkill -f kernel && pkill -f simple_monitor_server.py"
echo ""
echo "提示: 在浏览器中打开 http://localhost:3030 查看实时监控"
echo ""
echo "按 Ctrl+C 停止所有服务"

# 等待用户中断
trap "echo ''; echo '正在停止服务...'; kill $KERNEL_PID $MONITOR_PID 2>/dev/null; exit" INT

# 保持脚本运行
while true; do
    sleep 1
done