#!/usr/bin/env python3
"""
Aurelia 服务器配置管理工具
用于添加、删除、修改目标服务器配置
"""

import json
import os
import sys
import argparse
import base64
import getpass
from pathlib import Path
from typing import Dict, List, Any

CONFIG_FILE = "config/target_servers.json"

def load_config() -> Dict[str, Any]:
    """加载配置文件"""
    if not os.path.exists(CONFIG_FILE):
        print(f"错误: 配置文件 {CONFIG_FILE} 不存在")
        sys.exit(1)
    
    with open(CONFIG_FILE, 'r', encoding='utf-8') as f:
        return json.load(f)

def save_config(config: Dict[str, Any]) -> None:
    """保存配置文件"""
    with open(CONFIG_FILE, 'w', encoding='utf-8') as f:
        json.dump(config, f, indent=2, ensure_ascii=False)
    print(f"✅ 配置已保存到 {CONFIG_FILE}")

def list_servers(config: Dict[str, Any]) -> None:
    """列出所有服务器"""
    servers = config.get("target_servers", [])
    
    if not servers:
        print("没有配置任何服务器")
        return
    
    print("\n📋 目标服务器列表:")
    print("-" * 80)
    print(f"{'ID':<15} {'名称':<20} {'IP':<15} {'用户':<10} {'启用':<6} {'优先级':<8} {'标签'}")
    print("-" * 80)
    
    for server in servers:
        enabled = "✅" if server["enabled"] else "❌"
        tags = ", ".join(server.get("tags", []))
        print(f"{server['id']:<15} {server['name']:<20} {server['ip']:<15} "
              f"{server['username']:<10} {enabled:<6} {server['priority']:<8} {tags}")
    
    print("-" * 80)
    print(f"总计: {len(servers)} 台服务器 (启用: {sum(1 for s in servers if s['enabled'])})")

def add_server(config: Dict[str, Any], args) -> None:
    """添加新服务器"""
    servers = config.get("target_servers", [])
    
    # 检查ID是否已存在
    if any(s["id"] == args.id for s in servers):
        print(f"❌ 错误: 服务器ID '{args.id}' 已存在")
        sys.exit(1)
    
    new_server = {
        "id": args.id,
        "name": args.name,
        "ip": args.ip,
        "port": args.port,
        "username": args.username,
        "remote_path": args.remote_path,
        "enabled": not args.disabled,
        "priority": args.priority,
        "tags": args.tags.split(",") if args.tags else [],
        "max_retries": args.max_retries,
        "retry_delay_seconds": args.retry_delay
    }
    
    # 处理认证方式
    if args.auth_method == "password":
        if args.password:
            password = args.password
        else:
            password = getpass.getpass(f"请输入服务器 {args.id} 的密码: ")
        new_server["auth_method"] = "password"
        new_server["password_base64"] = base64.b64encode(password.encode()).decode()
    elif args.auth_method == "key-with-passphrase":
        ssh_key_path = os.path.expanduser(args.ssh_key)
        new_server["ssh_key_path"] = ssh_key_path
        if args.passphrase:
            passphrase = args.passphrase
        else:
            passphrase = getpass.getpass(f"请输入SSH密钥的密码短语: ")
        new_server["auth_method"] = "keyWithPassphrase"
        new_server["password_base64"] = base64.b64encode(passphrase.encode()).decode()
    else:  # 默认使用密钥
        ssh_key_path = os.path.expanduser(args.ssh_key)
        new_server["ssh_key_path"] = ssh_key_path
        new_server["auth_method"] = "key"
    
    servers.append(new_server)
    config["target_servers"] = servers
    
    save_config(config)
    print(f"✅ 成功添加服务器: {args.id} ({args.ip})")

def remove_server(config: Dict[str, Any], server_id: str) -> None:
    """删除服务器"""
    servers = config.get("target_servers", [])
    original_count = len(servers)
    
    servers = [s for s in servers if s["id"] != server_id]
    
    if len(servers) == original_count:
        print(f"❌ 错误: 服务器ID '{server_id}' 不存在")
        sys.exit(1)
    
    config["target_servers"] = servers
    save_config(config)
    print(f"✅ 成功删除服务器: {server_id}")

