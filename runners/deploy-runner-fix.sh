#!/bin/bash
# Deploy GitHub Actions Runner with China optimizations

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Deploy GitHub Actions Runner (Fixed)${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if .env file exists
if [ ! -f .env ]; then
    echo -e "${YELLOW}Creating .env file...${NC}"
    echo -n "Enter your GitHub Token (ghp_...): "
    read GITHUB_TOKEN
    echo "GITHUB_TOKEN=$GITHUB_TOKEN" > .env
    echo -e "${GREEN}✓ .env file created${NC}"
else
    echo -e "${GREEN}✓ Found .env file${NC}"
fi

# Step 1: Fix Docker mirrors
echo ""
echo -e "${YELLOW}Step 1: Fixing Docker mirrors...${NC}"
if [ -f fix-docker-mirror.sh ]; then
    chmod +x fix-docker-mirror.sh
    ./fix-docker-mirror.sh
else
    echo -e "${YELLOW}fix-docker-mirror.sh not found, skipping...${NC}"
fi

# Step 2: Try different build strategies
echo ""
echo -e "${YELLOW}Step 2: Building runner image...${NC}"

BUILD_SUCCESS=false

# Strategy 1: Try with alternative Dockerfile (Aliyun base)
echo -e "${YELLOW}Trying alternative Dockerfile (Aliyun base image)...${NC}"
if [ -f Dockerfile.cn-alt ]; then
    if docker build -f Dockerfile.cn-alt -t aurelia-runner:latest .; then
        echo -e "${GREEN}✓ Build successful with alternative Dockerfile${NC}"
        BUILD_SUCCESS=true
    else
        echo -e "${RED}✗ Alternative Dockerfile build failed${NC}"
    fi
fi

# Strategy 2: Try with fixed Dockerfile
if [ "$BUILD_SUCCESS" = false ] && [ -f Dockerfile.cn-fixed ]; then
    echo -e "${YELLOW}Trying fixed Dockerfile...${NC}"
    if docker build -f Dockerfile.cn-fixed -t aurelia-runner:latest .; then
        echo -e "${GREEN}✓ Build successful with fixed Dockerfile${NC}"
        BUILD_SUCCESS=true
    else
        echo -e "${RED}✗ Fixed Dockerfile build failed${NC}"
    fi
fi

# Strategy 3: Try with standard China Dockerfile
if [ "$BUILD_SUCCESS" = false ] && [ -f Dockerfile.cn ]; then
    echo -e "${YELLOW}Trying standard China Dockerfile...${NC}"
    if docker build -f Dockerfile.cn -t aurelia-runner:latest .; then
        echo -e "${GREEN}✓ Build successful with standard Dockerfile${NC}"
        BUILD_SUCCESS=true
    else
        echo -e "${RED}✗ Standard Dockerfile build failed${NC}"
    fi
fi

# Strategy 4: Use pre-built image
if [ "$BUILD_SUCCESS" = false ]; then
    echo -e "${YELLOW}Using pre-built image from Docker Hub...${NC}"
    
    # Create simple docker-compose with pre-built image
    cat > docker-compose.prebuilt.yml << 'EOF'
version: '3.8'

services:
  runner:
    image: myoung34/github-runner:latest
    container_name: aurelia-runner
    environment:
      - REPO_URL=https://github.com/tricorefile/aurelia
      - RUNNER_NAME=prebuilt-runner-${HOSTNAME}
      - ACCESS_TOKEN=${GITHUB_TOKEN}
      - RUNNER_WORKDIR=/tmp/runner/work
      - LABELS=self-hosted,linux,x64,docker
      - RUNNER_GROUP=default
      - EPHEMERAL=true
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./work:/tmp/runner/work
    restart: unless-stopped
EOF
    
    echo -e "${YELLOW}Pulling pre-built image...${NC}"
    if docker compose -f docker-compose.prebuilt.yml pull; then
        echo -e "${GREEN}✓ Pre-built image pulled successfully${NC}"
        BUILD_SUCCESS=true
        USE_PREBUILT=true
    else
        echo -e "${RED}✗ Failed to pull pre-built image${NC}"
    fi
fi

# Step 3: Run the runner
if [ "$BUILD_SUCCESS" = true ]; then
    echo ""
    echo -e "${YELLOW}Step 3: Starting runner...${NC}"
    
    # Stop any existing runners
    docker stop aurelia-runner 2>/dev/null || true
    docker rm aurelia-runner 2>/dev/null || true
    
    if [ "$USE_PREBUILT" = true ]; then
        # Use pre-built image
        docker compose -f docker-compose.prebuilt.yml up -d
    else
        # Use custom built image
        docker run -d \
            --name aurelia-runner \
            --env-file .env \
            -e GITHUB_OWNER=tricorefile \
            -e GITHUB_REPOSITORY=aurelia \
            -e RUNNER_NAME=docker-runner-$(hostname) \
            -e RUNNER_LABELS=self-hosted,linux,x64,docker,aurelia \
            -v /var/run/docker.sock:/var/run/docker.sock \
            -v $(pwd)/work:/home/runner/_work \
            --restart unless-stopped \
            aurelia-runner:latest
    fi
    
    # Wait and check status
    sleep 5
    
    if docker ps | grep -q aurelia-runner; then
        echo -e "${GREEN}✓ Runner started successfully${NC}"
        echo ""
        echo "View logs:"
        echo "  docker logs -f aurelia-runner"
        echo ""
        echo "Check registration:"
        echo "  https://github.com/tricorefile/aurelia/settings/actions/runners"
    else
        echo -e "${RED}✗ Runner failed to start${NC}"
        echo "Check logs:"
        echo "  docker logs aurelia-runner"
    fi
else
    echo ""
    echo -e "${RED}========================================${NC}"
    echo -e "${RED}  Build Failed - Manual Steps Required${NC}"
    echo -e "${RED}========================================${NC}"
    echo ""
    echo "Option 1: Use offline installation"
    echo "  ./offline-setup.sh"
    echo ""
    echo "Option 2: Manual Docker image import"
    echo "  1. On a machine with Docker Hub access:"
    echo "     docker pull ubuntu:22.04"
    echo "     docker save ubuntu:22.04 -o ubuntu-22.04.tar"
    echo "  2. Transfer ubuntu-22.04.tar to this server"
    echo "  3. Load the image:"
    echo "     docker load -i ubuntu-22.04.tar"
    echo "  4. Re-run this script"
    echo ""
    echo "Option 3: Use a proxy"
    echo "  export HTTP_PROXY=http://your-proxy:port"
    echo "  export HTTPS_PROXY=http://your-proxy:port"
    echo "  docker build -f Dockerfile.cn -t aurelia-runner ."
fi

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Deployment Complete${NC}"
echo -e "${BLUE}========================================${NC}"