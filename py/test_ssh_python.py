#!/usr/bin/env python3
"""
使用Python paramiko库测试SSH连接，不需要sshpass
"""

import sys

try:
    import paramiko
except ImportError:
    print("需要安装paramiko: pip3 install paramiko")
    sys.exit(1)

def test_ssh_connection():
    host = "194.146.13.14"
    port = 22
    username = "root"
    password = "Tricorelife@123"
    
    print(f"测试SSH连接到 {host}:{port}")
    print(f"用户名: {username}")
    print("-" * 40)
    
    # 创建SSH客户端
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    
    try:
        # 连接
        print("连接中...")
        client.connect(
            hostname=host,
            port=port,
            username=username,
            password=password,
            timeout=10
        )
        
        print("✅ 连接成功！")
        
        # 执行测试命令
        commands = [
            ("hostname", "主机名"),
            ("pwd", "当前目录"),
            ("ls -la /opt/ 2>/dev/null | head -5", "/opt目录"),
            ("ps aux | grep kernel | grep -v grep", "kernel进程"),
            ("df -h /opt", "磁盘空间"),
        ]
        
        for cmd, desc in commands:
            print(f"\n{desc}:")
            stdin, stdout, stderr = client.exec_command(cmd)
            output = stdout.read().decode().strip()
            if output:
                print(f"  {output}")
            else:
                print(f"  (无输出)")
                
    except Exception as e:
        print(f"❌ 连接失败: {e}")
        return False
    finally:
        client.close()
    
    return True

if __name__ == "__main__":
    success = test_ssh_connection()
    sys.exit(0 if success else 1)