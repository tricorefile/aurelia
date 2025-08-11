#\!/bin/bash

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 全局变量
KERNEL_PID=""
LOG_FILE="aurelia.log"

function print_header() {
    clear
    echo -e "${BLUE}======================================"
    echo -e "    🤖 Aurelia 智能体测试控制台"
    echo -e "======================================${NC}"
    echo ""
}

function start_agent() {
    echo -e "${GREEN}启动智能体...${NC}"
    
    # 检查是否已运行
    if pgrep -f "target/release/kernel" > /dev/null; then
        echo -e "${YELLOW}智能体已在运行${NC}"
        KERNEL_PID=$(pgrep -f "target/release/kernel")
        echo "PID: $KERNEL_PID"
    else
        ./target/release/kernel > $LOG_FILE 2>&1 &
        KERNEL_PID=$\!
        echo -e "${GREEN}✅ 已启动，PID: $KERNEL_PID${NC}"
        sleep 3
        
        # 验证启动
        if ps -p $KERNEL_PID > /dev/null; then
            echo -e "${GREEN}智能体运行正常${NC}"
        else
            echo -e "${RED}❌ 启动失败，请检查日志${NC}"
        fi
    fi
}

function stop_agent() {
    echo -e "${YELLOW}停止智能体...${NC}"
    pkill -f "target/release/kernel"
    sleep 2
    
    if pgrep -f "target/release/kernel" > /dev/null; then
        echo -e "${RED}❌ 停止失败，强制终止...${NC}"
        pkill -9 -f "target/release/kernel"
    else
        echo -e "${GREEN}✅ 已停止${NC}"
    fi
}

function view_logs() {
    echo -e "${BLUE}实时日志 (按Ctrl+C返回主菜单):${NC}"
    echo "----------------------------------------"
    tail -f $LOG_FILE
}

function check_health() {
    echo -e "${BLUE}=== 健康状态检查 ===${NC}"
    echo ""
    
    if pgrep -f "target/release/kernel" > /dev/null; then
        echo -e "${GREEN}✅ 智能体运行中${NC}"
        
        # 显示最近的健康检查
        echo -e "\n${YELLOW}最近健康检查记录:${NC}"
        grep "health" $LOG_FILE | tail -5
        
        # 显示CPU/内存使用
        echo -e "\n${YELLOW}系统资源使用:${NC}"
        PID=$(pgrep -f "target/release/kernel")
        ps aux | grep $PID | grep -v grep | awk '{print "CPU: "$3"%, Memory: "$4"%"}'
        
        # 显示监控状态
        echo -e "\n${YELLOW}监控服务状态:${NC}"
        grep "Monitoring.*agents" $LOG_FILE | tail -1
    else
        echo -e "${RED}❌ 智能体未运行${NC}"
    fi
}

function test_decision() {
    echo -e "${BLUE}=== 测试决策系统 ===${NC}"
    
    if \! pgrep -f "target/release/kernel" > /dev/null; then
        echo -e "${YELLOW}启动智能体进行测试...${NC}"
        start_agent
    fi
    
    echo "观察30秒内的决策..."
    
    # 清空临时日志
    > decision_test.log
    
    # 收集30秒的决策日志
    timeout 30 tail -f $LOG_FILE | grep -i "decision" > decision_test.log &
    TAIL_PID=$\!
    
    for i in {30..1}; do
        echo -ne "\r等待中... $i 秒"
        sleep 1
    done
    echo ""
    
    kill $TAIL_PID 2>/dev/null
    
    # 分析结果
    DECISION_COUNT=$(wc -l < decision_test.log)
    echo -e "\n${GREEN}决策统计:${NC}"
    echo "决策次数: $DECISION_COUNT"
    
    if [ $DECISION_COUNT -gt 0 ]; then
        echo -e "${GREEN}✅ 决策系统正常${NC}"
        echo -e "\n最近的决策:"
        cat decision_test.log | tail -3
    else
        echo -e "${RED}❌ 未检测到决策活动${NC}"
    fi
}

function test_replication() {
    echo -e "${BLUE}=== 测试自我复制 ===${NC}"
    
    echo "检查复制尝试..."
    grep -i "replication\|replica" $LOG_FILE | tail -10
    
    echo -e "\n${YELLOW}提示: 完整的复制测试需要配置目标服务器${NC}"
    echo "可以使用 Docker 环境进行测试:"
    echo "  cd scripts/docker && docker-compose up -d"
}

