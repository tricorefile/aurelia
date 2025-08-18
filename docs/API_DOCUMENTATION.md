# 📚 Aurelia Rust API 文档

## 概述

Aurelia 系统当前实现的 API 和接口主要分为以下几类：

1. **内部事件系统** - 基于 Tokio broadcast channel
2. **监控 HTTP API** - REST 风格的监控接口（设计但未完全实现）
3. **WebSocket 接口** - 实时数据推送（设计但未完全实现）
4. **外部集成** - Binance WebSocket 客户端

---

## 🔄 内部事件系统 (Event Bus)

### 核心事件类型 (`common/src/lib.rs`)

```rust
pub enum AppEvent {
    SystemVitals(SystemVitals),          // 系统资源状态
    MarketData(MarketData),              // 市场数据
    StrategyDecision(StrategyDecision),  // 策略决策
    ReloadConfig,                        // 重载配置
    SystemStateChange(SystemState),      // 系统状态变更
    FinancialUpdate(f64),               // 财务更新
    WebSearchQuery(String),             // Web搜索查询
    WebSearchResponse(Vec<String>),     // Web搜索响应
    LlmQuery(String),                   // LLM查询
    LlmResponse(String),                 // LLM响应
    ModuleReadyForHotSwap(String),      // 模块热更新就绪
    Deploy(DeploymentInfo),             // 部署命令
}
```

### 数据结构

```rust
// 策略决策
pub enum StrategyDecision {
    Buy(String, f64),   // 买入(Symbol, Price)
    Sell(String, f64),  // 卖出(Symbol, Price)
    Hold(String),       // 持有(Symbol)
}

// 市场数据
pub struct MarketData {
    pub symbol: String,     // 交易对
    pub price: f64,        // 价格
    pub quantity: f64,     // 数量
    pub timestamp: u64,    // 时间戳
}

// 系统资源
pub struct SystemVitals {
    pub cpu_usage: f32,        // CPU使用率
    pub mem_usage_mb: f64,     // 内存使用(MB)
    pub mem_total_mb: f64,     // 总内存(MB)
}

// 部署信息
pub struct DeploymentInfo {
    pub ip: String,            // 目标IP
    pub remote_user: String,   // 远程用户
    pub private_key_path: String,  // SSH密钥路径
    pub remote_path: String,   // 远程路径
}
```

---

## 🌐 监控服务 API (设计阶段)

### HTTP REST API 端点

监控服务设计了以下端点（在 `monitoring_service/src/lib.rs` 中定义但未完全实现）：

| 方法 | 端点 | 描述 | 请求体 | 响应 |
|------|------|------|--------|------|
| GET | `/api/agents` | 获取所有代理列表 | - | `Vec<AgentStatus>` |
| GET | `/api/agents/:id` | 获取特定代理信息 | - | `AgentStatus` |
| POST | `/api/agents/:id/metrics` | 更新代理指标 | `MetricsUpdate` | `ApiResponse<String>` |
| GET | `/api/cluster/status` | 获取集群状态 | - | `ClusterStatus` |
| GET | `/api/cluster/events` | 获取事件列表 | Query: `limit`, `severity`, `agent_id` | `Vec<ClusterEvent>` |
| GET | `/api/cluster/health` | 获取集群健康度 | - | `HealthResponse` |

### 请求/响应结构

```rust
// 指标更新请求
pub struct MetricsUpdate {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub replicas_active: Vec<String>,
}

// 代理状态
pub struct AgentStatus {
    pub agent_id: String,
    pub hostname: String,
    pub ip_address: String,
    pub status: AgentState,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub uptime_seconds: u64,
    pub last_heartbeat: DateTime<Utc>,
    pub version: String,
    pub role: String,
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub replicas_active: Vec<String>,
}

// 集群状态
pub struct ClusterStatus {
    pub total_agents: usize,
    pub healthy_agents: usize,
    pub degraded_agents: usize,
    pub offline_agents: usize,
    pub total_cpu_usage: f32,
    pub total_memory_usage: f32,
    pub total_tasks_completed: u32,
    pub total_tasks_failed: u32,
    pub cluster_health: ClusterHealth,
    pub agents: Vec<AgentStatus>,
    pub events: Vec<ClusterEvent>,
}

// API响应包装
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}
```

### WebSocket 端点

