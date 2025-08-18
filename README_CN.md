# Aurelia - 自主交易代理系统

[English Version](README.md)

## 概述

Aurelia 是一个用 Rust 构建的自主、自我复制的交易代理系统。它具备分布式部署能力、实时监控功能，以及针对加密货币市场的自适应策略执行。

## 特性

- **自主运行**：具备决策能力的自我管理代理
- **自我复制**：跨多服务器的自动部署和扩展
- **实时监控**：全面的健康检查和性能指标
- **自适应策略**：集成机器学习的动态交易策略
- **分布式架构**：支持故障转移的多节点部署
- **安全优先**：基于 SSH 的安全部署，支持多种认证方式

## 架构

系统由多个相互连接的模块组成：

- **Kernel（内核）**：协调所有组件的核心运行时
- **Autonomy Core（自治核心）**：自我管理和决策引擎
- **Strategy Engine（策略引擎）**：交易策略实现
- **Execution Engine（执行引擎）**：订单执行和管理
- **Perception Core（感知核心）**：市场数据收集和分析
- **Monitoring Service（监控服务）**：实时系统监控和告警
- **Deployment Tester（部署测试器）**：自动化测试和验证

## 快速开始

### 前置要求

- Rust 1.70+ 
- Python 3.8+（用于监控面板）
- 部署服务器的 SSH 访问权限
- Binance API 凭证（用于实盘交易）

### 构建

```bash
# 克隆仓库
git clone https://github.com/tricorefile/aurelia.git
cd aurelia

# 构建所有组件
cargo build --release

# 运行测试
cargo test --all
```

### 配置

1. 设置服务器配置：
```bash
cp config/target_servers.json.example config/target_servers.json
# 编辑并填入您的服务器详情
```

2. 配置环境变量：
```bash
cp .env.example .env
# 添加您的 API 密钥
```

3. 设置 SSH 密钥：
```bash
ssh-keygen -t rsa -b 4096 -f ~/.ssh/aurelia_deploy
```

### 部署

部署到单个服务器：
```bash
./scripts/deploy.sh <服务器IP>
```

部署到多个服务器：
```bash
python3 py/smart_deploy.py <服务器IP> --tag latest
```

### 监控

启动监控面板：
```bash
python3 py/api_monitor.py
# 访问 http://localhost:3030
```

## 开发

### 本地运行

```bash
# 启动带监控的内核
./scripts/start_with_monitor.sh

# 运行特定组件
cargo run --bin kernel
```

### 测试

```bash
# 单元测试
cargo test

# 集成测试
cargo test --test integration_test

# 部署测试
./scripts/test/run_deployment_test.sh
```

## CI/CD

项目使用 GitHub Actions 进行持续集成和部署：

- **CI**：每次推送和拉取请求时运行
- **Release**：为多个平台创建二进制文件
- **Deploy**：自动部署到配置的服务器

## 文档

- [API 文档](docs/API_DOCUMENTATION.md)
- [部署指南](docs/DEPLOYMENT_GUIDE.md)
- [服务器配置](docs/SERVER_CONFIG_GUIDE.md)
- [监控指南](docs/MONITORING_COMPARISON.md)
- [GitHub Actions 设置](docs/GITHUB_ACTIONS.md)

## 项目结构

```
aurelia/
├── kernel/              # 核心运行时
├── autonomy_core/       # 自主代理逻辑
├── strategy_engine/     # 交易策略
├── execution_engine/    # 订单执行
├── monitoring_service/  # 系统监控
├── deployment_tester/   # 测试框架
├── scripts/            # 部署脚本
├── py/                 # Python 工具
└── docs/              # 文档
```

## 贡献

欢迎贡献！请在提交 PR 之前阅读我们的贡献指南。

1. Fork 仓库
2. 创建您的特性分支（`git checkout -b feature/amazing-feature`）
3. 提交您的更改（`git commit -m 'feat: add amazing feature'`）
4. 推送到分支（`git push origin feature/amazing-feature`）
5. 开启 Pull Request

### 提交信息规范

我们遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

- `feat:` 新功能
- `fix:` 错误修复
- `docs:` 文档更改
- `style:` 代码样式更改（格式化等）
- `refactor:` 代码重构
- `test:` 测试添加或修改
- `chore:` 构建过程或辅助工具更改
- `perf:` 性能改进

## 安全

- 永远不要提交 API 密钥或敏感数据
- 使用 SSH 密钥进行服务器访问
- 在所有生产服务器上启用 2FA
- 定期进行安全审计和依赖更新

## 许可证

本项目为专有和机密项目。

## 支持

如有问题和疑问：
- GitHub Issues：[https://github.com/tricorefile/aurelia/issues](https://github.com/tricorefile/aurelia/issues)
- 文档：[https://docs.aurelia.io](https://docs.aurelia.io)

## 状态

![CI](https://github.com/tricorefile/aurelia/workflows/CI/badge.svg)
![Release](https://github.com/tricorefile/aurelia/workflows/Release/badge.svg)

---

使用 Rust 用 ❤️ 构建