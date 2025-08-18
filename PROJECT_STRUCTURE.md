# Aurelia 项目结构

## 目录结构

```
aurelia/
├── autonomy_core/          # 自治核心模块
│   ├── src/
│   │   ├── autonomous_agent.rs    # 自主代理
│   │   ├── decision_maker.rs      # 决策引擎
│   │   ├── deployment_commander.rs # 部署指挥官
│   │   ├── health_monitor.rs      # 健康监控
│   │   ├── recovery_manager.rs    # 恢复管理
│   │   ├── self_replicator.rs     # 自我复制
│   │   ├── server_config.rs       # 服务器配置
│   │   ├── ssh_deployer.rs        # SSH部署器
│   │   └── task_scheduler.rs      # 任务调度
│   └── Cargo.toml
│
├── common/                  # 公共库
│   ├── src/
│   │   └── lib.rs         # 共享类型和工具
│   └── Cargo.toml
│
├── config/                  # 配置文件
│   └── target_servers.json # 目标服务器配置
│
├── deployment_tester/       # 部署测试器
│   ├── src/
│   │   ├── config.rs      # 配置管理
│   │   ├── deployer.rs    # 部署逻辑
│   │   ├── monitor.rs     # 监控功能
│   │   └── validator.rs   # 验证器
│   └── Cargo.toml
│
├── docs/                    # 文档
│   ├── API_DOCUMENTATION.md
│   ├── DEPLOYMENT_GUIDE.md
│   ├── MONITORING_COMPARISON.md
│   ├── PASSWORD_AUTH_GUIDE.md
│   ├── SERVER_CONFIG_GUIDE.md
│   └── SYSTEM_STATUS.md
│
├── execution_engine/        # 执行引擎
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
│
├── kernel/                  # 核心内核
│   ├── src/
│   │   └── main.rs        # 主程序入口
│   ├── examples/
│   │   └── test_ssh_deploy.rs # SSH部署测试
│   └── Cargo.toml
│
├── metamorphosis_engine/    # 变形引擎
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
│
├── monitoring_service/      # 监控服务
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
│
├── perception_core/         # 感知核心
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
│
├── py/                      # Python脚本
│   ├── api_monitor.py      # API监控面板
│   ├── enhanced_monitor.py # 增强监控
│   ├── server_manager.py   # 服务器管理
│   ├── test_deployment.py  # 部署测试
│   ├── test_ssh_connection.py # SSH测试
│   └── README.md
│
├── reasoning_engine/        # 推理引擎
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
│
├── resource_monitor/        # 资源监控
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
│
├── scripts/                 # Shell脚本
│   ├── deploy.sh           # 部署脚本
│   ├── remote_exec.sh      # 远程执行
│   ├── start_with_monitor.sh # 启动监控
│   └── test_pure_rust_deploy.sh # Rust部署测试
│
├── strategy_engine/         # 策略引擎
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
│
├── survival_protocol/       # 生存协议
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
│
├── tests/                   # 测试文件
│   ├── test_autonomous.rs
│   ├── test_deployment.rs
│   └── test_rust_config.rs
│
├── Cargo.toml              # 工作空间配置
├── README.md               # 项目说明
├── SSH_TESTING_GUIDE.md    # SSH测试指南
└── PROJECT_STRUCTURE.md   # 本文件
```

## 核心模块说明

### 1. autonomy_core - 自治核心
- **自主决策**: 根据市场和系统状态做出决策
- **自我部署**: 纯Rust实现的SSH部署能力
- **健康监控**: 实时监控系统健康状态
- **自我复制**: 自动复制到新服务器
- **故障恢复**: 自动检测和恢复故障

### 2. kernel - 系统内核
- 主程序入口
- 协调所有模块
- 事件循环和消息传递
- API服务

### 3. 策略模块
- **perception_core**: 市场数据感知
- **reasoning_engine**: 交易逻辑推理
- **strategy_engine**: 策略执行
- **execution_engine**: 订单执行

### 4. 支持模块
- **monitoring_service**: 监控服务
- **resource_monitor**: 资源监控
- **survival_protocol**: 生存协议
- **metamorphosis_engine**: 系统进化

## 关键文件

### 配置文件
- `config/target_servers.json`: 目标服务器配置
- `Cargo.toml`: Rust项目配置
- `.env`: 环境变量（API密钥等）

### 脚本工具
- `scripts/deploy.sh`: 部署到远程服务器
- `scripts/remote_exec.sh`: 批量远程执行
- `py/server_manager.py`: 服务器配置管理
- `py/api_monitor.py`: Web监控界面

### 文档
- `README.md`: 项目总览
- `SSH_TESTING_GUIDE.md`: SSH连接测试
- `docs/DEPLOYMENT_GUIDE.md`: 部署指南
- `docs/API_DOCUMENTATION.md`: API文档

## 构建和运行

### 构建
```bash
# Debug版本
cargo build

# Release版本
cargo build --release
```

### 运行
```bash
# 运行kernel
cargo run --bin kernel

# 或运行编译后的版本
./target/release/kernel
```

### 测试
```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --package autonomy_core

# 运行SSH部署测试
cargo run --example test_ssh_deploy
```

### 部署
```bash
# 使用脚本部署
./scripts/deploy.sh deploy 194.146.13.14 -P 'password'

# 使用Python管理服务器
python3 py/server_manager.py list
python3 py/server_manager.py add server-1 "Server 1" 192.168.1.100
```

## 开发流程

1. **修改代码**: 在相应模块的 `src/` 目录
2. **测试**: 运行 `cargo test`
3. **构建**: `cargo build --release`
4. **部署**: 使用部署脚本或内置部署功能
5. **监控**: 使用 `py/api_monitor.py` 监控运行状态

## 特色功能

### 纯Rust SSH部署
- 无需外部依赖（sshpass等）
- 支持密码和密钥认证
- 自动配置systemd服务
- 完整的部署生命周期管理

### 自治能力
- 自主决策交易
- 自动故障恢复
- 自我复制扩展
- 动态资源管理

### 监控系统
- 实时API监控
- Web界面展示
- 多服务器管理
- 健康状态追踪