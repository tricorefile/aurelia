#!/usr/bin/env python3
"""
测试SSH连接到服务器
"""

import base64
import socket
import sys

def test_tcp_connection(host, port):
    """测试TCP连接"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)
        result = sock.connect_ex((host, port))
        sock.close()
        
        if result == 0:
            print(f"✅ TCP连接成功: {host}:{port}")
            return True
        else:
            print(f"❌ TCP连接失败: {host}:{port} - 错误码: {result}")
            return False
    except Exception as e:
        print(f"❌ TCP连接错误: {e}")
        return False

def test_ssh_banner(host, port):
    """测试SSH banner"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)
        sock.connect((host, port))
        
        # 接收SSH banner
        banner = sock.recv(1024)
        sock.close()
        
        print(f"✅ SSH Banner: {banner.decode('utf-8', errors='ignore').strip()}")
        return True
    except Exception as e:
        print(f"❌ 无法获取SSH Banner: {e}")
        return False

def main():
    print("=" * 60)
    print("🔍 测试SSH连接到 194.146.13.14")
    print("=" * 60)
    
    # 1. 测试TCP连接
    print("\n1. TCP连接测试:")
    tcp_ok = test_tcp_connection("194.146.13.14", 22)
    
    if tcp_ok:
        # 2. 测试SSH Banner
        print("\n2. SSH Banner测试:")
        test_ssh_banner("194.146.13.14", 22)
    
    # 3. 测试使用paramiko（如果安装了）
    print("\n3. Paramiko SSH测试:")
    try:
        import paramiko
        
        client = paramiko.SSHClient()
        client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        
        # 解码密码
        password_base64 = "QTh2ZDBWSERHbHBRWTNWdTM3ZUN6NDAwZkNDMWI="
        password = base64.b64decode(password_base64).decode()
        
        print(f"   尝试连接 admin@194.146.13.14:22")
        print(f"   使用密码: {password[:3]}...{password[-3:]}")
        
        try:
            client.connect(
                hostname="194.146.13.14",
                port=22,
                username="admin",
                password=password,
                timeout=10,
                look_for_keys=False,
                allow_agent=False
            )
            
            print("   ✅ Paramiko连接成功!")
            
            # 执行测试命令
            stdin, stdout, stderr = client.exec_command("echo 'Connection successful'")
            output = stdout.read().decode()
            print(f"   命令输出: {output.strip()}")
            
            client.close()
            
        except paramiko.AuthenticationException as e:
            print(f"   ❌ 认证失败: {e}")
        except paramiko.SSHException as e:
            print(f"   ❌ SSH错误: {e}")
        except Exception as e:
            print(f"   ❌ 连接错误: {e}")
            
    except ImportError:
        print("   ⚠️ paramiko未安装，跳过此测试")
        print("   安装命令: pip3 install paramiko")
    
    print("\n" + "=" * 60)
    print("测试完成")
    print("=" * 60)

if __name__ == "__main__":
    main()