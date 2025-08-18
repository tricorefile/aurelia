#!/usr/bin/env python3
"""
Aurelia 增强版监控面板 - 包含交易监控
"""

import http.server
import socketserver
import json
import subprocess
import os
import time
from datetime import datetime, timedelta
import threading
import re
from collections import deque, defaultdict

PORT = 3030
LOG_FILE = "aurelia_output.log"

# 全局数据存储
monitoring_data = {
    "agents": {},
    "events": deque(maxlen=100),
    "cluster_status": {
        "total_agents": 0,
        "healthy_agents": 0,
        "cpu_usage": 0,
        "memory_usage": 0,
        "last_update": None
    },
    "trading": {
        "market_data": deque(maxlen=50),
        "strategy_decisions": deque(maxlen=50),
        "positions": {},
        "orders": deque(maxlen=100),
        "pnl": 0.0,
        "total_trades": 0,
        "successful_trades": 0,
        "failed_trades": 0,
        "last_price": {},
        "volume_24h": 0,
        "trading_active": False
    },
    "performance": {
        "trade_history": deque(maxlen=1000),
        "win_rate": 0.0,
        "avg_profit": 0.0,
        "max_drawdown": 0.0,
        "sharpe_ratio": 0.0,
        "daily_pnl": defaultdict(float)
    }
}

def parse_log_file():
    """解析日志文件获取监控数据"""
    global monitoring_data
    
    if not os.path.exists(LOG_FILE):
        return
    
    try:
        with open(LOG_FILE, 'r') as f:
            lines = f.readlines()[-2000:]  # 读取最后2000行
            
        for line in lines:
            # 解析市场数据
            if "MarketData" in line:
                match = re.search(r'symbol[:\s]*"?([A-Z]+)"?.*price[:\s]*([0-9.]+)', line, re.IGNORECASE)
                if match:
                    symbol = match.group(1)
                    price = float(match.group(2))
                    monitoring_data["trading"]["last_price"][symbol] = price
                    monitoring_data["trading"]["market_data"].append({
                        "time": datetime.now().strftime("%H:%M:%S"),
                        "symbol": symbol,
                        "price": price
                    })
                    monitoring_data["trading"]["trading_active"] = True
            
            # 解析策略决策
            if "StrategyDecision" in line or "Buy" in line or "Sell" in line:
                timestamp = datetime.now().strftime("%H:%M:%S")
                decision_type = "BUY" if "Buy" in line else "SELL" if "Sell" in line else "HOLD"
                
                # 尝试提取价格和数量
                price_match = re.search(r'([0-9]+\.?[0-9]*)', line)
                price = float(price_match.group(1)) if price_match else 0
                
                monitoring_data["trading"]["strategy_decisions"].append({
                    "time": timestamp,
                    "type": decision_type,
                    "price": price,
                    "symbol": "BTCUSDT"  # 默认交易对
                })
                
                monitoring_data["trading"]["total_trades"] += 1
                
                # 创建交易事件
                monitoring_data["events"].append({
                    "time": timestamp,
                    "type": "trade",
                    "message": f"{decision_type} signal at ${price:.2f}"
                })
            
            # 解析执行引擎活动
            if "Execution Engine" in line:
                monitoring_data["trading"]["trading_active"] = True
                if "order" in line.lower():
                    monitoring_data["trading"]["orders"].append({
                        "time": datetime.now().strftime("%H:%M:%S"),
                        "message": "Order executed"
                    })
            
            # 解析WebSocket连接（Binance）
            if "Binance WebSocket" in line or "WebSocket connection" in line:
                monitoring_data["trading"]["trading_active"] = True
                monitoring_data["events"].append({
                    "time": datetime.now().strftime("%H:%M:%S"),
                    "type": "connection",
                    "message": "Connected to Binance WebSocket"
                })
            
            # 解析监控日志
            if "Monitoring" in line and "agents" in line:
                match = re.search(r'Monitoring (\d+) agents', line)
                if match:
                    monitoring_data["cluster_status"]["total_agents"] = int(match.group(1))
                    monitoring_data["cluster_status"]["healthy_agents"] = int(match.group(1))
            
            # 解析决策日志
            if "decision" in line.lower():
                timestamp = line.split('[0m')[0].split('Z')[0].split('T')[1] if 'Z[0m' in line else datetime.now().strftime("%H:%M:%S")
                monitoring_data["events"].append({
                    "time": timestamp,
                    "type": "decision",
                    "message": "Autonomous decision made"
                })
            
            # 解析健康检查
            if "health" in line.lower():
                timestamp = line.split('[0m')[0].split('Z')[0].split('T')[1] if 'Z[0m' in line else datetime.now().strftime("%H:%M:%S")
                monitoring_data["events"].append({
                    "time": timestamp,
                    "type": "health",
                    "message": "Health check performed"
                })
        
        # 计算交易统计
        if monitoring_data["trading"]["total_trades"] > 0:
            monitoring_data["performance"]["win_rate"] = (
                monitoring_data["trading"]["successful_trades"] / 
                monitoring_data["trading"]["total_trades"] * 100
            )
        
        # 获取进程信息
        try:
            result = subprocess.run(['pgrep', '-f', 'target/release/kernel'], capture_output=True, text=True)
            if result.stdout.strip():
                pid = result.stdout.strip().split('\n')[0]
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
        time.sleep(3)