def enable_server(config: Dict[str, Any], server_id: str, enabled: bool) -> None:
    """启用/禁用服务器"""
    servers = config.get("target_servers", [])
    
    for server in servers:
        if server["id"] == server_id:
            server["enabled"] = enabled
            config["target_servers"] = servers
            save_config(config)
            status = "启用" if enabled else "禁用"
            print(f"✅ 成功{status}服务器: {server_id}")
            return
    
    print(f"❌ 错误: 服务器ID '{server_id}' 不存在")
    sys.exit(1)

def update_server(config: Dict[str, Any], args) -> None:
    """更新服务器配置"""
    servers = config.get("target_servers", [])
    
    for server in servers:
        if server["id"] == args.id:
            # 更新指定的字段
            if args.name:
                server["name"] = args.name
            if args.ip:
                server["ip"] = args.ip
            if args.port:
                server["port"] = args.port
            if args.username:
                server["username"] = args.username
            if args.ssh_key:
                server["ssh_key_path"] = os.path.expanduser(args.ssh_key)
            if args.remote_path:
                server["remote_path"] = args.remote_path
            if args.priority is not None:
                server["priority"] = args.priority
            if args.tags:
                server["tags"] = args.tags.split(",")
            if args.max_retries is not None:
                server["max_retries"] = args.max_retries
            if args.retry_delay is not None:
                server["retry_delay_seconds"] = args.retry_delay
            
            config["target_servers"] = servers
            save_config(config)
            print(f"✅ 成功更新服务器: {args.id}")
            return
    
    print(f"❌ 错误: 服务器ID '{args.id}' 不存在")
    sys.exit(1)

def show_server(config: Dict[str, Any], server_id: str) -> None:
    """显示服务器详细信息"""
    servers = config.get("target_servers", [])
    
    for server in servers:
        if server["id"] == server_id:
            print(f"\n📦 服务器详细信息: {server_id}")
            print("-" * 40)
            for key, value in server.items():
                if key == "tags":
                    value = ", ".join(value) if value else "无"
                elif key == "enabled":
                    value = "是" if value else "否"
                elif key == "password_base64":
                    # 不显示实际密码，只显示是否已设置
                    value = "已设置" if value else "未设置"
                    key = "密码"
                elif key == "auth_method":
                    auth_map = {"key": "SSH密钥", "password": "密码", "keyWithPassphrase": "带密码短语的密钥"}
                    value = auth_map.get(value, value)
                    key = "认证方式"
                print(f"{key:<20}: {value}")
            print("-" * 40)
            return
    
    print(f"❌ 错误: 服务器ID '{server_id}' 不存在")
    sys.exit(1)

def test_connection(config: Dict[str, Any], server_id: str) -> None:
    """测试服务器连接"""
    servers = config.get("target_servers", [])
    
    for server in servers:
        if server["id"] == server_id:
            print(f"🔍 测试连接到 {server['name']} ({server['ip']})...")
            
            auth_method = server.get("auth_method", "key")
            
            import subprocess
            
            if auth_method == "password":
                # 使用sshpass进行密码认证
                if "password_base64" not in server:
                    print("❌ 密码未设置")
                    return
                
                try:
                    password = base64.b64decode(server["password_base64"]).decode()
                except:
                    print("❌ 密码解码失败")
                    return
                
                # 检查sshpass是否安装
                check_sshpass = subprocess.run(["which", "sshpass"], capture_output=True)
                if check_sshpass.returncode != 0:
                    print("❌ 需要安装sshpass来测试密码认证: brew install sshpass (Mac) 或 apt install sshpass (Linux)")
                    return
                
                cmd = [
                    "sshpass", "-p", password,
                    "ssh",
                    "-o", "ConnectTimeout=5",
                    "-o", "StrictHostKeyChecking=no",
                    "-p", str(server["port"]),
                    f"{server['username']}@{server['ip']}",
                    "echo 'Connection successful'"
                ]
            else:
                # 使用密钥认证
                ssh_key_path = server.get("ssh_key_path")
                if not ssh_key_path:
                    print("❌ SSH密钥路径未设置")
                    return
                    
                ssh_key = os.path.expanduser(ssh_key_path)
                if not os.path.exists(ssh_key):
                    print(f"❌ SSH密钥不存在: {ssh_key}")
                    return
                
                cmd = [
                    "ssh",
                    "-o", "ConnectTimeout=5",
                    "-o", "StrictHostKeyChecking=no",
                    "-i", ssh_key,
                    "-p", str(server["port"]),
                    f"{server['username']}@{server['ip']}",
                    "echo 'Connection successful'"
                ]
            
            try:
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
                if result.returncode == 0:
                    print(f"✅ 连接成功!")
                else:
                    print(f"❌ 连接失败: {result.stderr}")
            except subprocess.TimeoutExpired:
                print("❌ 连接超时")
            except Exception as e:
                print(f"❌ 连接错误: {e}")
            return
    
    print(f"❌ 错误: 服务器ID '{server_id}' 不存在")
    sys.exit(1)

