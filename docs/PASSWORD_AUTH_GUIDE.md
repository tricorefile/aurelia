# Aurelia 密码认证配置指南

## 概述

Aurelia 现在支持三种SSH认证方式：
1. **SSH密钥认证**（默认）
2. **密码认证**
3. **带密码短语的SSH密钥认证**

所有密码都使用Base64编码存储在配置文件中，提供基本的安全保护。

## 认证方式详解

### 1. SSH密钥认证（推荐）

最安全的认证方式，使用公钥/私钥对进行认证。

```bash
python3 server_manager.py add server-1 "生产服务器" 192.168.1.100 ubuntu \
  --auth-method key \
  --ssh-key ~/.ssh/id_rsa \
  --port 22
```

### 2. 密码认证

使用用户名和密码进行认证。适用于无法使用SSH密钥的场景。

```bash
# 方式1：命令行提供密码（不推荐，会在命令历史中留下痕迹）
python3 server_manager.py add server-2 "测试服务器" 192.168.1.101 admin \
  --auth-method password \
  --password "your_password"

# 方式2：交互式输入密码（推荐）
python3 server_manager.py add server-3 "开发服务器" 192.168.1.102 developer \
  --auth-method password
# 系统会提示：请输入服务器 server-3 的密码: 
```

### 3. 带密码短语的SSH密钥

用于加密的SSH私钥，需要密码短语才能使用。

```bash
python3 server_manager.py add server-4 "安全服务器" 192.168.1.103 secure_user \
  --auth-method key-with-passphrase \
  --ssh-key ~/.ssh/encrypted_id_rsa \
  --passphrase "key_passphrase"
```

## 配置文件格式

密码和认证信息存储在 `config/target_servers.json` 中：

```json
{
  "target_servers": [
    {
      "id": "password-server",
      "name": "密码认证服务器",
      "ip": "192.168.1.100",
      "port": 22,
      "username": "admin",
      "auth_method": "password",
      "password_base64": "cGFzc3dvcmQxMjM=",  // Base64编码的密码
      "remote_path": "/home/admin/aurelia",
      "enabled": true,
      "priority": 1
    },
    {
      "id": "key-server",
      "name": "密钥认证服务器",
      "ip": "192.168.1.101",
      "port": 22,
      "username": "ubuntu",
      "auth_method": "key",
      "ssh_key_path": "/Users/username/.ssh/id_rsa",
      "remote_path": "/home/ubuntu/aurelia",
      "enabled": true,
      "priority": 2
    }
  ]
}
```

## Rust代码集成

在Rust代码中使用不同的认证方式：

```rust
use autonomy_core::{TargetServer, AuthMethod};

// 创建密码认证服务器
let mut password_server = TargetServer::new(
    "server-1".to_string(),
    "Password Server".to_string(),
    "192.168.1.100".to_string(),
    "admin".to_string()
);
password_server.set_password("secure_password");

// 创建密钥认证服务器
let key_server = TargetServer::new(
    "server-2".to_string(),
    "Key Server".to_string(),
    "192.168.1.101".to_string(),
    "ubuntu".to_string()
);

// 获取解码后的密码
if let Some(password) = password_server.get_password() {
    println!("Password: {}", password);
}

// 检查认证方式
match password_server.auth_method {
    AuthMethod::Password => println!("使用密码认证"),
    AuthMethod::Key => println!("使用密钥认证"),
    AuthMethod::KeyWithPassphrase => println!("使用带密码短语的密钥"),
}
```

## 安全最佳实践

### 1. 密码存储安全

- 配置文件权限设置：
```bash
chmod 600 config/target_servers.json
```

- 使用环境变量提供密码：
```bash
export SERVER_PASSWORD="your_password"
python3 server_manager.py add server-1 "服务器" 192.168.1.100 admin \
  --auth-method password \
  --password "$SERVER_PASSWORD"
```

### 2. 密码管理建议

- **避免硬编码密码**：不要在脚本中硬编码密码
- **定期更换密码**：建议每3-6个月更换一次密码
- **使用强密码**：至少12位，包含大小写字母、数字和特殊字符
- **优先使用SSH密钥**：密钥认证比密码更安全

### 3. 交互式密码输入

推荐使用交互式输入避免密码泄露：

```python
import getpass

# Python示例
password = getpass.getpass("请输入密码: ")
```

## 测试连接

测试不同认证方式的连接：

```bash
# 测试密钥认证
python3 server_manager.py test key-server

# 测试密码认证（需要安装sshpass）
# Mac: brew install hudochenkov/sshpass/sshpass
# Linux: apt install sshpass
python3 server_manager.py test password-server
```

## 故障排除

### 1. 密码认证失败

- 检查密码是否正确
- 确认服务器允许密码认证（/etc/ssh/sshd_config中PasswordAuthentication yes）
- 检查防火墙设置

### 2. 密钥认证失败

- 检查密钥文件权限（chmod 600）
- 确认公钥已添加到服务器的authorized_keys
- 验证密钥路径是否正确

### 3. Base64编码/解码

手动编码/解码密码：

```bash
# 编码
echo -n "password123" | base64
# 输出: cGFzc3dvcmQxMjM=

# 解码
echo "cGFzc3dvcmQxMjM=" | base64 -d
# 输出: password123
```

## 迁移指南

从旧版本迁移到支持密码认证的版本：

1. **备份现有配置**：
```bash
cp config/target_servers.json config/target_servers.json.backup
```

2. **更新现有服务器配置**：
为每个服务器添加 `auth_method` 字段：
```json
"auth_method": "key"  // 对于使用密钥的服务器
```

3. **测试连接**：
```bash
python3 server_manager.py test <server-id>
```

## 相关命令参考

```bash
# 查看所有服务器
python3 server_manager.py list

# 显示服务器详情（密码会显示为"已设置"）
python3 server_manager.py show <server-id>

# 更新服务器认证方式
python3 server_manager.py update <server-id> --auth-method password

# 删除服务器
python3 server_manager.py remove <server-id>
```

## 注意事项

1. **Base64不是加密**：Base64只是编码，不提供真正的安全性
2. **生产环境建议**：在生产环境中，考虑使用密钥管理服务（如HashiCorp Vault）
3. **审计日志**：记录所有认证尝试和配置更改
4. **最小权限原则**：为每个服务器使用不同的账户和最小必要权限