# HTML页面
HTML_CONTENT = """
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Aurelia 交易监控面板</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0f0f23;
            color: #e0e0e0;
            min-height: 100vh;
            padding: 20px;
        }
        
        .container {
            max-width: 1400px;
            margin: 0 auto;
        }
        
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            border-radius: 15px;
            padding: 25px;
            margin-bottom: 25px;
            box-shadow: 0 10px 30px rgba(102, 126, 234, 0.3);
        }
        
        h1 {
            color: white;
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
        
        .trading-active {
            background: #22c55e;
            animation: pulse 2s infinite;
        }
        
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.7; }
        }
        
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 20px;
            margin-bottom: 25px;
        }
        
        .card {
            background: #1a1a2e;
            border: 1px solid #16213e;
            border-radius: 15px;
            padding: 20px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
        }
        
        .card.highlight {
            border-color: #667eea;
            box-shadow: 0 4px 20px rgba(102, 126, 234, 0.3);
        }
        
        .card h3 {
            color: #9ca3af;
            font-size: 12px;
            text-transform: uppercase;
            margin-bottom: 10px;
            letter-spacing: 1px;
        }
        
        .metric {
            font-size: 32px;
            font-weight: bold;
            color: #fff;
        }
        
        .metric.positive {
            color: #22c55e;
        }
        
        .metric.negative {
            color: #ef4444;
        }
        
        .metric-unit {
            font-size: 16px;
            color: #9ca3af;
        }
        
        .progress-bar {
            width: 100%;
            height: 6px;
            background: #374151;
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
            background: #1a1a2e;
            border: 1px solid #16213e;
            border-radius: 15px;
            padding: 25px;
            margin-bottom: 25px;
        }
        
        .market-ticker {
            display: flex;
            gap: 20px;
            padding: 15px;
            background: #0f0f23;
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
            color: #9ca3af;
            margin-bottom: 5px;
        }
        
        .ticker-price {
            font-size: 20px;
            font-weight: bold;
            color: #fff;
        }
        
        .trade-list {
            max-height: 300px;
            overflow-y: auto;
        }
        
        .trade-item {
            display: flex;
            justify-content: space-between;
            padding: 10px;
            border-bottom: 1px solid #374151;
        }
        
        .trade-item.buy {
            border-left: 3px solid #22c55e;
        }
        
        .trade-item.sell {
            border-left: 3px solid #ef4444;
        }
        
        .event-item {
            padding: 10px 15px;
            border-left: 3px solid #374151;
            margin-bottom: 10px;
            background: #0f0f23;
            border-radius: 5px;
        }
        
        .event-health {
            border-left-color: #10b981;
        }
        
        .event-decision {
            border-left-color: #667eea;
        }
        
        .event-trade {
            border-left-color: #f59e0b;
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
        
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-top: 20px;
        }
        
        .stat-item {
            background: #0f0f23;
            padding: 15px;
            border-radius: 10px;
            text-align: center;
        }
        
        .stat-label {
            font-size: 12px;
            color: #9ca3af;
            margin-bottom: 5px;
        }
        
        .stat-value {
            font-size: 24px;
            font-weight: bold;
            color: #fff;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>
                📈 Aurelia 智能交易监控
                <span id="status-badge" class="status-badge">加载中...</span>
                <span id="trading-badge" class="status-badge" style="display:none;">交易中</span>
                <button class="refresh-btn" onclick="refreshData()" style="margin-left: auto;">🔄 刷新</button>
            </h1>
            <p style="margin-top: 10px; color: rgba(255,255,255,0.9);">
                最后更新: <span id="last-update">-</span>
            </p>
        </div>
        
        <!-- 交易概览 -->
        <div class="grid">
            <div class="card highlight">
                <h3>📊 交易状态</h3>
                <div class="metric" id="trading-status">未激活</div>
                <div style="margin-top: 10px; font-size: 14px;">
                    <span id="market-connection">⚪ 未连接市场</span>
                </div>
            </div>
            
            <div class="card">
                <h3>💰 今日盈亏</h3>
                <div class="metric" id="daily-pnl">$0.00</div>
                <div style="margin-top: 5px; font-size: 14px; color: #9ca3af;">
                    总交易: <span id="total-trades">0</span>
                </div>
            </div>
            
            <div class="card">
                <h3>📈 胜率</h3>
                <div class="metric" id="win-rate">0%</div>
                <div class="progress-bar">
                    <div id="win-rate-bar" class="progress-fill" style="width: 0%"></div>
                </div>
            </div>
            
            <div class="card">
                <h3>🎯 策略决策</h3>
                <div class="metric" id="last-decision">等待中</div>
                <div style="margin-top: 5px; font-size: 12px; color: #9ca3af;">
                    <span id="decision-time">-</span>
                </div>
            </div>
        </div>
        
        <!-- 市场数据 -->
        <div class="trading-section">
            <h2 style="margin-bottom: 20px;">📈 市场数据</h2>
            <div class="market-ticker" id="market-ticker">
                <!-- 动态生成 -->
            </div>
            
            <div class="stats-grid">
                <div class="stat-item">
                    <div class="stat-label">24h成交量</div>
                    <div class="stat-value" id="volume-24h">$0</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">持仓数量</div>
                    <div class="stat-value" id="positions">0</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">挂单数量</div>
                    <div class="stat-value" id="open-orders">0</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">最大回撤</div>
                    <div class="stat-value" id="max-drawdown">0%</div>
                </div>
            </div>
        </div>
        
        <!-- 最近交易 -->
        <div class="trading-section">
            <h2 style="margin-bottom: 20px;">💹 最近交易信号</h2>
            <div class="trade-list" id="trade-list">
                <!-- 动态生成 -->
            </div>
        </div>
        
        <!-- 系统监控 -->
        <div class="grid">
            <div class="card">
                <h3>💻 CPU 使用率</h3>
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
            
            <div class="card">
                <h3>🤖 活跃智能体</h3>
                <div class="metric" id="total-agents">0</div>
            </div>
        </div>
        
        <!-- 事件日志 -->
        <div class="trading-section">
            <h2 style="margin-bottom: 20px;">📝 系统事件</h2>
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
                    document.getElementById('status-badge').textContent = hasAgents ? '系统运行中' : '系统已停止';
                    
                    // 更新交易状态
                    const tradingActive = data.trading.trading_active;
                    const tradingBadge = document.getElementById('trading-badge');
                    if (tradingActive) {
                        tradingBadge.style.display = 'inline-block';
                        tradingBadge.className = 'status-badge trading-active';
                        document.getElementById('trading-status').textContent = '交易激活';
                        document.getElementById('trading-status').style.color = '#22c55e';
                        document.getElementById('market-connection').textContent = '🟢 已连接市场';
                    } else {
                        tradingBadge.style.display = 'none';
                        document.getElementById('trading-status').textContent = '未激活';
                        document.getElementById('trading-status').style.color = '#9ca3af';
                        document.getElementById('market-connection').textContent = '⚪ 未连接市场';
                    }
                    
                    // 更新交易指标
                    const pnl = data.trading.pnl || 0;
                    const pnlElement = document.getElementById('daily-pnl');
                    pnlElement.textContent = '$' + pnl.toFixed(2);
                    pnlElement.className = 'metric ' + (pnl >= 0 ? 'positive' : 'negative');
                    
                    document.getElementById('total-trades').textContent = data.trading.total_trades;
                    
                    const winRate = data.performance.win_rate || 0;
                    document.getElementById('win-rate').textContent = winRate.toFixed(1) + '%';
                    document.getElementById('win-rate-bar').style.width = winRate + '%';
                    
                    // 更新最后决策
                    if (data.trading.strategy_decisions && data.trading.strategy_decisions.length > 0) {
                        const lastDecision = data.trading.strategy_decisions[data.trading.strategy_decisions.length - 1];
                        document.getElementById('last-decision').textContent = lastDecision.type;
                        document.getElementById('decision-time').textContent = lastDecision.time;
                        
                        const decisionElement = document.getElementById('last-decision');
                        if (lastDecision.type === 'BUY') {
                            decisionElement.style.color = '#22c55e';
                        } else if (lastDecision.type === 'SELL') {
                            decisionElement.style.color = '#ef4444';
                        } else {
                            decisionElement.style.color = '#9ca3af';
                        }
                    }
                    
                    // 更新市场数据
                    const ticker = document.getElementById('market-ticker');
                    ticker.innerHTML = '';
                    for (const [symbol, price] of Object.entries(data.trading.last_price || {})) {
                        ticker.innerHTML += `
                            <div class="ticker-item">
                                <div class="ticker-symbol">${symbol}</div>
                                <div class="ticker-price">$${price.toFixed(2)}</div>
                            </div>
                        `;
                    }
                    
                    // 更新交易列表
                    const tradeList = document.getElementById('trade-list');
                    tradeList.innerHTML = '';
                    if (data.trading.strategy_decisions && data.trading.strategy_decisions.length > 0) {
                        const recentTrades = [...data.trading.strategy_decisions].reverse().slice(0, 10);
                        for (const trade of recentTrades) {
                            const tradeClass = trade.type.toLowerCase() === 'buy' ? 'buy' : trade.type.toLowerCase() === 'sell' ? 'sell' : '';
                            tradeList.innerHTML += `
                                <div class="trade-item ${tradeClass}">
                                    <div>
                                        <strong>${trade.type}</strong>
                                        <span style="margin-left: 10px;">$${trade.price.toFixed(2)}</span>
                                    </div>
                                    <span class="event-time">${trade.time}</span>
                                </div>
                            `;
                        }
                    } else {
                        tradeList.innerHTML = '<p style="color: #9ca3af; text-align: center;">暂无交易记录</p>';
                    }
                    
                    // 更新其他统计
                    document.getElementById('volume-24h').textContent = '$' + (data.trading.volume_24h || 0).toFixed(0);
                    document.getElementById('positions').textContent = Object.keys(data.trading.positions || {}).length;
                    document.getElementById('open-orders').textContent = (data.trading.orders || []).length;
                    document.getElementById('max-drawdown').textContent = (data.performance.max_drawdown || 0).toFixed(1) + '%';
                    
                    // 更新系统指标
                    document.getElementById('total-agents').textContent = data.cluster_status.total_agents;
                    document.getElementById('cpu-usage').textContent = data.cluster_status.cpu_usage.toFixed(1);
                    document.getElementById('memory-usage').textContent = data.cluster_status.memory_usage.toFixed(1);
                    document.getElementById('cpu-progress').style.width = data.cluster_status.cpu_usage + '%';
                    document.getElementById('memory-progress').style.width = data.cluster_status.memory_usage + '%';
                    document.getElementById('last-update').textContent = data.cluster_status.last_update || '未知';
                    
                    // 更新事件列表
                    const eventsList = document.getElementById('events-list');
                    eventsList.innerHTML = '';
                    
                    if (data.events && data.events.length > 0) {
                        const reversedEvents = [...data.events].reverse();
                        for (const event of reversedEvents.slice(0, 20)) {
                            let eventClass = 'event-item';
                            if (event.type === 'health') eventClass += ' event-health';
                            else if (event.type === 'decision') eventClass += ' event-decision';
                            else if (event.type === 'trade') eventClass += ' event-trade';
                            
                            eventsList.innerHTML += `
                                <div class="${eventClass}">
                                    <div style="display: flex; justify-content: space-between;">
                                        <strong>${event.message}</strong>
                                        <span class="event-time">${event.time}</span>
                                    </div>
                                </div>
                            `;
                        }
                    } else {
                        eventsList.innerHTML = '<p style="color: #9ca3af; text-align: center;">暂无事件</p>';
                    }
                })
                .catch(error => {
                    console.error('Error fetching data:', error);
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
        elif self.path == '/api/status':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            # 转换deque为list以便JSON序列化
            data_copy = json.loads(json.dumps(monitoring_data, default=list))
            self.wfile.write(json.dumps(data_copy).encode('utf-8'))
        else:
            self.send_error(404, "File not found")
    
    def log_message(self, format, *args):
        # 禁用日志输出
        pass

def main():
    print(f"""
╔══════════════════════════════════════════╗
║     📈 Aurelia 交易监控面板              ║
╚══════════════════════════════════════════╝

📊 监控面板地址: http://localhost:{PORT}
📡 API 端点: http://localhost:{PORT}/api/status

功能特性:
✅ 实时交易状态监控
✅ 市场数据追踪
✅ 策略决策记录
✅ 盈亏统计分析
✅ 系统性能监控

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