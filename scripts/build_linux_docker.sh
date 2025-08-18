#!/bin/bash

# Docker-based Linux binary build script

set -e

echo "======================================"
echo "  Building Linux Binary with Docker"
echo "======================================"
echo ""

# Build using Docker
echo "1. Building Linux binary in Docker..."
docker build -f Dockerfile.build --output=target/linux --target=export .

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Build successful!"
    echo "   Binary location: target/linux/kernel"
    
    # Check the binary
    echo ""
    echo "2. Checking binary info..."
    file target/linux/kernel
    ls -lh target/linux/kernel
    
    echo ""
    echo "======================================"
    echo "  Deployment Commands"
    echo "======================================"
    echo ""
    echo "1. Upload to Tencent Cloud:"
    echo "   scp target/linux/kernel ubuntu@106.54.1.130:~/aurelia/"
    echo ""
    echo "2. SSH to server:"
    echo "   ssh -i ~/.ssh/tencent.pem ubuntu@106.54.1.130"
    echo ""
    echo "3. Run on server:"
    echo "   cd ~/aurelia"
    echo "   chmod +x kernel"
    echo "   ./kernel"
else
    echo ""
    echo "❌ Build failed"
    echo "Check Docker installation and try again"
fi