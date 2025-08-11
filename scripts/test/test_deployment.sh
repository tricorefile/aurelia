#!/bin/bash

# Aurelia Agent Self-Replication Test Script
# This script tests the agent's ability to deploy itself to remote Ubuntu servers

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_ENV_FILE="${SCRIPT_DIR}/test_env.json"
LOG_FILE="${SCRIPT_DIR}/test_deployment.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "${LOG_FILE}"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "${LOG_FILE}"
    exit 1
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "${LOG_FILE}"
}

# Step 1: Pre-deployment checks
pre_deployment_check() {
    log "Starting pre-deployment checks..."
    
    # Check if test environment file exists
    if [ ! -f "${TEST_ENV_FILE}" ]; then
        error "Test environment file not found: ${TEST_ENV_FILE}"
    fi
    
    # Check if kernel binary exists
    if [ ! -f "${SCRIPT_DIR}/target/release/kernel" ]; then
        warning "Release binary not found, building..."
        cargo build --release
    fi
    
    # Parse test servers from JSON
    SERVERS=$(jq -r '.test_environments[] | .ip' "${TEST_ENV_FILE}")
    
    # Test SSH connectivity to each server
    for server in ${SERVERS}; do
        log "Testing SSH connection to ${server}..."
        ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no ubuntu@${server} "echo 'SSH connection successful'" || \
            error "Cannot connect to ${server}"
    done
    
    log "Pre-deployment checks completed successfully"
}

# Step 2: Build and prepare deployment package
prepare_deployment() {
    log "Preparing deployment package..."
    
    # Create deployment directory
    DEPLOY_DIR="${SCRIPT_DIR}/deploy_package"
    rm -rf "${DEPLOY_DIR}"
    mkdir -p "${DEPLOY_DIR}"
    
    # Copy necessary files
    cp "${SCRIPT_DIR}/target/release/kernel" "${DEPLOY_DIR}/"
    cp -r "${SCRIPT_DIR}/config" "${DEPLOY_DIR}/"
    
    # Create .env file for deployment
    cat > "${DEPLOY_DIR}/.env" <<EOF
BINANCE_API_KEY=test_api_key
BINANCE_API_SECRET=test_api_secret
DEPLOYMENT_MODE=test
EOF
    
    # Create startup script
    cat > "${DEPLOY_DIR}/start_agent.sh" <<'EOF'
#!/bin/bash
cd "$(dirname "$0")"
nohup ./kernel > aurelia.log 2>&1 &
echo $! > aurelia.pid
echo "Agent started with PID: $(cat aurelia.pid)"
EOF
    chmod +x "${DEPLOY_DIR}/start_agent.sh"
    
    # Create monitoring script
    cat > "${DEPLOY_DIR}/monitor_agent.sh" <<'EOF'
#!/bin/bash
if [ -f aurelia.pid ]; then
    PID=$(cat aurelia.pid)
    if ps -p $PID > /dev/null; then
        echo "Agent is running (PID: $PID)"
        ps aux | grep $PID | grep -v grep
        echo "Memory usage:"
        pmap $PID | tail -1
        echo "Recent logs:"
        tail -n 20 aurelia.log
    else
        echo "Agent is not running"
        exit 1
    fi
else
    echo "PID file not found"
    exit 1
fi
EOF
    chmod +x "${DEPLOY_DIR}/monitor_agent.sh"
    
    log "Deployment package prepared"
}

