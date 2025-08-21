#!/bin/bash
# Fix GitHub Runner update issue where Runner.Listener is missing

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  GitHub Runner Update Fix${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if running in container
if [ -f /.dockerenv ]; then
    echo -e "${GREEN}Detected Docker environment${NC}"
    RUNNER_DIR="/home/runner"
else
    echo "Enter runner directory path (default: /home/runner):"
    read -r RUNNER_PATH
    RUNNER_DIR="${RUNNER_PATH:-/home/runner}"
fi

cd "$RUNNER_DIR"

echo -e "${YELLOW}Current directory contents:${NC}"
ls -la

# Check for versioned directories
echo ""
echo -e "${YELLOW}Checking for versioned directories...${NC}"
VERSIONED_BINS=$(ls -d bin.* 2>/dev/null || true)
VERSIONED_EXTERNALS=$(ls -d externals.* 2>/dev/null || true)

if [ -n "$VERSIONED_BINS" ]; then
    echo "Found versioned bin directories:"
    echo "$VERSIONED_BINS"
    
    # Get latest version
    LATEST_BIN=$(ls -d bin.* | sort -V | tail -n 1)
    echo -e "${GREEN}Latest bin version: $LATEST_BIN${NC}"
    
    # Fix symlinks
    echo -e "${YELLOW}Fixing bin symlink...${NC}"
    rm -f bin
    ln -s "$LATEST_BIN" bin
    echo -e "${GREEN}✓ Created symlink: bin -> $LATEST_BIN${NC}"
fi

if [ -n "$VERSIONED_EXTERNALS" ]; then
    echo "Found versioned externals directories:"
    echo "$VERSIONED_EXTERNALS"
    
    # Get latest version
    LATEST_EXTERNALS=$(ls -d externals.* | sort -V | tail -n 1)
    echo -e "${GREEN}Latest externals version: $LATEST_EXTERNALS${NC}"
    
    # Fix symlinks
    echo -e "${YELLOW}Fixing externals symlink...${NC}"
    rm -f externals
    ln -s "$LATEST_EXTERNALS" externals
    echo -e "${GREEN}✓ Created symlink: externals -> $LATEST_EXTERNALS${NC}"
fi

# Verify Runner.Listener exists
echo ""
echo -e "${YELLOW}Verifying Runner.Listener...${NC}"
if [ -f "bin/Runner.Listener" ]; then
    echo -e "${GREEN}✓ Runner.Listener found${NC}"
    ls -la bin/Runner.Listener
else
    echo -e "${RED}✗ Runner.Listener still missing${NC}"
    echo ""
    echo -e "${YELLOW}Attempting alternative fix...${NC}"
    
    # Check if we have the runner package
    if [ -f "bin.2.328.0/Runner.Listener" ]; then
        echo "Found Runner.Listener in bin.2.328.0"
        rm -rf bin
        ln -s bin.2.328.0 bin
        rm -rf externals
        ln -s externals.2.328.0 externals
        echo -e "${GREEN}✓ Fixed symlinks to version 2.328.0${NC}"
    elif [ -f "bin.2.311.0/Runner.Listener" ]; then
        echo "Found Runner.Listener in bin.2.311.0"
        rm -rf bin
        ln -s bin.2.311.0 bin
        rm -rf externals
        ln -s externals.2.311.0 externals
        echo -e "${GREEN}✓ Reverted to version 2.311.0${NC}"
    else
        echo -e "${RED}Cannot find Runner.Listener in any version directory${NC}"
        echo "Manual reinstall required"
        exit 1
    fi
fi

# Set correct permissions
echo ""
echo -e "${YELLOW}Setting correct permissions...${NC}"
chmod +x bin/Runner.Listener 2>/dev/null || true
chmod +x bin/Runner.Worker 2>/dev/null || true
chmod +x bin/Runner.PluginHost 2>/dev/null || true
chmod +x run.sh 2>/dev/null || true
chmod +x run-helper.sh 2>/dev/null || true

echo -e "${GREEN}✓ Permissions set${NC}"

# Final verification
echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Verification${NC}"
echo -e "${BLUE}========================================${NC}"

if [ -f "bin/Runner.Listener" ] && [ -x "bin/Runner.Listener" ]; then
    echo -e "${GREEN}✓ Runner.Listener is ready${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Exit this container/shell"
    echo "2. Restart the runner:"
    echo "   docker compose restart"
    echo "   OR"
    echo "   ./run.sh"
else
    echo -e "${RED}✗ Runner.Listener still not functional${NC}"
    echo ""
    echo "Recommended: Perform clean reinstall"
    echo "1. Remove runner: ./config.sh remove"
    echo "2. Delete and recreate container"
fi