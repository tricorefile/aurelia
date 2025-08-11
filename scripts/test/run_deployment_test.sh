#!/bin/bash

# Aurelia Deployment Test Script
# Run from project root: ./scripts/test/run_deployment_test.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CONFIG_DIR="${SCRIPT_DIR}/../config"

cd "${PROJECT_ROOT}"

echo "==================================="
echo "Aurelia Deployment Test"
echo "==================================="

# Check for config file argument
CONFIG_FILE="${1:-${CONFIG_DIR}/test_config.json}"

if [ ! -f "${CONFIG_FILE}" ]; then
    echo "❌ Configuration file not found: ${CONFIG_FILE}"
    echo "Usage: $0 [config_file]"
    echo "Default: ${CONFIG_DIR}/test_config.json"
    exit 1
fi

echo "Using configuration: ${CONFIG_FILE}"
echo ""

# Build the project
echo "Building Aurelia..."
if ! cargo build --release; then
    echo "❌ Build failed"
    exit 1
fi
echo "✅ Build successful"

# Run deployment tests
echo ""
echo "Available test commands:"
echo "  1. full       - Run complete test suite"
echo "  2. connection - Test SSH connections only"
echo "  3. deploy     - Deploy agents to servers"
echo "  4. replication- Test self-replication"
echo "  5. validate   - Run validation tests"
echo "  6. monitor    - Start continuous monitoring"
echo "  7. cleanup    - Clean up deployments"
echo ""

# Use cargo to run the deployment tester
cargo run --example run_test -- --config "${CONFIG_FILE}" "${2:-full}"