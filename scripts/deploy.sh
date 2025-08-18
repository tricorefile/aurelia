#!/bin/bash

# Aurelia Remote Deployment Script
# 用于部署和管理远程服务器上的Aurelia系统

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 默认配置
REMOTE_USER="root"
REMOTE_PATH="/opt/aurelia"
LOCAL_BINARY="./target/release/kernel"
CONFIG_FILE="config/target_servers.json"

# 函数：显示使用说明
show_usage() {
    echo "使用方法: $0 <command> [options]"
    echo ""
    echo "命令:"
    echo "  deploy <server_ip>     - 部署Aurelia到远程服务器"
    echo "  start <server_ip>      - 启动远程Aurelia服务"
    echo "  stop <server_ip>       - 停止远程Aurelia服务"
    echo "  status <server_ip>     - 查看远程服务状态"
    echo "  logs <server_ip>       - 查看远程服务日志"
    echo "  update <server_ip>     - 更新远程服务器上的二进制文件"
    echo "  clean <server_ip>      - 清理远程服务器上的Aurelia目录"
    echo ""
    echo "选项:"
    echo "  -u <user>              - SSH用户名 (默认: root)"
    echo "  -p <path>              - 远程路径 (默认: /opt/aurelia)"
    echo "  -k <key_path>          - SSH密钥路径"
    echo "  -P <password>          - SSH密码"
    echo ""
    echo "示例:"
    echo "  $0 deploy 194.146.13.14 -u ubuntu -k ~/.ssh/id_rsa"
    echo "  $0 start 194.146.13.14 -P 'your_password'"
    echo "  $0 logs 194.146.13.14"
}

# 函数：解析参数
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -u|--user)
                REMOTE_USER="$2"
                shift 2
                ;;
            -p|--path)
                REMOTE_PATH="$2"
                shift 2
                ;;
            -k|--key)
                SSH_KEY="$2"
                SSH_OPTS="-i $SSH_KEY"
                shift 2
                ;;
            -P|--password)
                SSH_PASSWORD="$2"
                shift 2
                ;;
            *)
                shift
                ;;
        esac
    done
}

# 函数：执行SSH命令
ssh_exec() {
    local server=$1
    local command=$2
    
    if [ -n "$SSH_PASSWORD" ]; then
        sshpass -p "$SSH_PASSWORD" ssh $SSH_OPTS -o StrictHostKeyChecking=no $REMOTE_USER@$server "$command"
    else
        ssh $SSH_OPTS -o StrictHostKeyChecking=no $REMOTE_USER@$server "$command"
    fi
}

# 函数：SCP文件传输
scp_upload() {
    local server=$1
    local source=$2
    local dest=$3
    
    if [ -n "$SSH_PASSWORD" ]; then
        sshpass -p "$SSH_PASSWORD" scp $SSH_OPTS -o StrictHostKeyChecking=no $source $REMOTE_USER@$server:$dest
    else
        scp $SSH_OPTS -o StrictHostKeyChecking=no $source $REMOTE_USER@$server:$dest
    fi
}

# 函数：部署到服务器
deploy_to_server() {
    local server=$1
    echo -e "${GREEN}[部署] 开始部署到 $server${NC}"
    
    # 1. 编译本地二进制文件
    echo -e "${YELLOW}[编译] 正在编译release版本...${NC}"
    cargo build --release
    
    # 2. 创建远程目录
    echo -e "${YELLOW}[SSH] 创建远程目录...${NC}"
    ssh_exec $server "mkdir -p $REMOTE_PATH/{logs,config,data}"
    
    # 3. 上传二进制文件
    echo -e "${YELLOW}[上传] 上传二进制文件...${NC}"
    scp_upload $server $LOCAL_BINARY $REMOTE_PATH/kernel
    ssh_exec $server "chmod +x $REMOTE_PATH/kernel"
    
    # 4. 上传配置文件
    if [ -f "$CONFIG_FILE" ]; then
        echo -e "${YELLOW}[上传] 上传配置文件...${NC}"
        scp_upload $server $CONFIG_FILE $REMOTE_PATH/config/
    fi
    
    # 5. 创建systemd服务文件
    echo -e "${YELLOW}[配置] 创建systemd服务...${NC}"
    cat > /tmp/aurelia.service <<EOF
[Unit]
Description=Aurelia Autonomous Trading System
After=network.target

[Service]
Type=simple
User=$REMOTE_USER
WorkingDirectory=$REMOTE_PATH
ExecStart=$REMOTE_PATH/kernel
Restart=always
RestartSec=10
StandardOutput=append:$REMOTE_PATH/logs/aurelia.log
StandardError=append:$REMOTE_PATH/logs/aurelia.error.log

[Install]
WantedBy=multi-user.target
EOF
    
    scp_upload $server /tmp/aurelia.service /tmp/aurelia.service
    ssh_exec $server "sudo mv /tmp/aurelia.service /etc/systemd/system/ && sudo systemctl daemon-reload"
    
    echo -e "${GREEN}[完成] 部署完成！${NC}"
    echo -e "${YELLOW}使用 '$0 start $server' 启动服务${NC}"
}

