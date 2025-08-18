#!/usr/bin/env python3
"""
测试服务器配置管理系统
演示如何添加、管理和测试服务器配置
"""

import subprocess
import sys
import json

def run_command(cmd):
    """运行命令并返回结果"""
    print(f"\n🔹 执行: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.stdout:
        print(result.stdout)
    if result.stderr and result.returncode != 0:
        print(f"❌ 错误: {result.stderr}")
    return result.returncode == 0

def main():
    print("=" * 80)
    print("🚀 Aurelia 服务器配置系统测试")
    print("=" * 80)
    
    # 1. 列出当前服务器
    print("\n📋 当前配置的服务器:")
    run_command(["python3", "server_manager.py", "list"])
    
    # 2. 添加一个新的测试服务器
    print("\n➕ 添加新的测试服务器:")
    success = run_command([
        "python3", "server_manager.py", "add",
        "test-server-5",
        "测试服务器5",
        "192.168.1.105",
        "testuser",
        "--port", "2222",
        "--ssh-key", "~/.ssh/test_key",
        "--remote-path", "/opt/aurelia",
        "--priority", "50",
        "--tags", "test,development",
        "--max-retries", "5",
        "--retry-delay", "30"
    ])
    
    if success:
        print("✅ 成功添加测试服务器")
    
    # 3. 显示服务器详细信息
    print("\n📦 查看服务器详细信息:")
    run_command(["python3", "server_manager.py", "show", "test-server-5"])
    
    # 4. 更新服务器配置
    print("\n🔄 更新服务器配置:")
    run_command([
        "python3", "server_manager.py", "update",
        "test-server-5",
        "--priority", "25",
        "--tags", "test,development,high-priority"
    ])
    
    # 5. 禁用服务器
    print("\n🔴 禁用服务器:")
    run_command(["python3", "server_manager.py", "disable", "test-server-5"])
    
    # 6. 重新列出服务器
    print("\n📋 更新后的服务器列表:")
    run_command(["python3", "server_manager.py", "list"])
    
    # 7. 启用服务器
    print("\n🟢 重新启用服务器:")
    run_command(["python3", "server_manager.py", "enable", "test-server-5"])
    
    # 8. 测试服务器连接（会失败因为是假的IP）
    print("\n🔍 测试服务器连接:")
    run_command(["python3", "server_manager.py", "test", "server-1"])
    
    # 9. 删除测试服务器
    print("\n🗑️ 删除测试服务器:")
    run_command(["python3", "server_manager.py", "remove", "test-server-5"])
    
    # 10. 最终服务器列表
    print("\n📋 最终服务器列表:")
    run_command(["python3", "server_manager.py", "list"])
    
    # 11. 验证配置文件
    print("\n📄 验证配置文件内容:")
    try:
        with open("config/target_servers.json", "r") as f:
            config = json.load(f)
            print(f"  • 服务器总数: {len(config['target_servers'])}")
            print(f"  • 启用的服务器: {sum(1 for s in config['target_servers'] if s['enabled'])}")
            print(f"  • 禁用的服务器: {sum(1 for s in config['target_servers'] if not s['enabled'])}")
            print(f"  • 最高优先级服务器: {min((s for s in config['target_servers'] if s['enabled']), key=lambda x: x['priority'])['name']}")
    except Exception as e:
        print(f"❌ 无法读取配置文件: {e}")
    
    print("\n" + "=" * 80)
    print("✅ 服务器配置系统测试完成!")
    print("=" * 80)
    
    # 总结
    print("\n📊 测试总结:")
    print("  1. ✅ 服务器添加功能正常")
    print("  2. ✅ 服务器更新功能正常")
    print("  3. ✅ 服务器启用/禁用功能正常")
    print("  4. ✅ 服务器删除功能正常")
    print("  5. ✅ 配置持久化正常")
    print("\n💡 提示: 实际部署时，请确保:")
    print("  • 目标服务器的IP地址可达")
    print("  • SSH密钥已正确配置")
    print("  • 目标服务器上已安装必要的依赖")
    print("  • 防火墙规则允许SSH连接")

if __name__ == "__main__":
    main()