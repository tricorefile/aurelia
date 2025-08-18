#!/usr/bin/env python3
"""
测试各种SSH连接方法
"""

import subprocess
import os
import sys

# 服务器信息
HOST = "194.146.13.14"
PORT = 22
USER = "root"
PASSWORD = "A8vd0VHDGlpQY3Vu37eCz400fCC1b"

def run_command(cmd):
    """执行命令并返回结果"""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=10)
        return result.stdout, result.stderr, result.returncode
    except subprocess.TimeoutExpired:
        return "", "Command timed out", 1
    except Exception as e:
        return "", str(e), 1

def test_network():
    """测试网络连接"""
    print("1. 测试网络连接...")
    stdout, stderr, code = run_command(f"ping -c 1 {HOST}")
    if code == 0:
        print(f"   ✅ 网络连通")
        return True
    else:
        print(f"   ❌ 网络不通")
        return False

def test_port():
    """测试端口"""
    print("2. 测试SSH端口...")
    stdout, stderr, code = run_command(f"nc -zv {HOST} {PORT} 2>&1")
    if "succeeded" in stdout or "succeeded" in stderr:
        print(f"   ✅ 端口 {PORT} 开放")
        return True
    else:
        print(f"   ❌ 端口 {PORT} 关闭")
        return False

def test_ssh_keyscan():
    """获取SSH主机密钥"""
    print("3. 获取SSH主机密钥...")
    stdout, stderr, code = run_command(f"ssh-keyscan -t rsa -p {PORT} {HOST} 2>/dev/null")
    if stdout:
        print(f"   ✅ SSH服务响应正常")
        print(f"   主机密钥类型: {stdout.split()[0] if stdout else 'unknown'}")
        return True
    else:
        print(f"   ❌ 无法获取主机密钥")
        return False

def test_ssh_password():
    """测试密码登录（使用expect或sshpass）"""
    print("4. 测试密码登录...")
    
    # 检查sshpass是否可用
    _, _, sshpass_check = run_command("which sshpass")
    
    if sshpass_check == 0:
        print("   使用sshpass测试...")
        cmd = f"sshpass -p '{PASSWORD}' ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 {USER}@{HOST} 'echo SUCCESS'"
        stdout, stderr, code = run_command(cmd)
        if code == 0 and "SUCCESS" in stdout:
            print(f"   ✅ 密码登录成功")
            return True
        else:
            print(f"   ❌ 密码登录失败")
            if stderr:
                print(f"   错误: {stderr[:100]}")
            return False
    else:
        print("   ⚠️ sshpass未安装，跳过密码测试")
        print("   安装方法: brew install hudochenkov/sshpass/sshpass")
        return None

def test_ssh_key():
    """测试密钥登录"""
    print("5. 测试SSH密钥登录...")
    
    key_path = os.path.expanduser("~/.ssh/id_rsa")
    if not os.path.exists(key_path):
        print(f"   ⚠️ 私钥不存在: {key_path}")
        print("   生成密钥: ssh-keygen -t rsa -b 4096")
        return None
    
    cmd = f"ssh -i {key_path} -o StrictHostKeyChecking=no -o ConnectTimeout=5 -o PasswordAuthentication=no {USER}@{HOST} 'echo SUCCESS' 2>&1"
    stdout, stderr, code = run_command(cmd)
    
    if code == 0 and "SUCCESS" in stdout:
        print(f"   ✅ 密钥登录成功")
        return True
    else:
        print(f"   ❌ 密钥登录失败")
        if "Permission denied" in stdout or "Permission denied" in stderr:
            print("   需要将公钥添加到服务器")
        return False

def setup_ssh_key():
    """设置SSH密钥"""
    print("\n设置SSH密钥认证:")
    print("-" * 40)
    
    key_path = os.path.expanduser("~/.ssh/id_rsa")
    pub_key_path = key_path + ".pub"
    
    # 1. 检查密钥
    if not os.path.exists(key_path):
        print("1. 生成新的SSH密钥对:")
        print(f"   ssh-keygen -t rsa -b 4096 -f {key_path}")
        print("")
    
    # 2. 添加公钥到服务器
    if os.path.exists(pub_key_path):
        with open(pub_key_path, 'r') as f:
            pub_key = f.read().strip()
        
        print("2. 将公钥添加到服务器 (需要输入密码):")
        print(f"   ssh-copy-id -i {pub_key_path} {USER}@{HOST}")
        print("\n   或手动添加:")
        print(f"   echo '{pub_key}' | ssh {USER}@{HOST} 'cat >> ~/.ssh/authorized_keys'")
        print("\n   使用sshpass自动添加:")
        print(f"   sshpass -p '{PASSWORD}' ssh-copy-id -i {pub_key_path} -o StrictHostKeyChecking=no {USER}@{HOST}")

def main():
    print("=" * 50)
    print("SSH连接测试工具")
    print("=" * 50)
    print(f"\n目标服务器: {USER}@{HOST}:{PORT}")
    print(f"密码: {PASSWORD[:3]}...{PASSWORD[-3:]}")
    print("")
    
    # 运行测试
    results = []
    results.append(("网络连接", test_network()))
    results.append(("端口检查", test_port()))
    results.append(("SSH服务", test_ssh_keyscan()))
    results.append(("密码登录", test_ssh_password()))
    results.append(("密钥登录", test_ssh_key()))
    
    # 总结
    print("\n" + "=" * 50)
    print("测试结果总结:")
    print("-" * 50)
    
    for test_name, result in results:
        if result is True:
            status = "✅ 成功"
        elif result is False:
            status = "❌ 失败"
        else:
            status = "⚠️ 跳过"
        print(f"{test_name:12} : {status}")
    
    # 建议
    print("\n" + "=" * 50)
    if results[3][1] is False:  # 密码登录失败
        print("密码登录失败，可能原因:")
        print("  1. 密码错误")
        print("  2. SSH配置禁用密码认证")
        print("  3. root用户被禁止登录")
        print("\n查看SSH配置:")
        print("  ssh -v root@194.146.13.14")
    
    if results[4][1] is not True:  # 密钥登录未成功
        print("\n推荐使用SSH密钥登录:")
        setup_ssh_key()

if __name__ == "__main__":
    main()