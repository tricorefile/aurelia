#!/bin/bash

# Aurelia Docker Environment Test Script
# Run from project root: ./scripts/test/run_docker_test.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DOCKER_DIR="${SCRIPT_DIR}/../docker"

cd "${PROJECT_ROOT}"

echo "==================================="
echo "Aurelia Docker Environment Test"
echo "==================================="

# Step 1: Check Docker
echo ""
echo "Step 1: Checking Docker..."
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    echo "Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

if ! docker info &> /dev/null; then
    echo "❌ Docker is not running. Please start Docker."
    exit 1
fi
echo "✅ Docker is ready"

# Step 2: Build the project
echo ""
echo "Step 2: Building Aurelia..."
if cargo build --release 2>&1 | grep -q "Finished"; then
    echo "✅ Build successful"
else
    echo "❌ Build failed. Running full build..."
    cargo build --release
fi

# Step 3: Create deployment directories
echo ""
echo "Step 3: Creating deployment directories..."
mkdir -p deploy_primary deploy_replica1 deploy_replica2 deploy_monitor
echo "✅ Directories created"

# Step 4: Start Docker containers
echo ""
echo "Step 4: Starting test servers..."
cd "${DOCKER_DIR}"
docker-compose down 2>/dev/null || true
docker-compose up -d
cd "${PROJECT_ROOT}"
sleep 5
echo "✅ Test servers started"

# Step 5: Verify containers
echo ""
echo "Step 5: Verifying containers..."
for container in aurelia-primary aurelia-replica1 aurelia-replica2 aurelia-monitor; do
    if docker ps | grep -q $container; then
        echo "  ✅ $container is running"
    else
        echo "  ❌ $container failed to start"
        exit 1
    fi
done

# Step 6: Test SSH connectivity
echo ""
echo "Step 6: Testing SSH connectivity..."
for port in 2221 2222 2223; do
    echo "  Testing port $port..."
    if ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 -p $port root@localhost "echo 'Connected'" 2>/dev/null; then
        echo "  ✅ Port $port is accessible"
    else
        echo "  ⚠️  Port $port connection failed (SSH key may need setup)"
    fi
done

echo ""
echo "==================================="
echo "Test Environment Ready!"
echo "==================================="
echo ""
echo "Servers:"
echo "  Primary:  localhost:2221 (172.20.0.10)"
echo "  Replica1: localhost:2222 (172.20.0.11)"
echo "  Replica2: localhost:2223 (172.20.0.12)"
echo "  Monitor:  localhost:2224 (172.20.0.20)"
echo ""
echo "Deploy to primary:"
echo "  scp -P 2221 target/release/kernel root@localhost:/home/ubuntu/aurelia/"
echo ""
echo "Start agent:"
echo "  ssh -p 2221 root@localhost 'cd /home/ubuntu/aurelia && ./kernel'"
echo ""
echo "Monitor logs:"
echo "  docker logs -f aurelia-primary"
echo ""
echo "Stop environment:"
echo "  cd ${DOCKER_DIR} && docker-compose down"
echo ""