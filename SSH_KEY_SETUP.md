# SSH 密钥配置和测试指南

## 当前状态

- ✅ 网络连接正常 (194.146.13.14)
- ✅ SSH端口开放 (22)
- ❌ SSH服务响应异常（无法获取主机密钥）
- 📝 新密码: A8vd0VHDGlpQY3Vu37eCz400fCC1b

## 1. 生成SSH密钥对

如果还没有SSH密钥，生成一个：

```bash
# 生成RSA密钥对（推荐）
ssh-keygen -t rsa -b 4096 -C "aurelia@deployment"

# 或生成ED25519密钥（更安全）
ssh-keygen -t ed25519 -C "aurelia@deployment"
```

按提示操作：
- 保存位置：直接回车使用默认 `~/.ssh/id_rsa`
- 密码短语：可以留空或设置（建议留空便于自动化）

## 2. 手动测试SSH连接

### 测试密码登录

```bash
# 基本连接测试（需要手动输入密码）
ssh root@194.146.13.14

# 使用详细模式查看连接过程
ssh -vvv root@194.146.13.14

# 指定只使用密码认证
ssh -o PreferredAuthentications=password -o PubkeyAuthentication=no root@194.146.13.14
```

密码: `A8vd0VHDGlpQY3Vu37eCz400fCC1b`

### 如果安装了sshpass

```bash
# 安装sshpass (macOS)
brew install hudochenkov/sshpass/sshpass

# 测试密码连接
sshpass -p 'A8vd0VHDGlpQY3Vu37eCz400fCC1b' ssh -o StrictHostKeyChecking=no root@194.146.13.14 'hostname'
```

## 3. 配置密钥登录

### 方法1：使用ssh-copy-id（推荐）

```bash
# 复制公钥到服务器（需要输入密码）
ssh-copy-id -i ~/.ssh/id_rsa.pub root@194.146.13.14

# 或使用sshpass自动化
sshpass -p 'A8vd0VHDGlpQY3Vu37eCz400fCC1b' ssh-copy-id -i ~/.ssh/id_rsa.pub -o StrictHostKeyChecking=no root@194.146.13.14
```

### 方法2：手动添加公钥

```bash
# 1. 查看本地公钥
cat ~/.ssh/id_rsa.pub

# 2. 登录服务器
ssh root@194.146.13.14

# 3. 在服务器上添加公钥
mkdir -p ~/.ssh
chmod 700 ~/.ssh
echo "你的公钥内容" >> ~/.ssh/authorized_keys
chmod 600 ~/.ssh/authorized_keys
```

### 方法3：一行命令添加

```bash
cat ~/.ssh/id_rsa.pub | ssh root@194.146.13.14 'mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys'
```

## 4. 测试密钥登录

```bash
# 测试密钥登录
ssh -i ~/.ssh/id_rsa root@194.146.13.14

# 如果成功，应该直接登录无需密码
```

## 5. 在Rust代码中使用

### 密码登录

```rust
use autonomy_core::SshDeployer;

let mut deployer = SshDeployer::new();
deployer.connect_with_password(
    "194.146.13.14", 
    22, 
    "root", 
    "A8vd0VHDGlpQY3Vu37eCz400fCC1b"
)?;
```

### 密钥登录

```rust
use autonomy_core::SshDeployer;
use std::path::PathBuf;

let mut deployer = SshDeployer::new();
let key_path = PathBuf::from("/Users/harryma/.ssh/id_rsa");

// 无密码短语
deployer.connect_with_key(
    "194.146.13.14",
    22,
    "root", 
    &key_path,
    None
)?;

// 有密码短语
deployer.connect_with_key(
    "194.146.13.14",
    22,
    "root",
    &key_path,
    Some("passphrase")
)?;
```

## 6. 故障排除

### SSH连接失败

1. **检查SSH服务状态**
   ```bash
   # 如果能登录服务器
   systemctl status sshd
   ```

2. **检查SSH配置**
   ```bash
   # 查看是否允许密码认证
   grep PasswordAuthentication /etc/ssh/sshd_config
   
   # 查看是否允许root登录
   grep PermitRootLogin /etc/ssh/sshd_config
   ```

3. **检查防火墙**
   ```bash
   # 测试端口
   telnet 194.146.13.14 22
   nc -zv 194.146.13.14 22
   ```

### SSH握手失败

如果出现"SSH handshake failed"错误：

1. **协议版本问题**
   ```bash
   # 强制使用SSH v2
   ssh -2 root@194.146.13.14
   ```

2. **加密算法不兼容**
   ```bash
   # 查看支持的算法
   ssh -Q cipher
   ssh -Q mac
   ssh -Q kex
   
   # 指定算法
   ssh -c aes256-ctr root@194.146.13.14
   ```

3. **主机密钥问题**
   ```bash
   # 清除已知主机
   ssh-keygen -R 194.146.13.14
   
   # 重新连接
   ssh -o StrictHostKeyChecking=no root@194.146.13.14
   ```

## 7. 配置文件优化

创建或编辑 `~/.ssh/config`:

```
Host aurelia
    HostName 194.146.13.14
    User root
    Port 22
    IdentityFile ~/.ssh/id_rsa
    PasswordAuthentication yes
    StrictHostKeyChecking no
    ConnectTimeout 10
    ServerAliveInterval 60
```

然后可以简化连接：
```bash
ssh aurelia
```

## 8. 自动化脚本

创建测试脚本 `test_ssh.sh`:

```bash
#!/bin/bash

HOST="194.146.13.14"
USER="root"
PASSWORD="A8vd0VHDGlpQY3Vu37eCz400fCC1b"

echo "测试SSH连接..."

# 测试密钥
if ssh -o PasswordAuthentication=no -o ConnectTimeout=5 $USER@$HOST 'echo "密钥登录成功"' 2>/dev/null; then
    echo "✅ 密钥认证工作正常"
    exit 0
fi

# 测试密码（需要sshpass）
if command -v sshpass &> /dev/null; then
    if sshpass -p "$PASSWORD" ssh -o StrictHostKeyChecking=no $USER@$HOST 'echo "密码登录成功"' 2>/dev/null; then
        echo "✅ 密码认证工作正常"
        exit 0
    fi
fi

echo "❌ SSH连接失败"
exit 1
```

## 当前测试结果

基于测试，服务器存在以下情况：
- ✅ 服务器可达（ping通）
- ✅ 22端口开放
- ❌ SSH服务可能配置异常（无法获取主机密钥）
- ❓ 可能需要特定的SSH客户端配置

建议：
1. 首先尝试手动SSH连接确认密码
2. 如果密码正确，配置密钥登录
3. 使用密钥登录进行自动化部署