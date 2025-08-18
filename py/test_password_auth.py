#!/usr/bin/env python3
"""
测试密码认证功能
演示如何添加使用密码认证的服务器
"""

import subprocess
import json
import sys

def run_command(cmd, input_text=None):
    """运行命令并返回结果"""
    print(f"\n🔹 执行: {' '.join(cmd)}")
    if input_text:
        result = subprocess.run(cmd, input=input_text, capture_output=True, text=True)
    else:
        result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.stdout:
        print(result.stdout)
    if result.stderr and result.returncode != 0:
        print(f"❌ 错误: {result.stderr}")
    return result.returncode == 0

def main():
    print("=" * 80)
    print("🔐 Aurelia 密码认证测试")
    print("=" * 80)
    
    print("\n📋 当前服务器列表:")
    run_command(["python3", "server_manager.py", "list"])
    
    # 1. 添加使用密码认证的服务器（命令行提供密码）
    print("\n➕ 添加使用密码认证的服务器（命令行提供密码）:")
    success = run_command([
        "python3", "server_manager.py", "add",
        "password-server-1",
        "密码认证服务器1",
        "192.168.1.201",
        "admin",
        "--auth-method", "password",
        "--password", "test123456",
        "--port", "22",
        "--priority", "10",
        "--tags", "password,test"
    ])
    
    if success:
        print("✅ 成功添加密码认证服务器")
    
    # 2. 查看服务器详情
    print("\n📦 查看密码认证服务器详情:")
    run_command(["python3", "server_manager.py", "show", "password-server-1"])
    
    # 3. 添加使用带密码短语的密钥认证服务器
    print("\n➕ 添加使用带密码短语的密钥认证服务器:")
    run_command([
        "python3", "server_manager.py", "add",
        "key-passphrase-server",
        "带密码短语密钥服务器",
        "192.168.1.202",
        "ubuntu",
        "--auth-method", "key-with-passphrase",
        "--ssh-key", "~/.ssh/encrypted_key",
        "--passphrase", "keypass123",
        "--priority", "20",
        "--tags", "encrypted,secure"
    ])
    
    # 4. 列出所有服务器，显示不同的认证方式
    print("\n📋 更新后的服务器列表:")
    run_command(["python3", "server_manager.py", "list"])
    
    # 5. 验证配置文件中的密码存储
    print("\n🔍 检查配置文件中的密码存储:")
    with open("config/target_servers.json", "r") as f:
        config = json.load(f)
        
    for server in config["target_servers"]:
        if server["id"] in ["password-server-1", "key-passphrase-server"]:
            print(f"\n服务器: {server['id']}")
            print(f"  认证方式: {server.get('auth_method', 'key')}")
            if "password_base64" in server:
                print(f"  密码已加密存储: {server['password_base64'][:20]}...")
            if "ssh_key_path" in server:
                print(f"  SSH密钥路径: {server.get('ssh_key_path', '未设置')}")
    
    # 6. 测试连接（会失败因为是假的IP，但可以看到使用了正确的认证方式）
    print("\n🔍 测试密码认证服务器连接:")
    run_command(["python3", "server_manager.py", "test", "password-server-1"])
    
    # 7. 清理测试数据
    print("\n🗑️ 清理测试服务器:")
    run_command(["python3", "server_manager.py", "remove", "password-server-1"])
    run_command(["python3", "server_manager.py", "remove", "key-passphrase-server"])
    
    print("\n" + "=" * 80)
    print("✅ 密码认证测试完成!")
    print("=" * 80)
    
    print("\n📊 支持的认证方式:")
    print("  1. 🔑 SSH密钥认证 (--auth-method key)")
    print("     • 默认方式")
    print("     • 需要指定 --ssh-key 路径")
    print()
    print("  2. 🔐 密码认证 (--auth-method password)")
    print("     • 使用 --password 提供密码")
    print("     • 如果不提供会提示输入")
    print("     • 密码以base64编码存储")
    print()
    print("  3. 🔒 带密码短语的密钥 (--auth-method key-with-passphrase)")
    print("     • 需要 --ssh-key 和 --passphrase")
    print("     • 适用于加密的SSH密钥")
    
    print("\n⚠️ 安全建议:")
    print("  • 避免在命令行直接输入密码（使用交互式输入）")
    print("  • 确保配置文件权限设置正确 (chmod 600)")
    print("  • 定期更换密码和密钥")
    print("  • 优先使用SSH密钥认证而非密码")

if __name__ == "__main__":
    main()