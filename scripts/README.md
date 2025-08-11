# Aurelia Scripts Directory

This directory contains all test-related scripts and configurations for the Aurelia autonomous agent system.

## Directory Structure

```
scripts/
├── test/                    # Test execution scripts
│   ├── run_local_test.sh   # Local testing without servers
│   ├── run_docker_test.sh  # Docker environment testing  
│   └── run_deployment_test.sh  # Full deployment testing
│
├── docker/                  # Docker configurations
│   └── docker-compose.yml  # Multi-container test environment
│
└── config/                  # Test configurations
    ├── test_config.json     # Default test configuration
    └── docker_config.json   # Docker-specific configuration
```

## Quick Start

### Local Testing
Run the autonomous agent locally without external dependencies:
```bash
./scripts/test/run_local_test.sh
```

### Docker Testing
Set up a multi-server test environment with Docker:
```bash
./scripts/test/run_docker_test.sh
```

### Deployment Testing
Run comprehensive deployment tests with SSH connectivity:
```bash
./scripts/test/run_deployment_test.sh [config_file]
```

## Test Scripts

### run_local_test.sh
- Creates necessary configuration files (.env, strategy.json, state.json)
- Builds the project in release mode
- Runs the kernel binary locally
- Demonstrates autonomous capabilities without external servers

### run_docker_test.sh
- Checks Docker installation and status
- Builds the Aurelia project
- Creates deployment directories
- Starts Docker containers (primary, replica1, replica2, monitor)
- Verifies container health and SSH connectivity
- Provides connection information for manual testing

### run_deployment_test.sh
- Accepts custom configuration files
- Builds the project
- Provides test command options:
  - `full` - Complete test suite
  - `connection` - SSH connection testing
  - `deploy` - Agent deployment
  - `replication` - Self-replication testing
  - `validate` - Validation tests
  - `monitor` - Continuous monitoring
  - `cleanup` - Clean up deployments

## Configuration Files

### test_config.json
Default configuration for testing environments:
- Server connection details (IP, port, SSH keys)
- Deployment paths
- Test settings (duration, intervals, thresholds)
- Resource limits

### docker_config.json
Docker-specific configuration:
- Container network settings
- Port mappings
- Volume mounts
- Resource constraints

## Docker Environment

The `docker-compose.yml` creates a test network with:
- **aurelia-primary** (172.20.0.10:2221) - Primary node
- **aurelia-replica1** (172.20.0.11:2222) - Replica node 1
- **aurelia-replica2** (172.20.0.12:2223) - Replica node 2
- **aurelia-monitor** (172.20.0.20:2224) - Monitoring node

All containers run Ubuntu 22.04 with SSH enabled for deployment testing.

## Usage Examples

### Run Full Test Suite
```bash
cd /path/to/aurelia
./scripts/test/run_deployment_test.sh scripts/config/test_config.json full
```

### Start Docker Environment Only
```bash
cd scripts/docker
docker-compose up -d
```

### Deploy to Docker Container
```bash
scp -P 2221 target/release/kernel root@localhost:/home/ubuntu/aurelia/
ssh -p 2221 root@localhost 'cd /home/ubuntu/aurelia && ./kernel'
```

### Monitor Container Logs
```bash
docker logs -f aurelia-primary
```

### Clean Up Docker Environment
```bash
cd scripts/docker
docker-compose down
```

## Notes

- All scripts should be run from the project root directory
- Docker tests require Docker to be installed and running
- SSH tests require proper SSH key configuration
- Configuration files support both local and remote server testing
- The autonomous agent will demonstrate self-monitoring, replication, and recovery capabilities