#!/usr/bin/env python3
"""
Smart deployment script for Aurelia
Automatically detects target server architecture and downloads appropriate binary from GitHub releases
"""

import argparse
import json
import os
import platform
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path
from typing import Optional, Tuple

# ANSI color codes
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
NC = '\033[0m'  # No Color

class DeploymentManager:
    def __init__(self, github_repo: str = "tricorefile/aurelia"):
        self.github_repo = github_repo
        self.api_base = "https://api.github.com"
        
    def log_info(self, message: str):
        print(f"{GREEN}[INFO]{NC} {message}")
    
    def log_warn(self, message: str):
        print(f"{YELLOW}[WARN]{NC} {message}")
    
    def log_error(self, message: str):
        print(f"{RED}[ERROR]{NC} {message}")
    
    def run_ssh_command(self, host: str, user: str, key_path: str, command: str) -> Tuple[int, str, str]:
        """Execute SSH command and return (returncode, stdout, stderr)"""
        ssh_cmd = [
            "ssh", "-i", key_path,
            "-o", "StrictHostKeyChecking=no",
            "-o", "ConnectTimeout=10",
            f"{user}@{host}",
            command
        ]
        
        result = subprocess.run(ssh_cmd, capture_output=True, text=True)
        return result.returncode, result.stdout, result.stderr
    
    def detect_server_architecture(self, host: str, user: str, key_path: str) -> Tuple[str, str]:
        """Detect target server architecture and OS"""
        self.log_info(f"Detecting server architecture for {host}...")
        
        # Get architecture
        ret, arch, _ = self.run_ssh_command(host, user, key_path, "uname -m")
        if ret != 0:
            raise Exception(f"Failed to detect architecture: {arch}")
        arch = arch.strip()
        
        # Get OS info
        ret, os_info, _ = self.run_ssh_command(
            host, user, key_path, 
            'cat /etc/os-release | grep "^ID=" | cut -d= -f2 | tr -d \'"\''
        )
        os_id = os_info.strip() if ret == 0 else "unknown"
        
        self.log_info(f"Detected: {arch} on {os_id}")
        return arch, os_id
    
    def map_architecture_to_asset(self, arch: str, os_id: str) -> str:
        """Map server architecture to release asset name"""
        arch_map = {
            "x86_64": "linux-x86_64",
            "amd64": "linux-x86_64",
            "aarch64": "linux-aarch64",
            "arm64": "linux-aarch64",
        }
        
        base_name = arch_map.get(arch)
        if not base_name:
            raise Exception(f"Unsupported architecture: {arch}")
        
        # Use musl version for Alpine Linux
        if os_id == "alpine":
            if base_name == "linux-x86_64":
                return "aurelia-linux-x86_64-musl"
        
        return f"aurelia-{base_name}"
    
    def get_release_info(self, tag: str = "latest") -> dict:
        """Get release information from GitHub"""
        self.log_info(f"Fetching release info for tag: {tag}")
        
        if tag == "latest":
            url = f"{self.api_base}/repos/{self.github_repo}/releases/latest"
        else:
            url = f"{self.api_base}/repos/{self.github_repo}/releases/tags/{tag}"
        
        try:
            with urllib.request.urlopen(url) as response:
                return json.loads(response.read())
        except urllib.error.HTTPError as e:
            if e.code == 404:
                raise Exception(f"Release not found: {tag}")
            raise
    
    def download_asset(self, release_info: dict, asset_name: str, dest_path: Path) -> Path:
        """Download specific asset from release"""
        asset_file = f"{asset_name}.tar.gz"
        
        # Find asset URL
        download_url = None
        for asset in release_info.get("assets", []):
            if asset["name"] == asset_file:
                download_url = asset["browser_download_url"]
                break
        
        if not download_url:
            available = [a["name"] for a in release_info.get("assets", [])]
            raise Exception(f"Asset not found: {asset_file}\nAvailable: {available}")
        
        self.log_info(f"Downloading {asset_file}...")
        
        # Download file
        output_file = dest_path / asset_file
        urllib.request.urlretrieve(download_url, output_file)
        
        return output_file
    
    def extract_binary(self, archive_path: Path) -> Path:
        """Extract kernel binary from archive"""
        self.log_info("Extracting binary...")
        
        extract_dir = archive_path.parent
        subprocess.run(
            ["tar", "xzf", str(archive_path), "-C", str(extract_dir)],
            check=True
        )
        
        binary_path = extract_dir / "kernel"
        if not binary_path.exists():
            raise Exception("Binary 'kernel' not found in archive")
        
        return binary_path
    
    def deploy_binary(self, binary_path: Path, host: str, user: str, key_path: str, 
                     deploy_path: str = "/opt/aurelia"):
        """Deploy binary to target server"""
        self.log_info(f"Deploying to {host}:{deploy_path}")
        
        # Create deployment directory
        self.run_ssh_command(
            host, user, key_path,
            f"sudo mkdir -p {deploy_path} && sudo chown {user}:{user} {deploy_path}"
        )
        
        # Stop existing service
        self.log_info("Stopping existing service...")
        self.run_ssh_command(host, user, key_path, "sudo systemctl stop aurelia 2>/dev/null || true")
        
        # Copy binary
        self.log_info("Uploading binary...")
        scp_cmd = [
            "scp", "-i", key_path,
            "-o", "StrictHostKeyChecking=no",
            str(binary_path),
            f"{user}@{host}:{deploy_path}/"
        ]
        subprocess.run(scp_cmd, check=True)
        
        # Set permissions
        self.run_ssh_command(host, user, key_path, f"chmod +x {deploy_path}/kernel")
    
    def setup_systemd_service(self, host: str, user: str, key_path: str, 
                            deploy_path: str = "/opt/aurelia"):
        """Create and configure systemd service"""
        self.log_info("Setting up systemd service...")
        
        service_content = f"""[Unit]
Description=Aurelia Autonomous System
After=network.target

[Service]
Type=simple
User={user}
WorkingDirectory={deploy_path}
ExecStart={deploy_path}/kernel
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target"""
        
        # Create service file
        cmd = f"echo '{service_content}' | sudo tee /etc/systemd/system/aurelia.service > /dev/null"
        self.run_ssh_command(host, user, key_path, cmd)
        
        # Reload systemd
        self.run_ssh_command(host, user, key_path, "sudo systemctl daemon-reload")
    
    def start_service(self, host: str, user: str, key_path: str) -> bool:
        """Start and enable the service"""
        self.log_info("Starting service...")
        
        # Start service
        self.run_ssh_command(host, user, key_path, "sudo systemctl start aurelia")
        self.run_ssh_command(host, user, key_path, "sudo systemctl enable aurelia")
        
        # Check status
        import time
        time.sleep(3)
        
        ret, status, _ = self.run_ssh_command(host, user, key_path, "sudo systemctl is-active aurelia")
        
        if status.strip() == "active":
            self.log_info("✅ Service started successfully!")
            
            # Show status
            ret, output, _ = self.run_ssh_command(
                host, user, key_path, 
                "sudo systemctl status aurelia --no-pager | head -15"
            )
            print(f"\n{BLUE}Service Status:{NC}")
            print(output)
            
            # Show logs
            ret, logs, _ = self.run_ssh_command(
                host, user, key_path,
                "sudo journalctl -u aurelia -n 10 --no-pager"
            )
            print(f"\n{BLUE}Recent Logs:{NC}")
            print(logs)
            
            return True
        else:
            self.log_error("❌ Service failed to start")
            ret, logs, _ = self.run_ssh_command(
                host, user, key_path,
                "sudo journalctl -u aurelia -n 20 --no-pager"
            )
            print(logs)
            return False
    
    def deploy(self, host: str, user: str = "ubuntu", key_path: str = None, 
              tag: str = "latest", deploy_path: str = "/opt/aurelia"):
        """Main deployment process"""
        if not key_path:
            key_path = str(Path.home() / ".ssh" / "id_rsa")
        
        print("=" * 50)
        print("  Aurelia Smart Deployment")
        print("=" * 50)
        print(f"Target: {user}@{host}")
        print(f"Release: {tag}")
        print(f"Deploy Path: {deploy_path}")
        print()
        
        try:
            # 1. Detect architecture
            arch, os_id = self.detect_server_architecture(host, user, key_path)
            
            # 2. Map to asset name
            asset_name = self.map_architecture_to_asset(arch, os_id)
            self.log_info(f"Target asset: {asset_name}")
            
            # 3. Get release info
            release_info = self.get_release_info(tag)
            
            # 4. Download binary
            with tempfile.TemporaryDirectory() as temp_dir:
                temp_path = Path(temp_dir)
                archive_path = self.download_asset(release_info, asset_name, temp_path)
                
                # 5. Extract binary
                binary_path = self.extract_binary(archive_path)
                
                # 6. Deploy to server
                self.deploy_binary(binary_path, host, user, key_path, deploy_path)
            
            # 7. Setup systemd service
            self.setup_systemd_service(host, user, key_path, deploy_path)
            
            # 8. Start service
            success = self.start_service(host, user, key_path)
            
            if success:
                print()
                print("=" * 50)
                print("  Deployment Complete!")
                print("=" * 50)
                print(f"\n{GREEN}Access commands:{NC}")
                print(f"  SSH: ssh -i {key_path} {user}@{host}")
                print(f"  Logs: ssh -i {key_path} {user}@{host} 'sudo journalctl -u aurelia -f'")
                print(f"  Status: ssh -i {key_path} {user}@{host} 'sudo systemctl status aurelia'")
                print(f"  Restart: ssh -i {key_path} {user}@{host} 'sudo systemctl restart aurelia'")
                return 0
            else:
                return 1
                
        except Exception as e:
            self.log_error(str(e))
            return 1

def main():
    parser = argparse.ArgumentParser(description="Deploy Aurelia to remote server")
    parser.add_argument("host", help="Target server hostname or IP")
    parser.add_argument("-u", "--user", default="ubuntu", help="SSH user (default: ubuntu)")
    parser.add_argument("-k", "--key", help="SSH private key path")
    parser.add_argument("-t", "--tag", default="latest", help="Release tag (default: latest)")
    parser.add_argument("-p", "--path", default="/opt/aurelia", help="Deploy path (default: /opt/aurelia)")
    parser.add_argument("-r", "--repo", default="tricorefile/aurelia", help="GitHub repo")
    
    args = parser.parse_args()
    
    manager = DeploymentManager(args.repo)
    sys.exit(manager.deploy(
        host=args.host,
        user=args.user,
        key_path=args.key,
        tag=args.tag,
        deploy_path=args.path
    ))

if __name__ == "__main__":
    main()