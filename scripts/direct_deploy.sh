#!/bin/bash

# Direct deployment script - builds locally and deploys directly

set -e

TARGET_SERVER="${1}"
SSH_KEY="${2:-$HOME/.ssh/tencent.pem}"
SSH_USER="${3:-ubuntu}"
DEPLOY_PATH="/opt/aurelia"

if [ -z "$TARGET_SERVER" ]; then
    echo "Usage: $0 <target_server> [ssh_key] [ssh_user]"
    echo "Example: $0 106.54.1.130 ~/.ssh/tencent.pem ubuntu"
    exit 1
fi

echo "======================================"
echo "  Direct Deployment (Local Build)"
echo "======================================"
echo ""
echo "Target: $SSH_USER@$TARGET_SERVER"
echo "Key: $SSH_KEY"
echo ""

# Step 1: Build locally
echo "[1/5] Building binary locally..."
cargo build --release --bin kernel

if [ ! -f "target/release/kernel" ]; then
    echo "Build failed!"
    exit 1
fi

# Step 2: Create temporary package
echo "[2/5] Creating deployment package..."
TEMP_DIR=$(mktemp -d)
cp target/release/kernel $TEMP_DIR/
cd $TEMP_DIR
tar czf aurelia-deploy.tar.gz kernel
cd -

# Step 3: Upload to server
echo "[3/5] Uploading to server..."
scp -i "$SSH_KEY" -o StrictHostKeyChecking=no \
    $TEMP_DIR/aurelia-deploy.tar.gz \
    "$SSH_USER@$TARGET_SERVER:/tmp/"

# Step 4: Deploy on server
echo "[4/5] Deploying on server..."
ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no "$SSH_USER@$TARGET_SERVER" << 'ENDSSH'
# Create directory
sudo mkdir -p /opt/aurelia
sudo chown $USER:$USER /opt/aurelia

# Stop existing service
sudo systemctl stop aurelia 2>/dev/null || true

# Extract new binary
cd /opt/aurelia
tar xzf /tmp/aurelia-deploy.tar.gz
chmod +x kernel

# Create systemd service if not exists
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

[Install]
WantedBy=multi-user.target
EOF
    sudo systemctl daemon-reload
fi

# Start service
sudo systemctl start aurelia
sudo systemctl enable aurelia
ENDSSH

# Step 5: Verify
echo "[5/5] Verifying deployment..."
sleep 3

ssh -i "$SSH_KEY" "$SSH_USER@$TARGET_SERVER" \
    "sudo systemctl is-active aurelia"

STATUS=$?
if [ $STATUS -eq 0 ]; then
    echo ""
    echo "✅ Deployment successful!"
    echo ""
    echo "View logs:"
    echo "  ssh -i $SSH_KEY $SSH_USER@$TARGET_SERVER 'sudo journalctl -u aurelia -f'"
else
    echo ""
    echo "❌ Service failed to start"
    ssh -i "$SSH_KEY" "$SSH_USER@$TARGET_SERVER" \
        "sudo journalctl -u aurelia -n 20 --no-pager"
fi

# Cleanup
rm -rf $TEMP_DIR