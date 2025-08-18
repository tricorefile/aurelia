#!/bin/bash

# Smart deployment script that detects target server architecture and downloads appropriate binary

set -e

# Configuration
GITHUB_REPO="tricorefile/aurelia"
RELEASE_TAG="${1:-latest}"  # Use provided tag or 'latest'
TARGET_SERVER="${2}"
SSH_KEY="${3:-$HOME/.ssh/id_rsa}"
SSH_USER="${4:-ubuntu}"
DEPLOY_PATH="/opt/aurelia"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check parameters
if [ -z "$TARGET_SERVER" ]; then
    log_error "Usage: $0 [release_tag] <target_server> [ssh_key] [ssh_user]"
    echo "Example: $0 v1.0.0 192.168.1.100"
    echo "         $0 latest 192.168.1.100 ~/.ssh/deploy_key root"
    exit 1
fi

echo "======================================"
echo "  Smart Deployment Script"
echo "======================================"
echo ""
log_info "GitHub Repo: $GITHUB_REPO"
log_info "Release Tag: $RELEASE_TAG"
log_info "Target Server: $SSH_USER@$TARGET_SERVER"
log_info "Deploy Path: $DEPLOY_PATH"
echo ""

# Step 1: Detect target server architecture
log_info "Detecting target server architecture..."

ARCH_INFO=$(ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no "$SSH_USER@$TARGET_SERVER" \
    'uname -m && cat /etc/os-release | grep "^ID=" | cut -d= -f2 | tr -d \"')

ARCH=$(echo "$ARCH_INFO" | head -1)
OS_ID=$(echo "$ARCH_INFO" | tail -1)

log_info "Detected architecture: $ARCH"
log_info "Detected OS: $OS_ID"

# Map architecture to release asset name
case "$ARCH" in
    x86_64|amd64)
        if [[ "$OS_ID" == "alpine" ]]; then
            ASSET_NAME="aurelia-linux-x86_64-musl"
        else
            ASSET_NAME="aurelia-linux-x86_64"
        fi
        ;;
    aarch64|arm64)
        ASSET_NAME="aurelia-linux-aarch64"
        ;;
    *)
        log_error "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

log_info "Will download: ${ASSET_NAME}.tar.gz"

# Step 2: Get release download URL from GitHub
log_info "Fetching release information from GitHub..."

if [ "$RELEASE_TAG" == "latest" ]; then
    API_URL="https://api.github.com/repos/$GITHUB_REPO/releases/latest"
else
    API_URL="https://api.github.com/repos/$GITHUB_REPO/releases/tags/$RELEASE_TAG"
fi

# Get release info
RELEASE_INFO=$(curl -s "$API_URL")

if echo "$RELEASE_INFO" | grep -q "Not Found"; then
    log_error "Release not found: $RELEASE_TAG"
    exit 1
fi

# Extract download URL for our asset
DOWNLOAD_URL=$(echo "$RELEASE_INFO" | \
    grep -o "\"browser_download_url\": \"[^\"]*${ASSET_NAME}.tar.gz\"" | \
    cut -d'"' -f4)

if [ -z "$DOWNLOAD_URL" ]; then
    log_error "Asset not found: ${ASSET_NAME}.tar.gz"
    log_info "Available assets:"
    echo "$RELEASE_INFO" | grep -o "\"name\": \"[^\"]*\"" | cut -d'"' -f4
    exit 1
fi

log_info "Download URL: $DOWNLOAD_URL"

# Step 3: Download the binary
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

log_info "Downloading binary to temporary directory..."
curl -L -o "${ASSET_NAME}.tar.gz" "$DOWNLOAD_URL"

# Verify download
if [ ! -f "${ASSET_NAME}.tar.gz" ]; then
    log_error "Download failed"
    exit 1
fi

# Extract
log_info "Extracting archive..."
tar xzf "${ASSET_NAME}.tar.gz"

if [ ! -f "kernel" ]; then
    log_error "Binary 'kernel' not found in archive"
    ls -la
    exit 1
fi

log_info "Binary extracted successfully"

# Step 4: Deploy to target server
log_info "Deploying to target server..."

# Create deployment directory on target
ssh -i "$SSH_KEY" "$SSH_USER@$TARGET_SERVER" "sudo mkdir -p $DEPLOY_PATH && sudo chown $SSH_USER:$SSH_USER $DEPLOY_PATH"

# Stop existing service if running
log_info "Stopping existing service..."
ssh -i "$SSH_KEY" "$SSH_USER@$TARGET_SERVER" "sudo systemctl stop aurelia 2>/dev/null || true"

# Copy binary to server
log_info "Uploading binary..."
scp -i "$SSH_KEY" kernel "$SSH_USER@$TARGET_SERVER:$DEPLOY_PATH/"

# Set permissions
ssh -i "$SSH_KEY" "$SSH_USER@$TARGET_SERVER" "chmod +x $DEPLOY_PATH/kernel"

# Step 5: Create systemd service if it doesn't exist
log_info "Setting up systemd service..."

ssh -i "$SSH_KEY" "$SSH_USER@$TARGET_SERVER" << 'ENDSSH'
if [ ! -f /etc/systemd/system/aurelia.service ]; then
    sudo tee /etc/systemd/system/aurelia.service > /dev/null << 'EOF'
[Unit]
Description=Aurelia Autonomous System
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/opt/aurelia
ExecStart=/opt/aurelia/kernel
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF
    sudo systemctl daemon-reload
fi
ENDSSH

# Step 6: Start the service
log_info "Starting Aurelia service..."
ssh -i "$SSH_KEY" "$SSH_USER@$TARGET_SERVER" "sudo systemctl start aurelia && sudo systemctl enable aurelia"

# Step 7: Verify deployment
log_info "Verifying deployment..."
sleep 3

SERVICE_STATUS=$(ssh -i "$SSH_KEY" "$SSH_USER@$TARGET_SERVER" "sudo systemctl is-active aurelia")

if [ "$SERVICE_STATUS" == "active" ]; then
    log_info "✅ Deployment successful! Service is running."
    
    # Show service status
    echo ""
    log_info "Service status:"
    ssh -i "$SSH_KEY" "$SSH_USER@$TARGET_SERVER" "sudo systemctl status aurelia --no-pager | head -15"
    
    # Show recent logs
    echo ""
    log_info "Recent logs:"
    ssh -i "$SSH_KEY" "$SSH_USER@$TARGET_SERVER" "sudo journalctl -u aurelia -n 10 --no-pager"
else
    log_error "❌ Service failed to start"
    ssh -i "$SSH_KEY" "$SSH_USER@$TARGET_SERVER" "sudo journalctl -u aurelia -n 20 --no-pager"
    exit 1
fi

# Cleanup
cd /
rm -rf "$TEMP_DIR"

echo ""
echo "======================================"
echo "  Deployment Complete"
echo "======================================"
echo ""
echo "Access the system:"
echo "  SSH: ssh -i $SSH_KEY $SSH_USER@$TARGET_SERVER"
echo "  Logs: ssh -i $SSH_KEY $SSH_USER@$TARGET_SERVER 'sudo journalctl -u aurelia -f'"
echo "  Status: ssh -i $SSH_KEY $SSH_USER@$TARGET_SERVER 'sudo systemctl status aurelia'"
echo ""