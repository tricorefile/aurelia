# GitHub Actions Self-Hosted Runners for Aurelia

This directory contains the setup for self-hosted GitHub Actions runners that can be deployed using Docker.

## Features

- **Dockerized Runners**: Fully containerized GitHub Actions runners
- **Multi-Runner Support**: Run multiple runners for parallel job execution
- **Environment Separation**: Separate runners for production and testing
- **Persistent Caching**: Cargo and build caches persist across runs
- **Cross-Compilation Support**: Built-in support for multiple architectures
- **Automatic Registration**: Runners automatically register with GitHub
- **Easy Management**: Simple scripts for deployment and management

## Architecture

The setup includes:
- **Dockerfile**: Ubuntu 22.04 base with Rust, Docker CLI, and GitHub Actions runner
- **docker-compose.yml**: Multi-runner configuration with proper networking
- **entrypoint.sh**: Automatic runner registration and lifecycle management
- **deploy-runner.sh**: Management script for easy operations

## Prerequisites

1. **Docker and Docker Compose** installed on the host machine
2. **GitHub Personal Access Token (PAT)** or registration token
3. **SSH keys** for deployment to target servers (optional)

## Quick Start

### 1. Initial Setup

```bash
# Clone the repository
git clone https://github.com/tricorefile/aurelia.git
cd aurelia/runners

# Copy environment template
cp .env.example .env

# Edit .env and add your GitHub token
vim .env

# Run setup
chmod +x deploy-runner.sh
./deploy-runner.sh setup
```

### 2. Configure GitHub Token

Edit `.env` file and set your GitHub token:

```env
GITHUB_TOKEN=ghp_your_personal_access_token_here
```

#### Creating a GitHub PAT

1. Go to GitHub Settings → Developer settings → Personal access tokens → Fine-grained tokens
2. Click "Generate new token"
3. Set expiration and repository access (select `tricorefile/aurelia`)
4. Grant these permissions:
   - **Actions**: Read
   - **Administration**: Read and Write
   - **Metadata**: Read
5. Generate and copy the token

### 3. Start Runners

```bash
# Start all runners
./deploy-runner.sh start

# Start only production runners
./deploy-runner.sh start prod

# Start only test runner
./deploy-runner.sh start test

# Start single runner
./deploy-runner.sh start single
```

### 4. Verify Registration

Check that runners are registered:

```bash
# Check local status
./deploy-runner.sh status

# Check on GitHub
# Go to: https://github.com/tricorefile/aurelia/settings/actions/runners
```

## Runner Configuration

### Available Runners

1. **runner-1** (Production Runner 1)
   - Labels: `self-hosted, linux, x64, docker, aurelia, prod`
   - Container: `aurelia-runner-1`

2. **runner-2** (Production Runner 2)
   - Labels: `self-hosted, linux, x64, docker, aurelia, prod`
   - Container: `aurelia-runner-2`

3. **runner-test** (Test Runner)
   - Labels: `self-hosted, linux, x64, docker, aurelia, test`
   - Container: `aurelia-runner-test`

### Customizing Runners

Edit `docker-compose.yml` to:
- Add more runners
- Change labels
- Modify resource limits
- Adjust volume mounts

## Using Self-Hosted Runners in Workflows

### Basic Usage

```yaml
jobs:
  build:
    runs-on: [self-hosted, linux, x64, docker, aurelia, prod]
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --release
```

### Hybrid Approach (GitHub + Self-Hosted)

```yaml
jobs:
  test:
    runs-on: ${{ github.ref == 'refs/heads/main' && fromJSON('["self-hosted", "linux", "x64", "docker", "aurelia", "prod"]') || 'ubuntu-latest' }}
```

### Environment-Specific Runners

```yaml
jobs:
  deploy-prod:
    runs-on: [self-hosted, linux, x64, docker, aurelia, prod]
    
  deploy-test:
    runs-on: [self-hosted, linux, x64, docker, aurelia, test]
```

## Management Commands

