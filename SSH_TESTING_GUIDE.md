# SSH 连接手动测试指南

## 快速测试方法

### 1. 使用 SSH 命令行测试

#### 基本连接测试
```bash
# 测试连接（会提示输入密码）
ssh root@194.146.13.14

# 使用密码直接连接（需要 sshpass）
brew install hudochenkov/sshpass/sshpass
sshpass -p 'Tricorelife@123' ssh root@194.146.13.14

# 测试执行命令
ssh root@194.146.13.14 'hostname && uname -a'

# 使用详细模式查看连接过程
ssh -v root@194.146.13.14

# 检查 SSH 配置
ssh -T root@194.146.13.14
```

#### 端口和连接测试
```bash
# 测试端口是否开放
nc -zv 194.146.13.14 22

# 测试网络连通性
ping -c 3 194.146.13.14

# 使用 telnet 测试端口
telnet 194.146.13.14 22

# 查看 SSH 版本和支持的算法
ssh -Q cipher 194.146.13.14
ssh -Q mac 194.146.13.14
ssh -Q kex 194.146.13.14
```

### 2. 使用 Python 测试

创建测试脚本 `test_ssh.py`:

```python
#!/usr/bin/env python3
import socket
import sys

def test_ssh_port(host, port=22):
    """测试SSH端口"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)
        result = sock.connect_ex((host, port))
        sock.close()
        
        if result == 0:
            print(f"✅ 端口 {port} 开放")
            return True
        else:
            print(f"❌ 端口 {port} 关闭")
            return False
    except Exception as e:
        print(f"❌ 连接错误: {e}")
        return False

def test_ssh_banner(host, port=22):
    """获取SSH Banner"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)
        sock.connect((host, port))
        
        # 接收 SSH banner
        banner = sock.recv(1024).decode('utf-8').strip()
        print(f"SSH Banner: {banner}")
        
        sock.close()
        return banner
    except Exception as e:
        print(f"获取 banner 失败: {e}")
        return None

if __name__ == "__main__":
    host = "194.146.13.14"
    print(f"测试 SSH 连接到 {host}")
    print("-" * 40)
    
    # 测试端口
    if test_ssh_port(host):
        # 获取 banner
        test_ssh_banner(host)
```

### 3. 使用 OpenSSL 测试

```bash
# 测试 SSL/TLS 连接（如果SSH使用了TLS）
openssl s_client -connect 194.146.13.14:22

# 查看支持的加密套件
openssl ciphers -v
```

### 4. 使用 Rust 测试

运行已创建的测试程序：

```bash
# 编译并运行 Rust SSH 测试
cargo run --example test_ssh_deploy

# 或直接运行编译后的程序
./target/debug/examples/test_ssh_deploy
```

### 5. 调试 SSH 连接问题

#### 常见问题和解决方法

**1. 连接超时**
```bash
# 增加超时时间
ssh -o ConnectTimeout=30 root@194.146.13.14

# 检查防火墙
sudo iptables -L -n | grep 22
```

**2. 认证失败**
```bash
# 查看详细的认证过程
ssh -vvv root@194.146.13.14

# 检查允许的认证方法
ssh -o PreferredAuthentications=password root@194.146.13.14
ssh -o PreferredAuthentications=publickey root@194.146.13.14
```

**3. 协议不兼容**
```bash
# 指定 SSH 协议版本
ssh -2 root@194.146.13.14  # 强制使用 SSH2

# 指定加密算法
ssh -c aes256-ctr root@194.146.13.14
```

**4. Host Key 验证问题**
```bash
# 跳过 host key 检查（仅测试用）
ssh -o StrictHostKeyChecking=no root@194.146.13.14

# 清除已知的 host key
ssh-keygen -R 194.146.13.14
```

### 6. 配置文件测试

创建 `~/.ssh/config`:

```
Host aurelia-test
    HostName 194.146.13.14
    User root
    Port 22
    PasswordAuthentication yes
    PreferredAuthentications password
    StrictHostKeyChecking no
    ConnectTimeout 10
    ServerAliveInterval 60
    ServerAliveCountMax 3
```

然后测试：
```bash
ssh aurelia-test
```

### 7. 使用 curl 测试（如果有 HTTP API）

```bash
# 测试 API 端点
curl -v http://194.146.13.14:8080/health
curl -v http://194.146.13.14:3030/
```

### 8. 批量测试脚本

创建 `test_all.sh`:

```bash
#!/bin/bash

HOST="194.146.13.14"
PORT=22
USER="root"

echo "=== SSH 连接测试 ==="
echo "目标: $USER@$HOST:$PORT"
echo ""

# 1. Ping 测试
echo "1. 网络连通性..."
if ping -c 1 $HOST > /dev/null 2>&1; then
    echo "   ✅ 网络通"
else
    echo "   ❌ 网络不通"
fi

# 2. 端口测试
echo "2. SSH 端口..."
if nc -zv $HOST $PORT 2>&1 | grep -q succeeded; then
    echo "   ✅ 端口开放"
else
    echo "   ❌ 端口关闭"
fi

# 3. SSH Banner
echo "3. SSH Banner..."
timeout 2 bash -c "echo '' | nc $HOST $PORT" 2>/dev/null | head -1

# 4. SSH 连接测试
echo "4. SSH 连接..."
ssh -o ConnectTimeout=5 -o PasswordAuthentication=yes \
    -o PreferredAuthentications=password \
    -o StrictHostKeyChecking=no \
    $USER@$HOST "echo '   ✅ 连接成功'" 2>/dev/null || echo "   ❌ 连接失败"

echo ""
echo "测试完成"
```

### 9. 故障排除清单

- [ ] 网络是否连通？`ping 194.146.13.14`
- [ ] 端口是否开放？`nc -zv 194.146.13.14 22`
- [ ] SSH 服务是否运行？
- [ ] 防火墙是否允许连接？
- [ ] 用户名密码是否正确？
- [ ] SSH 配置是否允许密码认证？
- [ ] 是否有 IP 白名单限制？
- [ ] SSH 版本是否兼容？
- [ ] 加密算法是否支持？

### 10. 日志查看

如果有服务器访问权限：

```bash
# 查看 SSH 日志
sudo tail -f /var/log/auth.log        # Ubuntu/Debian
sudo tail -f /var/log/secure          # CentOS/RHEL

# 查看 SSH 配置
cat /etc/ssh/sshd_config | grep -E "PasswordAuthentication|PermitRootLogin|Port"
```

## 总结

通过以上多种方法可以全面测试 SSH 连接问题。最常见的问题是：

1. **密码认证被禁用** - 服务器可能只允许密钥认证
2. **root 登录被禁用** - 需要使用普通用户
3. **防火墙限制** - IP 白名单或端口限制
4. **密码错误** - 确认密码是否正确

建议按照顺序逐步测试，先确认网络和端口，再测试认证。