#!/usr/bin/env python3
"""
Aurelia API监控面板 - 通过Rust API获取数据
"""

import http.server
import socketserver
import json
import urllib.request
import urllib.error
import time
from datetime import datetime
import threading
from collections import deque

# 配置
WEB_PORT = 3030
API_BASE_URL = "http://localhost:8080"
UPDATE_INTERVAL = 3  # 秒

# 全局数据缓存
cache = {
    "last_update": None,
    "api_status": {},
    "agents": [],
    "cluster_status": {},
    "metrics": {},
    "trading": {},
    "events": deque(maxlen=100),
    "price_history": deque(maxlen=50),
    "trade_history": deque(maxlen=50),
    "api_health": True,
    "api_error": None
}

def fetch_api_data():
    """从Rust API获取数据"""
    global cache
    
    try:
        # 获取综合状态 - 注意这个端点可能较慢
        try:
            with urllib.request.urlopen(f"{API_BASE_URL}/api/status", timeout=30) as response:
                if response.status == 200:
                    cache["api_status"] = json.loads(response.read().decode('utf-8'))
        except Exception as e:
            print(f"获取status失败: {e}")
        
        # 获取代理列表 - 这个也可能较慢
        try:
            with urllib.request.urlopen(f"{API_BASE_URL}/api/agents", timeout=30) as response:
                if response.status == 200:
                    cache["agents"] = json.loads(response.read().decode('utf-8'))
        except Exception as e:
            print(f"获取agents失败: {e}")
        
        # 获取集群状态
        try:
            with urllib.request.urlopen(f"{API_BASE_URL}/api/cluster/status", timeout=30) as response:
                if response.status == 200:
                    cache["cluster_status"] = json.loads(response.read().decode('utf-8'))
        except Exception as e:
            print(f"获取cluster status失败: {e}")
        
        # 获取系统指标 - 这个通常很快
        try:
            with urllib.request.urlopen(f"{API_BASE_URL}/api/metrics", timeout=5) as response:
                if response.status == 200:
                    cache["metrics"] = json.loads(response.read().decode('utf-8'))
        except Exception as e:
            print(f"获取metrics失败: {e}")
        
        # 获取交易状态 - 这个也通常很快
        try:
            with urllib.request.urlopen(f"{API_BASE_URL}/api/trading", timeout=5) as response:
                if response.status == 200:
                    trading_data = json.loads(response.read().decode('utf-8'))
                    cache["trading"] = trading_data
                    
                    # 记录价格历史
                    if trading_data.get("last_price"):
                        for symbol, price in trading_data["last_price"].items():
                            cache["price_history"].append({
                                "time": datetime.now().strftime("%H:%M:%S"),
                                "symbol": symbol,
                                "price": price
                            })
                    
                    # 生成交易事件
                    if trading_data.get("total_trades", 0) > len(cache["trade_history"]):
                        cache["trade_history"].append({
                            "time": datetime.now().strftime("%H:%M:%S"),
                            "type": "TRADE",
                            "message": f"交易执行 (总计: {trading_data['total_trades']})"
                        })
        except Exception as e:
            print(f"获取trading失败: {e}")
        
        # 生成事件
        if cache["api_status"].get("trading_active"):
            cache["events"].append({
                "time": datetime.now().strftime("%H:%M:%S"),
                "type": "system",
                "message": "交易系统活跃"
            })
        
        cache["last_update"] = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        cache["api_health"] = True
        cache["api_error"] = None
        
    except (urllib.error.URLError, urllib.error.HTTPError) as e:
        cache["api_health"] = False
        cache["api_error"] = str(e)
        print(f"API请求错误: {e}")
    except Exception as e:
        cache["api_health"] = False
        cache["api_error"] = str(e)
        print(f"数据处理错误: {e}")

def update_data_loop():
    """后台线程定期更新数据"""
    while True:
        fetch_api_data()
        time.sleep(UPDATE_INTERVAL)