| 端点 | 描述 | 消息类型 |
|------|------|----------|
| `/ws` | WebSocket实时更新 | `initial`, `update`, `events` |

WebSocket 消息格式：
```json
{
    "type": "update",
    "timestamp": "2024-01-01T12:00:00Z",
    "data": { /* ClusterStatus */ }
}
```

---

## 🔌 外部集成

### 1. Binance WebSocket 客户端

位置：`perception_core/src/lib.rs`

```rust
const BINANCE_WS_API: &str = "wss://stream.binance.com:9443/ws/btcusdt@trade";

// 连接并订阅市场数据
pub async fn run(tx: EventSender) {
    let (ws_stream, _) = connect_async(BINANCE_WS_API).await;
    // 处理接收的市场数据并转发为 MarketData 事件
}
```

### 2. Binance REST API (执行引擎)

位置：`execution_engine/src/lib.rs`

```rust
// 下单接口（内部使用）
async fn execute_order(
    symbol: String,
    side: String,    // "BUY" 或 "SELL"
    quantity: f64,
    price: f64
) -> Result<String, Box<dyn std::error::Error>>
```

---

## 🎯 公开的 Trait 和接口

### 1. Deployer Trait

```rust
pub trait Deployer: Send + Sync {
    fn deploy(&self, info: DeploymentInfo) -> Result<(), Box<dyn std::error::Error>>;
}
```

用于实现不同的部署策略（SSH、Docker等）。

### 2. 模块生命周期接口

每个核心模块都实现了标准的生命周期：

```rust
impl Module {
    pub fn new(tx: EventSender, rx: EventReceiver) -> Self
    pub async fn run(&mut self)
}
```

---

## 📡 实际可用的 API

### 当前实际运行的服务

1. **简化监控服务** (`monitoring_service/src/simple_server.rs`)
   - 内部状态收集
   - 60秒日志输出
   - 无HTTP端点

2. **Python监控面板** (非Rust实现)
   - `http://localhost:3030/` - Web界面
   - `http://localhost:3030/api/status` - JSON状态

---

## 🚧 未来计划的 API

基于代码结构，以下API已设计但尚未实现：

1. **任务管理 API**
   - 创建任务
   - 查询任务状态
   - 取消任务

2. **策略配置 API**
   - 更新策略参数
   - 切换策略类型
   - 回测接口

3. **部署管理 API**
   - 触发远程部署
   - 查询部署状态
   - 管理节点

4. **自主决策 API**
   - 查询决策历史
   - 手动触发决策
   - 调整决策参数

---

## 📝 使用示例

### 1. 内部事件发送

```rust
use common::{AppEvent, MarketData};

// 发送市场数据
let event = AppEvent::MarketData(MarketData {
    symbol: "BTCUSDT".to_string(),
    price: 50000.0,
    quantity: 0.1,
    timestamp: 1234567890,
});
tx.send(event).unwrap();
```

### 2. 模块间通信

```rust
// 在 ExecutionEngine 中监听策略决策
match self.rx.recv().await {
    Ok(AppEvent::StrategyDecision(decision)) => {
        match decision {
            StrategyDecision::Buy(symbol, price) => {
                // 执行买入
            },
            StrategyDecision::Sell(symbol, price) => {
                // 执行卖出
            },
            _ => {}
        }
    }
    _ => {}
}
```

---

## 🔒 安全性说明

1. **无外部HTTP API暴露** - 当前Rust代码未暴露任何HTTP端点
2. **事件系统内部隔离** - 使用broadcast channel进行模块间通信
3. **API密钥管理** - 通过.env文件管理，不硬编码
4. **SSH密钥认证** - 部署使用密钥而非密码

---

## 📊 监控集成

虽然Rust代码设计了完整的监控API，但当前通过Python脚本实现：

- 解析日志文件提取数据
- 提供简单的HTTP服务
- 展示Web界面

要启用Rust原生监控API，需要：
1. 完成axum路由实现
2. 启动HTTP服务器
3. 实现WebSocket处理器
4. 连接数据收集器

---

## 总结

当前Aurelia系统的Rust实现主要提供：
- **内部事件总线** - 完整实现并运行
- **外部市场数据集成** - Binance WebSocket客户端
- **模块化架构** - 标准化的模块接口

设计但未完全实现：
- HTTP REST API
- WebSocket实时推送
- RPC接口

实际监控通过Python辅助脚本提供Web界面。