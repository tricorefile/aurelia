#!/bin/bash
set -e

# Configuration
GITHUB_OWNER=${GITHUB_OWNER:-tricorefile}
GITHUB_REPOSITORY=${GITHUB_REPOSITORY:-aurelia}
RUNNER_NAME=${RUNNER_NAME:-docker-runner-$(hostname)}
RUNNER_LABELS=${RUNNER_LABELS:-self-hosted,linux,x64,docker,aurelia}
RUNNER_WORKDIR=${RUNNER_WORKDIR:-_work}

# Check required environment variables
if [[ -z "$GITHUB_TOKEN" ]]; then
    echo "Error: GITHUB_TOKEN environment variable is not set"
    echo "Please provide a personal access token or registration token"
    exit 1
fi

# Function to get registration token from PAT
get_registration_token() {
    if [[ "$GITHUB_TOKEN" == *"AAAA"* ]] || [[ ${#GITHUB_TOKEN} -gt 50 ]]; then
        # This looks like a registration token, use it directly
        echo "$GITHUB_TOKEN"
    else
        # This is a PAT, get registration token
        echo "Getting registration token from GitHub API..."
        RESPONSE=$(curl -sX POST \
            -H "Authorization: token ${GITHUB_TOKEN}" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPOSITORY}/actions/runners/registration-token")
        
        TOKEN=$(echo "$RESPONSE" | jq -r .token)
        
        if [[ "$TOKEN" == "null" ]] || [[ -z "$TOKEN" ]]; then
            echo "Failed to get registration token. Response: $RESPONSE"
            exit 1
        fi
        
        echo "$TOKEN"
    fi
}

# Function to cleanup runner
cleanup() {
    echo "Cleaning up runner..."
    if [[ -f ".runner" ]]; then
        ./config.sh remove --token "${REG_TOKEN}" || true
    fi
}

# Set up cleanup trap
trap cleanup EXIT INT TERM

# Get registration token
REG_TOKEN=$(get_registration_token)

# Configure the runner
echo "Configuring GitHub Actions runner..."
echo "Repository: ${GITHUB_OWNER}/${GITHUB_REPOSITORY}"
echo "Runner Name: ${RUNNER_NAME}"
echo "Labels: ${RUNNER_LABELS}"

./config.sh \
    --url "https://github.com/${GITHUB_OWNER}/${GITHUB_REPOSITORY}" \
    --token "${REG_TOKEN}" \
    --name "${RUNNER_NAME}" \
    --labels "${RUNNER_LABELS}" \
    --work "${RUNNER_WORKDIR}" \
    --unattended \
    --replace

# Start the runner
echo "Starting GitHub Actions runner..."
./run.sh

# Keep the container running if the runner exits
echo "Runner stopped. Container will stay alive for debugging."
tail -f /dev/null