function test_scheduler() {
    echo -e "${BLUE}=== 测试任务调度 ===${NC}"
    
    echo "统计1分钟内的任务执行..."
    
    # 记录开始时间
    START_TIME=$(date +%s)
    HEALTH_COUNT=0
    TASK_COUNT=0
    
    while [ $(($(date +%s) - START_TIME)) -lt 60 ]; do
        # 统计新增的任务
        NEW_HEALTH=$(grep -c "health" $LOG_FILE)
        NEW_TASK=$(grep -c "task" $LOG_FILE)
        
        echo -ne "\r健康检查: $NEW_HEALTH | 任务执行: $NEW_TASK | 剩余: $((60 - $(date +%s) + START_TIME))秒"
        sleep 1
    done
    
    echo -e "\n\n${GREEN}任务调度统计完成${NC}"
    echo "健康检查: $NEW_HEALTH 次"
    echo "任务执行: $NEW_TASK 次"
}

function simulate_failure() {
    echo -e "${BLUE}=== 模拟故障 ===${NC}"
    
    if \! pgrep -f "target/release/kernel" > /dev/null; then
        echo -e "${RED}智能体未运行，无法模拟故障${NC}"
        return
    fi
    
    PID=$(pgrep -f "target/release/kernel")
    
    echo -e "${YELLOW}1. 暂停进程 (模拟挂起)${NC}"
    kill -STOP $PID
    sleep 3
    
    echo -e "${YELLOW}2. 检查状态${NC}"
    if ps -p $PID > /dev/null; then
        echo "进程已暂停"
    fi
    
    echo -e "${YELLOW}3. 恢复进程${NC}"
    kill -CONT $PID
    sleep 2
    
    echo -e "${GREEN}✅ 故障模拟完成${NC}"
    echo "检查恢复日志:"
    grep -i "recover" $LOG_FILE | tail -5
}

function view_metrics() {
    echo -e "${BLUE}=== 性能指标 ===${NC}"
    
    if pgrep -f "target/release/kernel" > /dev/null; then
        PID=$(pgrep -f "target/release/kernel")
        
        echo -e "${GREEN}进程信息:${NC}"
        ps aux | head -1
        ps aux | grep $PID | grep -v grep
        
        echo -e "\n${GREEN}内存映射:${NC}"
        if command -v pmap > /dev/null; then
            pmap $PID | tail -1
        else
            echo "pmap 命令不可用"
        fi
        
        echo -e "\n${GREEN}打开的文件:${NC}"
        lsof -p $PID 2>/dev/null | wc -l
        
        echo -e "\n${GREEN}网络连接:${NC}"
        lsof -p $PID -i 2>/dev/null | head -5
    else
        echo -e "${RED}智能体未运行${NC}"
    fi
}

function quick_test() {
    echo -e "${BLUE}=== 快速测试所有功能 ===${NC}"
    
    echo -e "\n${YELLOW}1. 启动测试${NC}"
    start_agent
    sleep 5
    
    echo -e "\n${YELLOW}2. 健康检查${NC}"
    check_health
    
    echo -e "\n${YELLOW}3. 决策测试${NC}"
    grep -i "decision" $LOG_FILE | tail -3
    
    echo -e "\n${YELLOW}4. 任务调度${NC}"
    grep -i "task" $LOG_FILE | tail -3
    
    echo -e "\n${YELLOW}5. 监控服务${NC}"
    grep -i "monitoring" $LOG_FILE | tail -3
    
    echo -e "\n${GREEN}✅ 快速测试完成${NC}"
}

# 主循环
while true; do
    print_header
    
    # 显示当前状态
    if pgrep -f "target/release/kernel" > /dev/null; then
        echo -e "状态: ${GREEN}● 运行中${NC} (PID: $(pgrep -f 'target/release/kernel'))"
    else
        echo -e "状态: ${RED}● 已停止${NC}"
    fi
    
    echo -e "\n${YELLOW}基础操作:${NC}"
    echo "  1. 启动智能体"
    echo "  2. 停止智能体"
    echo "  3. 查看实时日志"
    echo "  4. 查看健康状态"
    
    echo -e "\n${YELLOW}功能测试:${NC}"
    echo "  5. 测试决策系统"
    echo "  6. 测试自我复制"
    echo "  7. 测试任务调度"
    echo "  8. 模拟故障"
    
    echo -e "\n${YELLOW}监控分析:${NC}"
    echo "  9. 查看性能指标"
    echo "  10. 快速测试所有功能"
    
    echo -e "\n${YELLOW}其他:${NC}"
    echo "  0. 退出"
    
    echo -e "${BLUE}======================================${NC}"
    read -p "请选择操作 [0-10]: " choice
    
    case $choice in
        1) start_agent ;;
        2) stop_agent ;;
        3) view_logs ;;
        4) check_health ;;
        5) test_decision ;;
        6) test_replication ;;
        7) test_scheduler ;;
        8) simulate_failure ;;
        9) view_metrics ;;
        10) quick_test ;;
        0) 
            echo -e "${GREEN}再见！${NC}"
            exit 0 
            ;;
        *)
            echo -e "${RED}无效选项${NC}"
            ;;
    esac
    
    echo ""
    read -p "按Enter键继续..."
done