```bash
# Setup (first time only)
./deploy-runner.sh setup

# Start runners
./deploy-runner.sh start [all|prod|test|single]

# Stop runners
./deploy-runner.sh stop

# Restart runners
./deploy-runner.sh restart [all|prod|test|single]

# Rebuild and restart
./deploy-runner.sh rebuild

# View logs
./deploy-runner.sh logs [runner-1|runner-2|runner-test]

# Check status
./deploy-runner.sh status
```

## Deployment to Server

### 1. Prepare the Server

```bash
# Install Docker on the target server
ssh user@your-server
curl -fsSL https://get.docker.com | bash
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

### 2. Deploy Runners

```bash
# Copy runner files to server
scp -r runners/ user@your-server:/opt/aurelia-runners/

# SSH to server and start runners
ssh user@your-server
cd /opt/aurelia-runners
./deploy-runner.sh setup
./deploy-runner.sh start prod
```

### 3. Configure as System Service (Optional)

Create `/etc/systemd/system/aurelia-runners.service`:

```ini
[Unit]
Description=Aurelia GitHub Actions Runners
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/aurelia-runners
ExecStart=/usr/local/bin/docker-compose up -d
ExecStop=/usr/local/bin/docker-compose down
User=runner
Group=docker

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable aurelia-runners
sudo systemctl start aurelia-runners
```

## Security Considerations

1. **Token Security**
   - Never commit `.env` file with tokens
   - Use GitHub Secrets for sensitive data
   - Rotate PATs regularly

2. **Network Security**
   - Runners should be in a secure network
   - Use firewall rules to restrict access
   - Consider VPN for sensitive deployments

3. **Container Security**
   - Regularly update base images
   - Scan images for vulnerabilities
   - Limit container capabilities

4. **SSH Key Management**
   - Store SSH keys securely
   - Use separate keys for different environments
   - Implement key rotation

## Troubleshooting

### Runner Not Registering

```bash
# Check logs
./deploy-runner.sh logs runner-1

# Verify token
curl -H "Authorization: token YOUR_TOKEN" \
     https://api.github.com/repos/tricorefile/aurelia
```

### Permission Issues

```bash
# Fix Docker socket permissions
sudo chmod 666 /var/run/docker.sock

# Fix SSH key permissions
chmod 600 runners/ssh/aurelia_deploy
```

### Container Crashes

```bash
# Check container status
docker ps -a

# Inspect container
docker inspect aurelia-runner-1

# View detailed logs
docker logs -f aurelia-runner-1
```

### Cleanup Old Runners

```bash
# Remove offline runners from GitHub
# Go to: Settings → Actions → Runners → Remove offline runners

# Clean up local containers
docker-compose down -v
docker system prune -a
```

## Performance Optimization

1. **Cache Optimization**
   - Persistent cargo cache reduces build times
   - Shared cache volumes between runners
   - Regular cache cleanup with `cargo-cache`

2. **Resource Allocation**
   ```yaml
   # In docker-compose.yml
   deploy:
     resources:
       limits:
         cpus: '2'
         memory: 4G
   ```

3. **Parallel Execution**
   - Use multiple runners for parallel jobs
   - Configure `max-parallel` in workflows
   - Balance load across runners

## Monitoring

### Check Runner Health

```bash
# Local health check
docker-compose ps
docker stats

# GitHub API check
curl -H "Authorization: token $GITHUB_TOKEN" \
     https://api.github.com/repos/tricorefile/aurelia/actions/runners
```

### Set Up Alerts

Consider using monitoring tools:
- Prometheus + Grafana for metrics
- Datadog or New Relic for APM
- Custom scripts for health checks

## Best Practices

1. **Regular Updates**
   - Update runner version in Dockerfile
   - Keep Rust toolchain current
   - Update dependencies regularly

2. **Backup Configuration**
   - Backup `.env` file securely
   - Version control workflow files
   - Document custom configurations

3. **Testing**
   - Test on separate runner before production
   - Use test labels for experimental workflows
   - Implement gradual rollouts

4. **Documentation**
   - Document custom configurations
   - Keep runbooks updated
   - Track changes in CHANGELOG

## Support

For issues or questions:
- Check [GitHub Actions documentation](https://docs.github.com/en/actions)
- Review [runner troubleshooting](https://docs.github.com/en/actions/hosting-your-own-runners/troubleshooting)
- Open an issue in the Aurelia repository