def main():
    parser = argparse.ArgumentParser(description="Aurelia 服务器配置管理工具")
    subparsers = parser.add_subparsers(dest="command", help="可用命令")
    
    # list 命令
    subparsers.add_parser("list", help="列出所有服务器")
    
    # add 命令
    add_parser = subparsers.add_parser("add", help="添加新服务器")
    add_parser.add_argument("id", help="服务器唯一ID")
    add_parser.add_argument("name", help="服务器名称")
    add_parser.add_argument("ip", help="服务器IP地址")
    add_parser.add_argument("username", help="SSH用户名")
    add_parser.add_argument("--port", type=int, default=22, help="SSH端口 (默认: 22)")
    add_parser.add_argument("--auth-method", choices=["key", "password", "key-with-passphrase"], 
                          default="key", help="认证方式 (默认: key)")
    add_parser.add_argument("--ssh-key", default="~/.ssh/id_rsa", help="SSH密钥路径 (默认: ~/.ssh/id_rsa)")
    add_parser.add_argument("--password", help="SSH密码 (如果不提供会提示输入)")
    add_parser.add_argument("--passphrase", help="SSH密钥密码短语 (如果不提供会提示输入)")
    add_parser.add_argument("--remote-path", default="/home/ubuntu/aurelia", help="远程部署路径")
    add_parser.add_argument("--priority", type=int, default=100, help="优先级 (越小越高)")
    add_parser.add_argument("--tags", help="标签 (逗号分隔)")
    add_parser.add_argument("--disabled", action="store_true", help="初始状态为禁用")
    add_parser.add_argument("--max-retries", type=int, default=3, help="最大重试次数")
    add_parser.add_argument("--retry-delay", type=int, default=60, help="重试延迟(秒)")
    
    # remove 命令
    remove_parser = subparsers.add_parser("remove", help="删除服务器")
    remove_parser.add_argument("id", help="服务器ID")
    
    # enable 命令
    enable_parser = subparsers.add_parser("enable", help="启用服务器")
    enable_parser.add_argument("id", help="服务器ID")
    
    # disable 命令
    disable_parser = subparsers.add_parser("disable", help="禁用服务器")
    disable_parser.add_argument("id", help="服务器ID")
    
    # update 命令
    update_parser = subparsers.add_parser("update", help="更新服务器配置")
    update_parser.add_argument("id", help="服务器ID")
    update_parser.add_argument("--name", help="新名称")
    update_parser.add_argument("--ip", help="新IP地址")
    update_parser.add_argument("--port", type=int, help="新端口")
    update_parser.add_argument("--username", help="新用户名")
    update_parser.add_argument("--ssh-key", help="新SSH密钥路径")
    update_parser.add_argument("--remote-path", help="新远程路径")
    update_parser.add_argument("--priority", type=int, help="新优先级")
    update_parser.add_argument("--tags", help="新标签 (逗号分隔)")
    update_parser.add_argument("--max-retries", type=int, help="新最大重试次数")
    update_parser.add_argument("--retry-delay", type=int, help="新重试延迟")
    
    # show 命令
    show_parser = subparsers.add_parser("show", help="显示服务器详细信息")
    show_parser.add_argument("id", help="服务器ID")
    
    # test 命令
    test_parser = subparsers.add_parser("test", help="测试服务器连接")
    test_parser.add_argument("id", help="服务器ID")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        sys.exit(0)
    
    # 加载配置
    config = load_config()
    
    # 执行命令
    if args.command == "list":
        list_servers(config)
    elif args.command == "add":
        add_server(config, args)
    elif args.command == "remove":
        remove_server(config, args.id)
    elif args.command == "enable":
        enable_server(config, args.id, True)
    elif args.command == "disable":
        enable_server(config, args.id, False)
    elif args.command == "update":
        update_server(config, args)
    elif args.command == "show":
        show_server(config, args.id)
    elif args.command == "test":
        test_connection(config, args.id)

if __name__ == "__main__":
    main()