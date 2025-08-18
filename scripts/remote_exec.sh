#!/bin/bash

# Aurelia Remote Execution Helper
# 快速执行远程命令的辅助脚本

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 从配置文件读取服务器信息
CONFIG_FILE="config/target_servers.json"

# 函数：显示可用服务器
show_servers() {
    echo -e "${BLUE}可用服务器列表:${NC}"
    if [ -f "$CONFIG_FILE" ]; then
        echo -e "${YELLOW}从配置文件加载...${NC}"
        python3 -c "
import json
with open('$CONFIG_FILE') as f:
    data = json.load(f)
    for i, server in enumerate(data['servers'], 1):
        if server.get('enabled', True):
            auth = 'Password' if server['auth_method'] == 'password' else 'SSH Key'
            print(f'  {i}. {server[\"name\"]} - {server[\"ip\"]}:{server[\"port\"]} ({auth})')
"
    else
        echo -e "${RED}配置文件不存在${NC}"
    fi
}

# 函数：快速连接到服务器
quick_connect() {
    if [ -z "$1" ]; then
        show_servers
        echo ""
        read -p "请选择服务器编号: " server_num
    else
        server_num=$1
    fi
    
    # 从配置文件获取服务器信息
    server_info=$(python3 -c "
import json
import sys
try:
    with open('$CONFIG_FILE') as f:
        data = json.load(f)
        servers = [s for s in data['servers'] if s.get('enabled', True)]
        server = servers[int($server_num) - 1]
        print(f\"{server['ip']} {server['port']} {server['username']} {server['auth_method']} {server.get('password', '')} {server.get('ssh_key_path', '')}\")
except:
    sys.exit(1)
" 2>/dev/null)
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}无效的服务器编号${NC}"
        exit 1
    fi
    
    read ip port user auth_method password ssh_key <<< "$server_info"
    
    echo -e "${GREEN}连接到 $ip:$port (用户: $user)${NC}"
    
    if [ "$auth_method" = "password" ]; then
        if command -v sshpass &> /dev/null; then
            sshpass -p "$password" ssh -p $port -o StrictHostKeyChecking=no $user@$ip
        else
            echo -e "${YELLOW}提示: 需要手动输入密码${NC}"
            ssh -p $port $user@$ip
        fi
    else
        if [ -n "$ssh_key" ]; then
            ssh -p $port -i "$ssh_key" $user@$ip
        else
            ssh -p $port $user@$ip
        fi
    fi
}

# 函数：批量执行命令
batch_exec() {
    local command="$1"
    
    if [ -z "$command" ]; then
        echo -e "${RED}请提供要执行的命令${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}在所有启用的服务器上执行: $command${NC}"
    
    python3 -c "
import json
import subprocess
import sys

with open('$CONFIG_FILE') as f:
    data = json.load(f)
    
for server in data['servers']:
    if not server.get('enabled', True):
        continue
        
    print(f\"\\n\033[0;32m[{server['name']}] {server['ip']}\\033[0m\")
    
    if server['auth_method'] == 'password':
        cmd = f\"sshpass -p '{server['password']}' ssh -p {server['port']} -o StrictHostKeyChecking=no {server['username']}@{server['ip']} '{command}'\"
    else:
        key_opt = f\"-i {server['ssh_key_path']}\" if server.get('ssh_key_path') else \"\"
        cmd = f\"ssh -p {server['port']} {key_opt} -o StrictHostKeyChecking=no {server['username']}@{server['ip']} '{command}'\"
    
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    print(result.stdout)
    if result.stderr:
        print(f\"\\033[0;31m{result.stderr}\\033[0m\", file=sys.stderr)
"
}

# 函数：监控所有服务器
monitor_all() {
    echo -e "${BLUE}监控所有服务器状态...${NC}"
    
    while true; do
        clear
        echo -e "${BLUE}=== Aurelia 服务器监控面板 ===${NC}"
        echo -e "${YELLOW}时间: $(date)${NC}\n"
        
        python3 -c "
import json
import subprocess
import sys

with open('$CONFIG_FILE') as f:
    data = json.load(f)
    
for server in data['servers']:
    if not server.get('enabled', True):
        continue
    
    print(f\"\\033[0;34m━━━ {server['name']} ({server['ip']}) ━━━\\033[0m\")
    
    # 构建SSH命令
    if server['auth_method'] == 'password':
        ssh_prefix = f\"sshpass -p '{server['password']}' ssh -p {server['port']} -o StrictHostKeyChecking=no {server['username']}@{server['ip']}\"
    else:
        key_opt = f\"-i {server['ssh_key_path']}\" if server.get('ssh_key_path') else \"\"
        ssh_prefix = f\"ssh -p {server['port']} {key_opt} -o StrictHostKeyChecking=no {server['username']}@{server['ip']}\"
    
    # 检查kernel进程
    cmd = f\"{ssh_prefix} 'ps aux | grep kernel | grep -v grep | head -1'\"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=5)
    
    if result.stdout.strip():
        print(f\"\\033[0;32m✓ Kernel运行中\\033[0m\")
        # 获取CPU和内存使用
        cmd = f\"{ssh_prefix} 'ps aux | grep kernel | grep -v grep | awk \\\"{{print \\\\\\$3,\\\\\\$4}}\\\"'\"
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=5)
        if result.stdout.strip():
            cpu, mem = result.stdout.strip().split()
            print(f\"  CPU: {cpu}% | 内存: {mem}%\")
    else:
        print(f\"\\033[0;31m✗ Kernel未运行\\033[0m\")
    
    # 检查日志最后一行
    cmd = f\"{ssh_prefix} 'tail -1 /opt/aurelia/logs/aurelia.log 2>/dev/null || echo \\\"无日志\\\"'\"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=5)
    if result.stdout.strip():
        print(f\"  最新日志: {result.stdout.strip()[:80]}...\")
    
    print()
"
        
        echo -e "\n${YELLOW}按 Ctrl+C 退出监控${NC}"
        sleep 5
    done
}

# 函数：同步文件到所有服务器
sync_files() {
    local source="$1"
    local dest="${2:-/opt/aurelia/}"
    
    if [ -z "$source" ]; then
        echo -e "${RED}请提供源文件路径${NC}"
        exit 1
    fi
    
    if [ ! -e "$source" ]; then
        echo -e "${RED}源文件不存在: $source${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}同步 $source 到所有服务器的 $dest${NC}"
    
    python3 -c "
import json
import subprocess
import os

with open('$CONFIG_FILE') as f:
    data = json.load(f)
    
for server in data['servers']:
    if not server.get('enabled', True):
        continue
        
    print(f\"\\n\033[0;32m[{server['name']}] 上传到 {server['ip']}:{dest}\\033[0m\")
    
    if server['auth_method'] == 'password':
        cmd = f\"sshpass -p '{server['password']}' scp -P {server['port']} -o StrictHostKeyChecking=no -r {source} {server['username']}@{server['ip']}:{dest}\"
    else:
        key_opt = f\"-i {server['ssh_key_path']}\" if server.get('ssh_key_path') else \"\"
        cmd = f\"scp -P {server['port']} {key_opt} -o StrictHostKeyChecking=no -r {source} {server['username']}@{server['ip']}:{dest}\"
    
    result = subprocess.run(cmd, shell=True)
    if result.returncode == 0:
        print(\"✓ 上传成功\")
    else:
        print(f\"\\033[0;31m✗ 上传失败\\033[0m\")
"
}

# 函数：显示使用说明
show_usage() {
    echo "Aurelia 远程执行辅助工具"
    echo ""
    echo "使用方法: $0 <command> [args]"
    echo ""
    echo "命令:"
    echo "  list               - 显示所有配置的服务器"
    echo "  connect [n]        - 快速连接到服务器 (n为服务器编号)"
    echo "  exec <command>     - 在所有服务器上执行命令"
    echo "  monitor            - 实时监控所有服务器状态"
    echo "  sync <file> [dest] - 同步文件到所有服务器"
    echo "  start-all          - 启动所有服务器上的Aurelia"
    echo "  stop-all           - 停止所有服务器上的Aurelia"
    echo "  logs-all           - 查看所有服务器的最新日志"
    echo ""
    echo "示例:"
    echo "  $0 list                          # 列出所有服务器"
    echo "  $0 connect 1                     # 连接到第1个服务器"
    echo "  $0 exec 'df -h'                  # 查看所有服务器磁盘使用"
    echo "  $0 sync target/release/kernel    # 同步二进制文件"
    echo "  $0 monitor                       # 监控所有服务器"
}

# 主程序
case "$1" in
    list)
        show_servers
        ;;
    connect)
        quick_connect "$2"
        ;;
    exec)
        batch_exec "$2"
        ;;
    monitor)
        monitor_all
        ;;
    sync)
        sync_files "$2" "$3"
        ;;
    start-all)
        batch_exec "sudo systemctl start aurelia || /opt/aurelia/kernel > /opt/aurelia/logs/aurelia.log 2>&1 &"
        ;;
    stop-all)
        batch_exec "sudo systemctl stop aurelia || pkill -f kernel"
        ;;
    logs-all)
        batch_exec "tail -20 /opt/aurelia/logs/aurelia.log 2>/dev/null || journalctl -u aurelia -n 20 --no-pager"
        ;;
    *)
        show_usage
        ;;
esac