# Step 3: Deploy to test servers
deploy_to_servers() {
    log "Starting deployment to test servers..."
    
    SERVERS=$(jq -c '.test_environments[]' "${TEST_ENV_FILE}")
    
    echo "${SERVERS}" | while IFS= read -r server_config; do
        IP=$(echo "${server_config}" | jq -r '.ip')
        USER=$(echo "${server_config}" | jq -r '.user')
        REMOTE_PATH=$(echo "${server_config}" | jq -r '.remote_deploy_path')
        ROLE=$(echo "${server_config}" | jq -r '.role')
        
        log "Deploying to ${IP} (${ROLE})..."
        
        # Create remote directory
        ssh ${USER}@${IP} "mkdir -p ${REMOTE_PATH}"
        
        # Copy deployment package
        scp -r "${DEPLOY_DIR}"/* ${USER}@${IP}:${REMOTE_PATH}/
        
        # Start the agent
        ssh ${USER}@${IP} "cd ${REMOTE_PATH} && ./start_agent.sh"
        
        log "Deployment to ${IP} completed"
    done
}

# Step 4: Test self-replication capability
test_self_replication() {
    log "Testing self-replication capability..."
    
    PRIMARY_SERVER=$(jq -r '.test_environments[] | select(.role=="primary") | .ip' "${TEST_ENV_FILE}")
    REPLICA_SERVER=$(jq -r '.test_environments[] | select(.role=="replica") | .ip' "${TEST_ENV_FILE}")
    
    # Create deployment trigger file on primary
    log "Triggering self-replication from primary to replica..."
    
    DEPLOY_INFO=$(cat <<EOF
{
    "ip": "${REPLICA_SERVER}",
    "remote_user": "ubuntu",
    "private_key_path": "~/.ssh/id_rsa",
    "remote_path": "/home/ubuntu/aurelia_replica",
    "local_exe_path": "./kernel"
}
EOF
)
    
    # Send deployment event to primary agent
    ssh ubuntu@${PRIMARY_SERVER} "echo '${DEPLOY_INFO}' > /home/ubuntu/aurelia_agent/deploy_trigger.json"
    
    # Wait for replication
    sleep 30
    
    # Verify replica deployment
    log "Verifying replica deployment..."
    ssh ubuntu@${REPLICA_SERVER} "test -f /home/ubuntu/aurelia_replica/kernel" || \
        error "Self-replication failed - kernel not found on replica"
    
    log "Self-replication test passed"
}

# Step 5: Monitor and validate
monitor_deployment() {
    log "Starting monitoring phase..."
    
    DURATION=$(jq -r '.test_config.test_duration_minutes' "${TEST_ENV_FILE}")
    INTERVAL=$(jq -r '.test_config.health_check_interval_seconds' "${TEST_ENV_FILE}")
    
    END_TIME=$(($(date +%s) + ${DURATION} * 60))
    
    while [ $(date +%s) -lt ${END_TIME} ]; do
        log "Performing health checks..."
        
        SERVERS=$(jq -r '.test_environments[] | .ip' "${TEST_ENV_FILE}")
        for server in ${SERVERS}; do
            log "Checking ${server}..."
            ssh ubuntu@${server} "cd /home/ubuntu/aurelia_agent && ./monitor_agent.sh" || \
                warning "Health check failed for ${server}"
        done
        
        sleep ${INTERVAL}
    done
    
    log "Monitoring phase completed"
}

# Step 6: Cleanup
cleanup() {
    log "Cleaning up test deployment..."
    
    SERVERS=$(jq -r '.test_environments[] | .ip' "${TEST_ENV_FILE}")
    for server in ${SERVERS}; do
        log "Cleaning up ${server}..."
        ssh ubuntu@${server} "pkill -f kernel || true; rm -rf /home/ubuntu/aurelia_*" || \
            warning "Cleanup failed for ${server}"
    done
    
    rm -rf "${DEPLOY_DIR}"
    log "Cleanup completed"
}

# Main execution
main() {
    log "=== Starting Aurelia Agent Deployment Test ==="
    
    # Run test steps
    pre_deployment_check
    prepare_deployment
    deploy_to_servers
    test_self_replication
    monitor_deployment
    
    log "=== All tests completed successfully ==="
    
    # Optional cleanup
    read -p "Do you want to clean up the test deployment? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cleanup
    fi
}

# Trap for cleanup on error
trap 'error "Test failed! Check ${LOG_FILE} for details"' ERR

# Run main function
main "$@"