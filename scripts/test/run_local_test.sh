#!/bin/bash

# Aurelia Autonomous Agent Local Test Script
# Run from project root: ./scripts/test/run_local_test.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${PROJECT_ROOT}"

echo "======================================"
echo "Aurelia Autonomous Agent Local Test"
echo "======================================"
echo ""

# Check if rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust."
    exit 1
fi

# Build the project
echo "🔨 Building Aurelia..."
if cargo build --release 2>&1 | grep -q "Finished"; then
    echo "✅ Build successful"
else
    echo "⚠️  Building (this may take a while)..."
    cargo build --release
fi

# Create necessary files
echo ""
echo "📝 Creating configuration files..."

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    cat > .env << EOF
BINANCE_API_KEY=test_api_key
BINANCE_API_SECRET=test_api_secret
DEPLOYMENT_MODE=test
EOF
    echo "✅ Created .env file"
fi

# Create strategy.json if it doesn't exist
if [ ! -f config/strategy.json ]; then
    mkdir -p config
    cat > config/strategy.json << EOF
{
    "strategy_type": "momentum",
    "symbol": "BTCUSDT",
    "interval": "1h",
    "lookback_periods": 20,
    "threshold": 0.02
}
EOF
    echo "✅ Created config/strategy.json"
fi

# Create state.json if it doesn't exist
if [ ! -f config/state.json ]; then
    cat > config/state.json << EOF
{
    "funds": 1000.0,
    "positions": {},
    "last_update": null
}
EOF
    echo "✅ Created config/state.json"
fi

echo ""
echo "🚀 Starting Aurelia Autonomous Agent..."
echo ""
echo "The agent will demonstrate:"
echo "  • Self-monitoring capabilities"
echo "  • Autonomous decision making"
echo "  • Task scheduling and execution"
echo "  • Health checks and recovery"
echo ""
echo "Press Ctrl+C to stop"
echo ""
echo "======================================"
echo ""

# Run the kernel
./target/release/kernel