#!/bin/bash
set -e

# Configuration
RUNNER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$RUNNER_DIR/docker-compose.yml"
ENV_FILE="$RUNNER_DIR/.env"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to check prerequisites
check_prerequisites() {
    print_message "$YELLOW" "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        print_message "$RED" "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        print_message "$RED" "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    # Check if .env file exists
    if [ ! -f "$ENV_FILE" ]; then
        print_message "$YELLOW" "Creating .env file from template..."
        cp "$RUNNER_DIR/.env.example" "$ENV_FILE"
        print_message "$RED" "Please edit $ENV_FILE and add your GitHub token"
        exit 1
    fi
    
    # Check if GitHub token is set
    source "$ENV_FILE"
    if [ -z "$GITHUB_TOKEN" ] || [ "$GITHUB_TOKEN" == "your_github_token_here" ]; then
        print_message "$RED" "Please set GITHUB_TOKEN in $ENV_FILE"
        exit 1
    fi
    
    print_message "$GREEN" "✓ Prerequisites check passed"
}

# Function to setup SSH keys
setup_ssh_keys() {
    print_message "$YELLOW" "Setting up SSH keys..."
    
    SSH_DIR="$RUNNER_DIR/ssh"
    mkdir -p "$SSH_DIR"
    
    # Check if deployment key exists
    if [ ! -f "$SSH_DIR/aurelia_deploy" ]; then
        print_message "$YELLOW" "Deployment SSH key not found at $SSH_DIR/aurelia_deploy"
        read -p "Do you want to generate a new SSH key? (y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            ssh-keygen -t rsa -b 4096 -f "$SSH_DIR/aurelia_deploy" -N "" -C "aurelia-runner@docker"
            print_message "$GREEN" "✓ SSH key generated at $SSH_DIR/aurelia_deploy"
            print_message "$YELLOW" "Add this public key to your deployment servers:"
            cat "$SSH_DIR/aurelia_deploy.pub"
        else
            print_message "$YELLOW" "Please copy your deployment SSH key to $SSH_DIR/aurelia_deploy"
            exit 1
        fi
    fi
    
    # Set correct permissions
    chmod 600 "$SSH_DIR/aurelia_deploy"
    chmod 644 "$SSH_DIR/aurelia_deploy.pub" 2>/dev/null || true
    
    print_message "$GREEN" "✓ SSH keys configured"
}

# Function to build Docker image
build_image() {
    print_message "$YELLOW" "Building Docker image..."
    
    cd "$RUNNER_DIR"
    docker-compose build --no-cache
    
    print_message "$GREEN" "✓ Docker image built successfully"
}

# Function to start runners
start_runners() {
    local mode=$1
    print_message "$YELLOW" "Starting runners in $mode mode..."
    
    cd "$RUNNER_DIR"
    
    case $mode in
        all)
            docker-compose up -d
            ;;
        prod)
            docker-compose up -d runner-1 runner-2
            ;;
        test)
            docker-compose up -d runner-test
            ;;
        single)
            docker-compose up -d runner-1
            ;;
        *)
            print_message "$RED" "Invalid mode: $mode"
            exit 1
            ;;
    esac
    
    # Wait for containers to start
    sleep 5
    
    # Check status
    docker-compose ps
    
    print_message "$GREEN" "✓ Runners started successfully"
}

# Function to stop runners
stop_runners() {
    print_message "$YELLOW" "Stopping runners..."
    
    cd "$RUNNER_DIR"
    docker-compose down
    
    print_message "$GREEN" "✓ Runners stopped"
}

# Function to show logs
show_logs() {
    local service=$1
    cd "$RUNNER_DIR"
    
    if [ -z "$service" ]; then
        docker-compose logs -f
    else
        docker-compose logs -f "$service"
    fi
}

# Function to show status
show_status() {
    print_message "$YELLOW" "Runner Status:"
    
    cd "$RUNNER_DIR"
    docker-compose ps
    
    echo
    print_message "$YELLOW" "GitHub Runner Registration Status:"
    
    # Check registration status via GitHub API
    source "$ENV_FILE"
    
    if [[ "$GITHUB_TOKEN" != *"AAAA"* ]] && [[ ${#GITHUB_TOKEN} -le 50 ]]; then
        RUNNERS=$(curl -s \
            -H "Authorization: token $GITHUB_TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/repos/tricorefile/aurelia/actions/runners")
        
        echo "$RUNNERS" | jq -r '.runners[] | "\(.name): \(.status)"'
    else
        print_message "$YELLOW" "Using registration token, cannot query runner status via API"
    fi
}

# Main script
main() {
    local command=${1:-help}
    
    case $command in
        setup)
            check_prerequisites
            setup_ssh_keys
            build_image
            print_message "$GREEN" "Setup completed! Run './deploy-runner.sh start' to start the runners"
            ;;
        start)
            check_prerequisites
            start_runners "${2:-all}"
            ;;
        stop)
            stop_runners
            ;;
        restart)
            stop_runners
            sleep 2
            start_runners "${2:-all}"
            ;;
        rebuild)
            check_prerequisites
            stop_runners
            build_image
            start_runners "${2:-all}"
            ;;
        logs)
            show_logs "$2"
            ;;
        status)
            show_status
            ;;
        help|*)
            echo "Usage: $0 {setup|start|stop|restart|rebuild|logs|status} [options]"
            echo ""
            echo "Commands:"
            echo "  setup              - Initial setup (check prerequisites, setup SSH, build image)"
            echo "  start [mode]       - Start runners (modes: all, prod, test, single)"
            echo "  stop               - Stop all runners"
            echo "  restart [mode]     - Restart runners"
            echo "  rebuild            - Rebuild image and restart runners"
            echo "  logs [service]     - Show logs (optionally for specific service)"
            echo "  status             - Show runner status"
            echo ""
            echo "Examples:"
            echo "  $0 setup           - Initial setup"
            echo "  $0 start prod      - Start production runners only"
            echo "  $0 logs runner-1   - Show logs for runner-1"
            ;;
    esac
}

# Run main function
main "$@"