#!/usr/bin/env python3
"""
简单的Python脚本来测试Rust SSH部署功能
"""

import subprocess
import sys
import time

def run_command(cmd):
    """执行命令并返回输出"""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
        return result.stdout, result.stderr, result.returncode
    except Exception as e:
        return "", str(e), 1

def main():
    print("\n" + "="*50)
    print("  测试纯Rust SSH部署到目标服务器")
    print("="*50 + "\n")
    
    target_ip = "194.146.13.14"
    password = "Tricorelife@123"
    
    print(f"🎯 目标服务器: {target_ip}")
    print(f"📦 二进制文件: ./target/release/kernel\n")
    
    # 步骤1: 编译kernel
    print("1️⃣ 编译kernel...")
    stdout, stderr, code = run_command("cargo build --release --bin kernel")
    if code != 0:
        print(f"❌ 编译失败: {stderr}")
        return 1
    print("✅ 编译成功\n")
    
    # 步骤2: 测试SSH连接（使用sshpass）
    print("2️⃣ 测试SSH连接...")
    test_cmd = f"sshpass -p '{password}' ssh -o StrictHostKeyChecking=no root@{target_ip} 'hostname && pwd'"
    stdout, stderr, code = run_command(test_cmd)
    
    if code != 0:
        print(f"❌ SSH连接失败: {stderr}")
        print("\n请确保:")
        print("  1. 安装了sshpass: brew install hudochenkov/sshpass/sshpass")
        print("  2. 服务器IP和密码正确")
        print("  3. 端口22开放")
        return 1
    
    print(f"✅ SSH连接成功")
    print(f"   服务器响应: {stdout.strip()}\n")
    
    # 步骤3: 使用deploy.sh部署
    print("3️⃣ 使用deploy.sh部署...")
    deploy_cmd = f"./deploy.sh deploy {target_ip} -P '{password}'"
    print(f"   执行: {deploy_cmd}")
    stdout, stderr, code = run_command(deploy_cmd)
    
    if code != 0:
        print(f"⚠️ deploy.sh部署可能失败: {stderr}")
    else:
        print("✅ deploy.sh部署完成")
    
    print(stdout)
    
    # 步骤4: 启动服务
    print("\n4️⃣ 启动kernel服务...")
    start_cmd = f"./deploy.sh start {target_ip} -P '{password}'"
    stdout, stderr, code = run_command(start_cmd)
    
    if code != 0:
        print(f"⚠️ 启动可能失败: {stderr}")
    else:
        print("✅ 启动命令已执行")
    
    # 步骤5: 检查状态
    print("\n5️⃣ 检查服务状态...")
    time.sleep(3)  # 等待服务启动
    
    status_cmd = f"sshpass -p '{password}' ssh -o StrictHostKeyChecking=no root@{target_ip} 'ps aux | grep kernel | grep -v grep'"
    stdout, stderr, code = run_command(status_cmd)
    
    if stdout.strip():
        print("✅ Kernel正在运行!")
        print(f"   进程: {stdout.strip()}")
    else:
        print("⚠️ Kernel未检测到运行")
    
    # 步骤6: 获取日志
    print("\n6️⃣ 获取最新日志...")
    log_cmd = f"sshpass -p '{password}' ssh -o StrictHostKeyChecking=no root@{target_ip} 'tail -20 /opt/aurelia/logs/aurelia.log 2>/dev/null || echo \"无日志\"'"
    stdout, stderr, code = run_command(log_cmd)
    print(f"📜 日志内容:\n{stdout}")
    
    # 步骤7: 检查端口
    print("\n7️⃣ 检查监听端口...")
    port_cmd = f"sshpass -p '{password}' ssh -o StrictHostKeyChecking=no root@{target_ip} 'ss -tlnp | grep -E \"(8080|3030)\" || echo \"端口未监听\"'"
    stdout, stderr, code = run_command(port_cmd)
    print(f"🔌 端口状态:\n{stdout}")
    
    print("\n" + "="*50)
    print("  测试完成")
    print("="*50)
    
    if stdout.strip() != "端口未监听":
        print("\n✅ 部署成功!")
        print(f"\n访问方式:")
        print(f"  SSH: ssh root@{target_ip}")
        print(f"  API: http://{target_ip}:8080")
        print(f"  监控: http://{target_ip}:3030")
    else:
        print("\n⚠️ 部署可能未完全成功，请检查日志")
    
    return 0

if __name__ == "__main__":
    sys.exit(main())