# HTML页面
HTML_CONTENT = """
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Aurelia API监控面板</title>
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
            max-width: 1400px;
            margin: 0 auto;
        }
        
        .header {
            background: rgba(255, 255, 255, 0.95);
            border-radius: 15px;
            padding: 25px;
            margin-bottom: 25px;
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.1);
        }
        
        h1 {
            color: #333;
            display: flex;
            align-items: center;
            gap: 10px;
        }
        
        .api-badge {
            display: inline-block;
            padding: 5px 15px;
            border-radius: 20px;
            font-size: 14px;
            font-weight: 500;
            margin-left: 20px;
            background: #667eea;
            color: white;
        }
        
        .api-healthy {
            background: #10b981;
            animation: pulse 2s infinite;
        }
        
        .api-error {
            background: #ef4444;
        }
        
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.7; }
        }
        
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 25px;
        }
        
        .card {
            background: white;
            border-radius: 15px;
            padding: 20px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        }
        
        .card.highlight {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        
        .card h3 {
            color: #666;
            font-size: 12px;
            text-transform: uppercase;
            margin-bottom: 10px;
            letter-spacing: 1px;
        }
        
        .card.highlight h3 {
            color: rgba(255, 255, 255, 0.9);
        }
        
        .metric {
            font-size: 32px;
            font-weight: bold;
            color: #333;
        }
        
        .metric.positive {
            color: #10b981;
        }
        
        .metric.negative {
            color: #ef4444;
        }
        
        .card.highlight .metric {
            color: white;
        }
        
        .metric-unit {
            font-size: 16px;
            color: #999;
        }
        
        .progress-bar {
            width: 100%;
            height: 6px;
            background: #e5e7eb;
            border-radius: 3px;
            overflow: hidden;
            margin-top: 10px;
        }
        
        .progress-fill {
            height: 100%;
            background: linear-gradient(90deg, #667eea, #764ba2);
            transition: width 0.3s ease;
        }
        
        .trading-section {
            background: white;
            border-radius: 15px;
            padding: 25px;
            margin-bottom: 25px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        }
        
        .price-ticker {
            display: flex;
            gap: 20px;
            padding: 15px;
            background: #f9fafb;
            border-radius: 10px;
            margin-bottom: 20px;
            overflow-x: auto;
        }
        
        .ticker-item {
            min-width: 150px;
            text-align: center;
        }
        
        .ticker-symbol {
            font-size: 12px;
            color: #666;
            margin-bottom: 5px;
        }
        
        .ticker-price {
            font-size: 24px;
            font-weight: bold;
            color: #333;
        }
        
        .agents-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
            gap: 15px;
        }
        
        .agent-card {
            background: #f9fafb;
            border-radius: 10px;
            padding: 15px;
            border: 2px solid transparent;
            transition: all 0.3s ease;
        }
        
        .agent-card.running {
            border-color: #10b981;
        }
        
        .agent-status {
            display: inline-block;
            padding: 3px 8px;
            border-radius: 12px;
            font-size: 12px;
            font-weight: 500;
            margin-left: 10px;
        }
        
        .status-running {
            background: #10b981;
            color: white;
        }
        
        .status-idle {
            background: #6b7280;
            color: white;
        }
        
        .event-list {
            max-height: 300px;
            overflow-y: auto;
        }
        
        .event-item {
            padding: 10px;
            border-left: 3px solid #e5e7eb;
            margin-bottom: 10px;
            background: #f9fafb;
            border-radius: 5px;
        }
        
        .event-system {
            border-left-color: #667eea;
        }
        
        .event-trade {
            border-left-color: #f59e0b;
        }
        
        .event-error {
            border-left-color: #ef4444;
        }
        
        .event-time {
            font-size: 12px;
            color: #999;
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
        
        .api-info {
            background: #f0f9ff;
            border: 1px solid #0ea5e9;
            border-radius: 8px;
            padding: 10px 15px;
            margin-bottom: 20px;
            color: #0369a1;
        }
        
        .error-banner {
            background: #fef2f2;
            border: 1px solid #ef4444;
            border-radius: 8px;
            padding: 10px 15px;
            margin-bottom: 20px;
            color: #991b1b;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>
                🚀 Aurelia API 监控面板
                <span id="api-badge" class="api-badge">连接中...</span>
                <button class="refresh-btn" onclick="refreshData()" style="margin-left: auto;">🔄 刷新</button>
            </h1>
            <p style="margin-top: 10px; color: #666;">
                数据源: Rust API (http://localhost:8080) | 更新间隔: 3秒 | 最后更新: <span id="last-update">-</span>
            </p>
        </div>
        
        <div id="error-banner" class="error-banner" style="display: none;">
            <strong>⚠️ API连接错误:</strong> <span id="error-message"></span>
        </div>
        
        <div class="api-info">
            <strong>📡 数据来源:</strong> 实时从Rust API获取 | 
            <strong>更新方式:</strong> 自动轮询 | 
            <strong>数据格式:</strong> JSON
        </div>
        
        <!-- 系统概览 -->
        <div class="grid">
            <div class="card highlight">
                <h3>🖥️ 系统状态</h3>
                <div class="metric" id="system-status">运行中</div>
                <div style="margin-top: 10px;">
                    代理数量: <span id="total-agents">0</span>
                </div>
            </div>
            
            <div class="card">
                <h3>💹 交易状态</h3>
                <div class="metric" id="trading-status">未激活</div>
                <div style="margin-top: 10px;">
                    总交易: <span id="total-trades">0</span> | 
                    盈亏: <span id="pnl">$0.00</span>
                </div>
            </div>
            
            <div class="card">
                <h3>📊 CPU 使用率</h3>
                <div class="metric">
                    <span id="cpu-usage">0</span><span class="metric-unit">%</span>
                </div>
                <div class="progress-bar">
                    <div id="cpu-progress" class="progress-fill" style="width: 0%"></div>
                </div>
            </div>
            
            <div class="card">
                <h3>🧠 内存使用率</h3>
                <div class="metric">
                    <span id="memory-usage">0</span><span class="metric-unit">%</span>
                </div>
                <div class="progress-bar">
                    <div id="memory-progress" class="progress-fill" style="width: 0%"></div>
                </div>
            </div>
        </div>
        
        <!-- 价格行情 -->
        <div class="trading-section">
            <h2 style="margin-bottom: 20px;">📈 市场行情</h2>
            <div class="price-ticker" id="price-ticker">
                <!-- 动态生成 -->
            </div>
        </div>
        
        <!-- 代理状态 -->
        <div class="trading-section">
            <h2 style="margin-bottom: 20px;">🤖 代理状态</h2>
            <div class="agents-grid" id="agents-grid">
                <!-- 动态生成 -->
            </div>
        </div>
        
        <!-- 集群信息 -->
        <div class="grid">
            <div class="card">
                <h3>🌐 集群健康度</h3>
                <div class="metric" id="cluster-health">Unknown</div>
                <div style="margin-top: 10px; font-size: 14px;">
                    健康: <span id="healthy-agents">0</span> | 
                    降级: <span id="degraded-agents">0</span> | 
                    离线: <span id="offline-agents">0</span>
                </div>
            </div>
            
            <div class="card">
                <h3>⚡ 集群CPU</h3>
                <div class="metric">
                    <span id="cluster-cpu">0</span><span class="metric-unit">%</span>
                </div>
            </div>
            
            <div class="card">
                <h3>💾 集群内存</h3>
                <div class="metric">
                    <span id="cluster-memory">0</span><span class="metric-unit">%</span>
                </div>
            </div>
        </div>
        
        <!-- 事件日志 -->
        <div class="trading-section">
            <h2 style="margin-bottom: 20px;">📝 系统事件</h2>
            <div class="event-list" id="event-list">
                <!-- 动态生成 -->
            </div>
        </div>
    </div>
    
    <script>
        let lastDataHash = '';
        
        function refreshData() {
            fetch('/api/data')
                .then(response => response.json())
                .then(data => {
                    // 检查数据是否变化
                    const dataHash = JSON.stringify(data);
                    if (dataHash === lastDataHash) {
                        return; // 数据未变化，跳过更新
                    }
                    lastDataHash = dataHash;
                    
                    // 更新API状态
                    const apiBadge = document.getElementById('api-badge');
                    const errorBanner = document.getElementById('error-banner');
                    
                    if (data.api_health) {
                        apiBadge.textContent = 'API正常';
                        apiBadge.className = 'api-badge api-healthy';
                        errorBanner.style.display = 'none';
                    } else {
                        apiBadge.textContent = 'API错误';
                        apiBadge.className = 'api-badge api-error';
                        errorBanner.style.display = 'block';
                        document.getElementById('error-message').textContent = data.api_error || '未知错误';
                    }
                    
                    // 更新最后更新时间
                    document.getElementById('last-update').textContent = data.last_update || '未知';
                    
                    // 更新系统状态
                    if (data.api_status) {
                        document.getElementById('total-agents').textContent = data.api_status.total_agents || 0;
                        
                        if (data.api_status.trading_active) {
                            document.getElementById('trading-status').textContent = '交易中';
                            document.getElementById('trading-status').style.color = '#10b981';
                        } else {
                            document.getElementById('trading-status').textContent = '未激活';
                            document.getElementById('trading-status').style.color = '#6b7280';
                        }
                    }
                    
                    // 更新系统指标
                    if (data.metrics) {
                        const cpuUsage = data.metrics.cpu_usage || 0;
                        const memUsage = data.metrics.memory_percentage || 0;
                        
                        document.getElementById('cpu-usage').textContent = cpuUsage.toFixed(1);
                        document.getElementById('cpu-progress').style.width = cpuUsage + '%';
                        
                        document.getElementById('memory-usage').textContent = memUsage.toFixed(1);
                        document.getElementById('memory-progress').style.width = memUsage + '%';
                    }
                    
                    // 更新交易信息
                    if (data.trading) {
                        document.getElementById('total-trades').textContent = data.trading.total_trades || 0;
                        const pnl = data.trading.pnl || 0;
                        const pnlElement = document.getElementById('pnl');
                        pnlElement.textContent = '$' + pnl.toFixed(2);
                        pnlElement.className = pnl >= 0 ? 'positive' : 'negative';
                        
                        // 更新价格行情
                        const ticker = document.getElementById('price-ticker');
                        ticker.innerHTML = '';
                        if (data.trading.last_price) {
                            for (const [symbol, price] of Object.entries(data.trading.last_price)) {
                                ticker.innerHTML += `
                                    <div class="ticker-item">
                                        <div class="ticker-symbol">${symbol}</div>
                                        <div class="ticker-price">$${price.toFixed(2)}</div>
                                    </div>
                                `;
                            }
                        } else {
                            ticker.innerHTML = '<p style="color: #999;">暂无价格数据</p>';
                        }
                    }
                    
                    // 更新代理状态
                    const agentsGrid = document.getElementById('agents-grid');
                    agentsGrid.innerHTML = '';
                    if (data.agents && data.agents.length > 0) {
                        data.agents.forEach(agent => {
                            const statusClass = agent.status === 'Running' ? 'running' : '';
                            const statusBadgeClass = agent.status === 'Running' ? 'status-running' : 'status-idle';
                            
                            agentsGrid.innerHTML += `
                                <div class="agent-card ${statusClass}">
                                    <div>
                                        <strong>${agent.agent_id}</strong>
                                        <span class="agent-status ${statusBadgeClass}">${agent.status}</span>
                                    </div>
                                    <div style="margin-top: 10px; font-size: 14px; color: #666;">
                                        <div>主机: ${agent.hostname}</div>
                                        <div>IP: ${agent.ip_address}</div>
                                        <div>CPU: ${agent.cpu_usage.toFixed(1)}%</div>
                                        <div>内存: ${agent.memory_usage.toFixed(1)}%</div>
                                    </div>
                                </div>
                            `;
                        });
                    } else {
                        agentsGrid.innerHTML = '<p style="color: #999;">暂无代理数据</p>';
                    }
                    
                    // 更新集群状态
                    if (data.cluster_status) {
                        document.getElementById('cluster-health').textContent = data.cluster_status.cluster_health || 'Unknown';
                        document.getElementById('healthy-agents').textContent = data.cluster_status.healthy_agents || 0;
                        document.getElementById('degraded-agents').textContent = data.cluster_status.degraded_agents || 0;
                        document.getElementById('offline-agents').textContent = data.cluster_status.offline_agents || 0;
                        document.getElementById('cluster-cpu').textContent = (data.cluster_status.total_cpu_usage || 0).toFixed(1);
                        document.getElementById('cluster-memory').textContent = (data.cluster_status.total_memory_usage || 0).toFixed(1);
                    }
                    
                    // 更新事件列表
                    const eventList = document.getElementById('event-list');
                    eventList.innerHTML = '';
                    
                    // 添加交易历史作为事件
                    if (data.trade_history && data.trade_history.length > 0) {
                        const recentTrades = [...data.trade_history].reverse().slice(0, 5);
                        recentTrades.forEach(trade => {
                            eventList.innerHTML += `
                                <div class="event-item event-trade">
                                    <div style="display: flex; justify-content: space-between;">
                                        <strong>${trade.message}</strong>
                                        <span class="event-time">${trade.time}</span>
                                    </div>
                                </div>
                            `;
                        });
                    }
                    
                    // 添加系统事件
                    if (data.events && data.events.length > 0) {
                        const recentEvents = [...data.events].reverse().slice(0, 5);
                        recentEvents.forEach(event => {
                            eventList.innerHTML += `
                                <div class="event-item event-${event.type}">
                                    <div style="display: flex; justify-content: space-between;">
                                        <strong>${event.message}</strong>
                                        <span class="event-time">${event.time}</span>
                                    </div>
                                </div>
                            `;
                        });
                    }
                    
                    if (eventList.innerHTML === '') {
                        eventList.innerHTML = '<p style="color: #999; text-align: center;">暂无事件</p>';
                    }
                })
                .catch(error => {
                    console.error('Error fetching data:', error);
                    document.getElementById('api-badge').textContent = '连接失败';
                    document.getElementById('api-badge').className = 'api-badge api-error';
                });
        }
        
        // 自动刷新
        setInterval(refreshData, 3000);
        
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
        elif self.path == '/api/data':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            # 转换deque为list以便JSON序列化
            data_copy = json.loads(json.dumps(cache, default=list))
            self.wfile.write(json.dumps(data_copy).encode('utf-8'))
        else:
            self.send_error(404, "File not found")
    
    def log_message(self, format, *args):
        # 禁用日志输出
        pass

def main():
    print(f"""
╔══════════════════════════════════════════╗
║     🚀 Aurelia API 监控面板              ║
╚══════════════════════════════════════════╝

📡 数据源: Rust API (http://localhost:8080)
🌐 监控面板: http://localhost:{WEB_PORT}
🔄 更新间隔: {UPDATE_INTERVAL}秒

功能特性:
✅ 实时从Rust API获取数据
✅ 系统指标监控
✅ 交易状态追踪
✅ 代理管理
✅ 集群健康度

正在启动服务器...
""")
    
    # 启动后台数据更新线程
    update_thread = threading.Thread(target=update_data_loop, daemon=True)
    update_thread.start()
    
    # 启动HTTP服务器
    with socketserver.TCPServer(("", WEB_PORT), MonitoringHandler) as httpd:
        print(f"✅ 服务器已启动在端口 {WEB_PORT}")
        print(f"\n🌐 请在浏览器中打开: http://localhost:{WEB_PORT}")
        print("\n按 Ctrl+C 停止服务器")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n\n👋 服务器已停止")

if __name__ == "__main__":
    main()