# 函数：启动服务
start_service() {
    local server=$1
    echo -e "${GREEN}[启动] 启动Aurelia服务 on $server${NC}"
    
    # 使用systemd启动
    ssh_exec $server "sudo systemctl start aurelia"
    sleep 2
    
    # 检查状态
    ssh_exec $server "sudo systemctl status aurelia --no-pager" || true
    echo -e "${GREEN}[完成] 服务已启动${NC}"
}

# 函数：停止服务
stop_service() {
    local server=$1
    echo -e "${YELLOW}[停止] 停止Aurelia服务 on $server${NC}"
    
    ssh_exec $server "sudo systemctl stop aurelia" || true
    ssh_exec $server "pkill -f kernel" || true
    
    echo -e "${GREEN}[完成] 服务已停止${NC}"
}

# 函数：查看服务状态
check_status() {
    local server=$1
    echo -e "${GREEN}[状态] 检查服务状态 on $server${NC}"
    
    echo -e "\n${YELLOW}Systemd服务状态:${NC}"
    ssh_exec $server "sudo systemctl status aurelia --no-pager" || true
    
    echo -e "\n${YELLOW}进程状态:${NC}"
    ssh_exec $server "ps aux | grep kernel | grep -v grep" || echo "没有运行中的kernel进程"
    
    echo -e "\n${YELLOW}端口监听:${NC}"
    ssh_exec $server "ss -tlnp | grep -E '(8080|3030)'" || echo "没有监听端口"
}

# 函数：查看日志
view_logs() {
    local server=$1
    echo -e "${GREEN}[日志] 查看最近日志 on $server${NC}"
    
    echo -e "\n${YELLOW}最近50行日志:${NC}"
    ssh_exec $server "tail -n 50 $REMOTE_PATH/logs/aurelia.log 2>/dev/null || journalctl -u aurelia -n 50 --no-pager"
}

# 函数：更新二进制文件
update_binary() {
    local server=$1
    echo -e "${GREEN}[更新] 更新二进制文件 on $server${NC}"
    
    # 编译
    echo -e "${YELLOW}[编译] 正在编译release版本...${NC}"
    cargo build --release
    
    # 停止服务
    stop_service $server
    
    # 上传新文件
    echo -e "${YELLOW}[上传] 上传新的二进制文件...${NC}"
    scp_upload $server $LOCAL_BINARY $REMOTE_PATH/kernel
    ssh_exec $server "chmod +x $REMOTE_PATH/kernel"
    
    # 重启服务
    start_service $server
    
    echo -e "${GREEN}[完成] 更新完成！${NC}"
}

# 函数：清理远程目录
clean_remote() {
    local server=$1
    echo -e "${YELLOW}[清理] 清理远程Aurelia目录 on $server${NC}"
    
    read -p "确认要清理 $server 上的 $REMOTE_PATH 吗？(y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        stop_service $server
        ssh_exec $server "rm -rf $REMOTE_PATH"
        ssh_exec $server "sudo rm -f /etc/systemd/system/aurelia.service && sudo systemctl daemon-reload"
        echo -e "${GREEN}[完成] 清理完成${NC}"
    else
        echo -e "${YELLOW}[取消] 操作已取消${NC}"
    fi
}

# 主程序
main() {
    if [ $# -lt 2 ]; then
        show_usage
        exit 1
    fi
    
    COMMAND=$1
    SERVER=$2
    shift 2
    
    # 解析额外参数
    parse_args "$@"
    
    # 检查sshpass（如果使用密码）
    if [ -n "$SSH_PASSWORD" ] && ! command -v sshpass &> /dev/null; then
        echo -e "${RED}错误: 使用密码认证需要安装sshpass${NC}"
        echo "安装方法: brew install hudochenkov/sshpass/sshpass"
        exit 1
    fi
    
    # 执行命令
    case $COMMAND in
        deploy)
            deploy_to_server $SERVER
            ;;
        start)
            start_service $SERVER
            ;;
        stop)
            stop_service $SERVER
            ;;
        status)
            check_status $SERVER
            ;;
        logs)
            view_logs $SERVER
            ;;
        update)
            update_binary $SERVER
            ;;
        clean)
            clean_remote $SERVER
            ;;
        *)
            echo -e "${RED}错误: 未知命令 '$COMMAND'${NC}"
            show_usage
            exit 1
            ;;
    esac
}

# 运行主程序
main "$@"