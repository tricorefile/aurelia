# Python 工具脚本

本目录包含 Aurelia 项目的所有 Python 辅助脚本。

## 监控工具

### api_monitor.py
实时监控 Rust API 的 Web 界面
```bash
python3 api_monitor.py
# 访问 http://localhost:3030
```

### enhanced_monitor.py
增强版交易监控面板
```bash
python3 enhanced_monitor.py
# 访问 http://localhost:3030
```

### simple_monitor_server.py
简单的监控服务器
```bash
python3 simple_monitor_server.py
```

## 服务器管理

### server_manager.py
管理目标服务器配置
```bash
# 列出服务器
python3 server_manager.py list

# 添加服务器
python3 server_manager.py add <id> <name> <ip> --username <user> --password <pass>

# 启用/禁用服务器
python3 server_manager.py enable <id>
python3 server_manager.py disable <id>
```

## 测试工具

### test_ssh_connection.py
测试 SSH 连接（使用 paramiko）
```bash
pip3 install paramiko --break-system-packages --user
python3 test_ssh_connection.py
```

### test_deployment.py
测试部署流程
```bash
python3 test_deployment.py
```

### test_server_config.py
测试服务器配置
```bash
python3 test_server_config.py
```

## 依赖安装

某些脚本需要额外的 Python 包：

```bash
# 安装所有依赖
pip3 install --break-system-packages --user paramiko requests flask

# 或使用虚拟环境（推荐）
python3 -m venv venv
source venv/bin/activate
pip3 install paramiko requests flask
```

## 注意事项

1. 所有脚本都使用 Python 3
2. 监控脚本默认使用端口 3030
3. SSH 相关脚本需要正确的服务器配置
4. 建议在虚拟环境中运行以避免依赖冲突