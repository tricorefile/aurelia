#!/usr/bin/env python3
"""
Aurelia 监控页面服务器
简单的Python HTTP服务器，用于展示智能体监控数据
"""

import http.server
import socketserver
import json
import subprocess
import os
import time
from datetime import datetime
import threading
import re

PORT = 3030
LOG_FILE = "aurelia_output.log"

# 全局数据存储
monitoring_data = {
    "agents": {},
    "events": [],
    "cluster_status": {
        "total_agents": 0,
        "healthy_agents": 0,
        "cpu_usage": 0,
        "memory_usage": 0,
        "last_update": None
    }
}

def parse_log_file():
    """解析日志文件获取监控数据"""
    global monitoring_data
    
    if not os.path.exists(LOG_FILE):
        return
    
    try:
        with open(LOG_FILE, 'r') as f:
            lines = f.readlines()[-1000:]  # 只读取最后1000行
            
        # 解析监控数据
        for line in lines:
            # 查找监控日志
            if "Monitoring" in line and "agents" in line:
                match = re.search(r'Monitoring (\d+) agents', line)
                if match:
                    monitoring_data["cluster_status"]["total_agents"] = int(match.group(1))
                    monitoring_data["cluster_status"]["healthy_agents"] = int(match.group(1))
            
            # 查找决策日志
            if "decision" in line.lower():
                timestamp = line.split('[0m')[0].split('Z')[0].split('T')[1] if 'Z[0m' in line else datetime.now().strftime("%H:%M:%S")
                monitoring_data["events"].append({
                    "time": timestamp,
                    "type": "decision",
                    "message": "Autonomous decision made"
                })
            
            # 查找健康检查
            if "health" in line.lower():
                timestamp = line.split('[0m')[0].split('Z')[0].split('T')[1] if 'Z[0m' in line else datetime.now().strftime("%H:%M:%S")
                monitoring_data["events"].append({
                    "time": timestamp,
                    "type": "health",
                    "message": "Health check performed"
                })
        
        # 只保留最新的50个事件
        monitoring_data["events"] = monitoring_data["events"][-50:]
        
        # 获取进程信息
        try:
            result = subprocess.run(['pgrep', '-f', 'target/release/kernel'], capture_output=True, text=True)
            if result.stdout.strip():
                pid = result.stdout.strip().split('\n')[0]
                # 获取CPU和内存使用
                ps_result = subprocess.run(['ps', 'aux'], capture_output=True, text=True)
                for line in ps_result.stdout.split('\n'):
                    if pid in line:
                        parts = line.split()
                        monitoring_data["cluster_status"]["cpu_usage"] = float(parts[2])
                        monitoring_data["cluster_status"]["memory_usage"] = float(parts[3])
                        
                monitoring_data["agents"]["localhost"] = {
                    "id": "localhost",
                    "status": "Running",
                    "cpu": monitoring_data["cluster_status"]["cpu_usage"],
                    "memory": monitoring_data["cluster_status"]["memory_usage"],
                    "pid": pid
                }
        except:
            pass
        
        monitoring_data["cluster_status"]["last_update"] = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        
    except Exception as e:
        print(f"Error parsing log: {e}")

def update_monitoring_data():
    """后台线程定期更新监控数据"""
    while True:
        parse_log_file()
        time.sleep(5)

