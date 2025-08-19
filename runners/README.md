# GitHub Actions Self-Hosted Runner

Simple Docker-based self-hosted runner for the Aurelia project.

## Quick Start

1. **Create .env file**:
```bash
echo "GITHUB_TOKEN=your_github_pat_token" > .env
```

2. **Build and run**:
```bash
docker compose build
docker compose up -d
```

3. **Check status**:
```bash
docker compose logs -f
```

## Configuration

Environment variables (in `.env` file):
- `GITHUB_TOKEN`: Your GitHub Personal Access Token with `repo` scope
- `GITHUB_OWNER`: Repository owner (default: tricorefile)
- `GITHUB_REPOSITORY`: Repository name (default: aurelia)
- `RUNNER_NAME`: Runner name (default: aurelia-runner)
- `RUNNER_LABELS`: Runner labels (default: self-hosted,linux,x64,docker,aurelia)

## Management

```bash
# Start runner
docker compose up -d

# Stop runner
docker compose down

# View logs
docker compose logs -f

# Rebuild image
docker compose build --no-cache

# Restart runner
docker compose restart
```

## Requirements

- Docker Engine 20.10+
- Docker Compose 2.0+
- GitHub Personal Access Token with admin access to the repository

## Troubleshooting

If the runner fails to register:
1. Check your GitHub token has the correct permissions
2. Verify the repository exists and you have admin access
3. Check the logs: `docker compose logs`

## Directory Structure

```
runners/
├── Dockerfile          # Runner image definition
├── docker-compose.yml  # Docker Compose configuration
├── entrypoint.sh      # Runner startup script
├── README.md          # This file
└── work/             # Runner work directory (created automatically)
```