# GitHub Actions Self-Hosted Runner

Docker-based self-hosted runner for the Aurelia project, following GitHub's official setup guide.

## Quick Start

### 1. Prerequisites
- Docker Engine 20.10+
- Docker Compose 2.0+
- GitHub Personal Access Token (PAT) or Registration Token

### 2. Get Your Token

#### Option A: Personal Access Token (Recommended)
1. Go to: https://github.com/settings/tokens/new
2. Select scopes:
   - `repo` (full control of private repositories)
   - `admin:org` (if organization repository)
3. Generate token (starts with `github_pat_` or `ghp_`)

#### Option B: Registration Token
1. Go to: https://github.com/tricorefile/aurelia/settings/actions/runners
2. Click "New self-hosted runner"
3. Copy the token from the Configure section

### 3. Setup and Run

```bash
# Clone repository
git clone https://github.com/tricorefile/aurelia.git
cd aurelia/runners

# Create .env file with your token
echo "GITHUB_TOKEN=your_token_here" > .env

# Build and start runner
docker compose build
docker compose up -d

# Check logs
docker compose logs -f
```

## Configuration Details

The setup follows GitHub's official guide with these components:

1. **Dockerfile**: Based on Ubuntu 22.04 with Runner v2.328.0
2. **entrypoint.sh**: Handles PAT → Registration Token conversion
3. **docker-compose.yml**: Simple single-runner configuration

### Environment Variables

Set in `.env` file:
- `GITHUB_TOKEN`: Your PAT or registration token (required)
- `GITHUB_OWNER`: Repository owner (default: tricorefile)
- `GITHUB_REPOSITORY`: Repository name (default: aurelia)
- `RUNNER_NAME`: Runner name (default: aurelia-runner)
- `RUNNER_LABELS`: Runner labels (default: self-hosted,linux,x64,docker,aurelia)

## Using in Workflows

```yaml
jobs:
  build:
    runs-on: self-hosted
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --release
```

## Management Commands

```bash
# Start runner
docker compose up -d

# Stop runner
docker compose down

# View logs
docker compose logs -f

# Rebuild after updates
docker compose build --no-cache
docker compose up -d

# Remove and re-register
docker compose down -v
docker compose up -d
```

## Troubleshooting

### Runner not registering?
1. Check token validity: Token must have admin access to the repository
2. Verify repository access: `curl -H "Authorization: token YOUR_TOKEN" https://api.github.com/user`
3. Check logs: `docker compose logs`

### Network issues?
- Ensure Docker can reach github.com
- Check firewall/proxy settings
- Try: `docker run --rm alpine ping -c 4 github.com`

### Permission errors?
- Ensure Docker socket is accessible: `ls -la /var/run/docker.sock`
- Check token permissions in GitHub settings

## Updates

To update the runner version:
1. Edit `docker-compose.yml` and change `RUNNER_VERSION`
2. Update hash in `Dockerfile` if needed
3. Rebuild: `docker compose build --no-cache`

## Security Notes

- Never commit `.env` file
- Rotate tokens regularly
- Use repository-specific tokens when possible
- Keep runner version updated

## References

- [GitHub Actions Self-Hosted Runners](https://docs.github.com/en/actions/hosting-your-own-runners)
- [Runner Releases](https://github.com/actions/runner/releases)