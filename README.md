# Aurelia - Autonomous Trading Agent System

[中文版](README_CN.md)

## Overview

Aurelia is an autonomous, self-replicating trading agent system built in Rust. It features distributed deployment capabilities, real-time monitoring, and adaptive strategy execution for cryptocurrency markets.

## Features

- **Autonomous Operation**: Self-managing agents with decision-making capabilities
- **Self-Replication**: Automatic deployment and scaling across multiple servers
- **Real-time Monitoring**: Comprehensive health checks and performance metrics
- **Adaptive Strategies**: Dynamic trading strategies with machine learning integration
- **Distributed Architecture**: Multi-node deployment with failover support
- **Security First**: SSH-based secure deployment with multiple authentication methods

## Architecture

The system consists of multiple interconnected modules:

- **Kernel**: Core runtime orchestrating all components
- **Autonomy Core**: Self-management and decision-making engine
- **Strategy Engine**: Trading strategy implementation
- **Execution Engine**: Order execution and management
- **Perception Core**: Market data collection and analysis
- **Monitoring Service**: Real-time system monitoring and alerting
- **Deployment Tester**: Automated testing and validation

## Quick Start

### Prerequisites

- Rust 1.70+ 
- Python 3.8+ (for monitoring dashboard)
- SSH access to deployment servers
- Binance API credentials (for live trading)

### Building

```bash
# Clone the repository
git clone https://github.com/tricorefile/aurelia.git
cd aurelia

# Build all components
cargo build --release

# Run tests
cargo test --all
```

### Configuration

1. Set up server configuration:
```bash
cp config/target_servers.json.example config/target_servers.json
# Edit with your server details
```

2. Configure environment variables:
```bash
cp .env.example .env
# Add your API keys
```

3. Set up SSH keys:
```bash
ssh-keygen -t rsa -b 4096 -f ~/.ssh/aurelia_deploy
```

### Deployment

Deploy to a single server:
```bash
./scripts/deploy.sh <server-ip>
```

Deploy to multiple servers:
```bash
python3 py/smart_deploy.py <server-ip> --tag latest
```

### Monitoring

Start the monitoring dashboard:
```bash
python3 py/api_monitor.py
# Access at http://localhost:3030
```

## Development

### Running Locally

```bash
# Start the kernel with monitoring
./scripts/start_with_monitor.sh

# Run specific components
cargo run --bin kernel
```

### Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_test

# Deployment tests
./scripts/test/run_deployment_test.sh
```

## CI/CD

The project uses GitHub Actions for continuous integration and deployment:

- **CI**: Runs on every push and pull request
- **Release**: Creates binaries for multiple platforms
- **Deploy**: Automatic deployment to configured servers

## Documentation

- [API Documentation](docs/API_DOCUMENTATION.md)
- [Deployment Guide](docs/DEPLOYMENT_GUIDE.md)
- [Server Configuration](docs/SERVER_CONFIG_GUIDE.md)
- [Monitoring Guide](docs/MONITORING_COMPARISON.md)
- [GitHub Actions Setup](docs/GITHUB_ACTIONS.md)

## Project Structure

```
aurelia/
├── kernel/              # Core runtime
├── autonomy_core/       # Autonomous agent logic
├── strategy_engine/     # Trading strategies
├── execution_engine/    # Order execution
├── monitoring_service/  # System monitoring
├── deployment_tester/   # Testing framework
├── scripts/            # Deployment scripts
├── py/                 # Python utilities
└── docs/              # Documentation
```

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Commit Message Convention

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting, etc.)
- `refactor:` Code refactoring
- `test:` Test additions or modifications
- `chore:` Build process or auxiliary tool changes
- `perf:` Performance improvements

## Security

- Never commit API keys or sensitive data
- Use SSH keys for server access
- Enable 2FA on all production servers
- Regular security audits and dependency updates

## License

This project is proprietary and confidential.

## Support

For issues and questions:
- GitHub Issues: [https://github.com/tricorefile/aurelia/issues](https://github.com/tricorefile/aurelia/issues)
- Documentation: [https://docs.aurelia.io](https://docs.aurelia.io)

## Status

![CI](https://github.com/tricorefile/aurelia/workflows/CI/badge.svg)
![Release](https://github.com/tricorefile/aurelia/workflows/Release/badge.svg)

---

Built with ❤️ using Rust