# HTML页面
HTML_CONTENT = """
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Aurelia 监控面板</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        
        .header {
            background: white;
            border-radius: 15px;
            padding: 25px;
            margin-bottom: 25px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
        }
        
        h1 {
            color: #667eea;
            display: flex;
            align-items: center;
            gap: 10px;
        }
        
        .status-badge {
            display: inline-block;
            padding: 5px 15px;
            border-radius: 20px;
            font-size: 14px;
            font-weight: 500;
            margin-left: 20px;
        }
        
        .status-running {
            background: #10b981;
            color: white;
        }
        
        .status-stopped {
            background: #ef4444;
            color: white;
        }
        
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 25px;
        }
        
        .card {
            background: white;
            border-radius: 15px;
            padding: 20px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
        }
        
        .card h3 {
            color: #6b7280;
            font-size: 14px;
            text-transform: uppercase;
            margin-bottom: 10px;
        }
        
        .metric {
            font-size: 36px;
            font-weight: bold;
            color: #1f2937;
        }
        
        .metric-unit {
            font-size: 18px;
            color: #9ca3af;
        }
        
        .progress-bar {
            width: 100%;
            height: 8px;
            background: #e5e7eb;
            border-radius: 4px;
            overflow: hidden;
            margin-top: 10px;
        }
        
        .progress-fill {
            height: 100%;
            background: linear-gradient(90deg, #667eea, #764ba2);
            transition: width 0.3s ease;
        }
        
        .agents-section, .events-section {
            background: white;
            border-radius: 15px;
            padding: 25px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
            margin-bottom: 25px;
        }
        
        .agent-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 15px;
            background: #f9fafb;
            border-radius: 10px;
            margin-bottom: 10px;
        }
        
        .agent-info {
            display: flex;
            align-items: center;
            gap: 15px;
        }
        
        .agent-status-dot {
            width: 12px;
            height: 12px;
            border-radius: 50%;
            background: #10b981;
        }
        
        .agent-metrics {
            display: flex;
            gap: 20px;
        }
        
        .mini-metric {
            text-align: center;
        }
        
        .mini-metric-label {
            font-size: 12px;
            color: #9ca3af;
            text-transform: uppercase;
        }
        
        .mini-metric-value {
            font-size: 18px;
            font-weight: bold;
            color: #1f2937;
        }
        
        .event-item {
            padding: 10px 15px;
            border-left: 3px solid #e5e7eb;
            margin-bottom: 10px;
            background: #f9fafb;
            border-radius: 5px;
        }
        
        .event-health {
            border-left-color: #10b981;
        }
        
        .event-decision {
            border-left-color: #667eea;
        }
        
        .event-time {
            font-size: 12px;
            color: #9ca3af;
        }
        
        .refresh-btn {
            background: linear-gradient(135deg, #667eea, #764ba2);
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 8px;
            cursor: pointer;
            font-weight: 500;
        }
        
        .refresh-btn:hover {
            opacity: 0.9;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>
                🤖 Aurelia 智能体监控
                <span id="status-badge" class="status-badge">加载中...</span>
                <button class="refresh-btn" onclick="refreshData()" style="margin-left: auto;">🔄 刷新</button>
            </h1>
            <p style="margin-top: 10px; color: #6b7280;">
                最后更新: <span id="last-update">-</span>
            </p>
        </div>
        
        <div class="grid">
            <div class="card">
                <h3>活跃智能体</h3>
                <div class="metric">
                    <span id="total-agents">0</span>
                </div>
            </div>
            
            <div class="card">
                <h3>CPU 使用率</h3>
                <div class="metric">
                    <span id="cpu-usage">0</span><span class="metric-unit">%</span>
                </div>
                <div class="progress-bar">
                    <div id="cpu-progress" class="progress-fill" style="width: 0%"></div>
                </div>
            </div>
            
            <div class="card">
                <h3>内存使用率</h3>
                <div class="metric">
                    <span id="memory-usage">0</span><span class="metric-unit">%</span>
                </div>
                <div class="progress-bar">
                    <div id="memory-progress" class="progress-fill" style="width: 0%"></div>
                </div>
            </div>
            
            <div class="card">
                <h3>健康状态</h3>
                <div class="metric" style="font-size: 24px; color: #10b981;">
                    <span id="health-status">✅ 正常</span>
                </div>
            </div>
        </div>
        
        <div class="agents-section">
            <h2 style="margin-bottom: 20px;">智能体状态</h2>
            <div id="agents-list">
                <!-- 动态生成 -->
            </div>
        </div>
        
        <div class="events-section">
            <h2 style="margin-bottom: 20px;">最近事件</h2>
            <div id="events-list" style="max-height: 400px; overflow-y: auto;">
                <!-- 动态生成 -->
            </div>
        </div>
    </div>
    
    <script>
        function refreshData() {
            fetch('/api/status')
                .then(response => response.json())
                .then(data => {
                    // 更新状态
                    const hasAgents = data.cluster_status.total_agents > 0;
                    document.getElementById('status-badge').className = 'status-badge ' + (hasAgents ? 'status-running' : 'status-stopped');
                    document.getElementById('status-badge').textContent = hasAgents ? '运行中' : '已停止';
                    
                    // 更新指标
                    document.getElementById('total-agents').textContent = data.cluster_status.total_agents;
                    document.getElementById('cpu-usage').textContent = data.cluster_status.cpu_usage.toFixed(1);
                    document.getElementById('memory-usage').textContent = data.cluster_status.memory_usage.toFixed(1);
                    document.getElementById('cpu-progress').style.width = data.cluster_status.cpu_usage + '%';
                    document.getElementById('memory-progress').style.width = data.cluster_status.memory_usage + '%';
                    document.getElementById('last-update').textContent = data.cluster_status.last_update || '未知';
                    
                    // 更新健康状态
                    if (data.cluster_status.cpu_usage > 80 || data.cluster_status.memory_usage > 80) {
                        document.getElementById('health-status').textContent = '⚠️ 警告';
                        document.getElementById('health-status').style.color = '#f59e0b';
                    } else if (hasAgents) {
                        document.getElementById('health-status').textContent = '✅ 正常';
                        document.getElementById('health-status').style.color = '#10b981';
                    } else {
                        document.getElementById('health-status').textContent = '❌ 离线';
                        document.getElementById('health-status').style.color = '#ef4444';
                    }
                    
                    // 更新智能体列表
                    const agentsList = document.getElementById('agents-list');
                    agentsList.innerHTML = '';
                    
                    if (Object.keys(data.agents).length > 0) {
                        for (const [id, agent] of Object.entries(data.agents)) {
                            agentsList.innerHTML += `
                                <div class="agent-item">
                                    <div class="agent-info">
                                        <div class="agent-status-dot"></div>
                                        <div>
                                            <strong>${id}</strong>
                                            <div style="font-size: 12px; color: #9ca3af;">PID: ${agent.pid}</div>
                                        </div>
                                    </div>
                                    <div class="agent-metrics">
                                        <div class="mini-metric">
                                            <div class="mini-metric-label">CPU</div>
                                            <div class="mini-metric-value">${agent.cpu.toFixed(1)}%</div>
                                        </div>
                                        <div class="mini-metric">
                                            <div class="mini-metric-label">内存</div>
                                            <div class="mini-metric-value">${agent.memory.toFixed(1)}%</div>
                                        </div>
                                    </div>
                                </div>
                            `;
                        }
                    } else {
                        agentsList.innerHTML = '<p style="color: #9ca3af;">没有活跃的智能体</p>';
                    }
                    
                    // 更新事件列表
                    const eventsList = document.getElementById('events-list');
                    eventsList.innerHTML = '';
                    
                    if (data.events.length > 0) {
                        const reversedEvents = [...data.events].reverse();
                        for (const event of reversedEvents.slice(0, 20)) {
                            const eventClass = event.type === 'health' ? 'event-health' : 'event-decision';
                            eventsList.innerHTML += `
                                <div class="event-item ${eventClass}">
                                    <div style="display: flex; justify-content: space-between;">
                                        <strong>${event.message}</strong>
                                        <span class="event-time">${event.time}</span>
                                    </div>
                                </div>
                            `;
                        }
                    } else {
                        eventsList.innerHTML = '<p style="color: #9ca3af;">暂无事件</p>';
                    }
                })
                .catch(error => {
                    console.error('Error fetching data:', error);
                });
        }
        
        // 自动刷新
        setInterval(refreshData, 5000);
        
        // 初始加载
        refreshData();
    </script>
</body>
</html>
"""

class MonitoringHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/':
            self.send_response(200)
            self.send_header('Content-type', 'text/html; charset=utf-8')
            self.end_headers()
            self.wfile.write(HTML_CONTENT.encode('utf-8'))
        elif self.path == '/api/status':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps(monitoring_data).encode('utf-8'))
        else:
            self.send_error(404, "File not found")
    
    def log_message(self, format, *args):
        # 禁用日志输出
        pass

def main():
    print(f"""
╔══════════════════════════════════════════╗
║     🤖 Aurelia 监控面板服务器            ║
╚══════════════════════════════════════════╝

📊 监控面板地址: http://localhost:{PORT}
📡 API 端点: http://localhost:{PORT}/api/status

正在启动服务器...
""")
    
    # 启动后台数据更新线程
    update_thread = threading.Thread(target=update_monitoring_data, daemon=True)
    update_thread.start()
    
    # 启动HTTP服务器
    with socketserver.TCPServer(("", PORT), MonitoringHandler) as httpd:
        print(f"✅ 服务器已启动在端口 {PORT}")
        print(f"\n🌐 请在浏览器中打开: http://localhost:{PORT}")
        print("\n按 Ctrl+C 停止服务器")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n\n👋 服务器已停止")

if __name__ == "__main__":
    main()