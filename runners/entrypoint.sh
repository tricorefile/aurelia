#!/bin/bash
set -e

# Configuration
GITHUB_OWNER=${GITHUB_OWNER:-tricorefile}
GITHUB_REPOSITORY=${GITHUB_REPOSITORY:-aurelia}
RUNNER_NAME=${RUNNER_NAME:-docker-runner-$(hostname)}
RUNNER_LABELS=${RUNNER_LABELS:-self-hosted,linux,x64,docker,aurelia}
RUNNER_WORKDIR=${RUNNER_WORKDIR:-_work}

echo "=========================================="
echo "GitHub Runner Configuration"
echo "=========================================="
echo "Owner: ${GITHUB_OWNER}"
echo "Repository: ${GITHUB_REPOSITORY}"
echo "Runner Name: ${RUNNER_NAME}"
echo "Labels: ${RUNNER_LABELS}"
echo "=========================================="

# Check required environment variables
if [[ -z "$GITHUB_TOKEN" ]]; then
    echo "Error: GITHUB_TOKEN environment variable is not set"
    exit 1
fi

# Function to get registration token from PAT
get_registration_token() {
    # Check if this is a PAT (starts with ghp_) or a registration token
    if [[ "$GITHUB_TOKEN" == ghp_* ]]; then
        echo "Detected Personal Access Token (PAT)"
        echo "Getting registration token from GitHub API..."
        
        # First verify the PAT works
        echo "Verifying PAT..."
        AUTH_CHECK=$(curl -s -o /dev/null -w "%{http_code}" \
            -H "Authorization: token ${GITHUB_TOKEN}" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/user")
        
        if [[ "$AUTH_CHECK" != "200" ]]; then
            echo "Error: PAT authentication failed (HTTP $AUTH_CHECK)"
            echo "Please check:"
            echo "1. Your token is valid and not expired"
            echo "2. Your token has the required permissions"
            exit 1
        fi
        
        echo "PAT verified successfully"
        
        # Get registration token - CORRECT API ENDPOINT
        API_URL="https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPOSITORY}/actions/runners/registration-token"
        echo "Requesting registration token from: $API_URL"
        
        RESPONSE=$(curl -sX POST \
            -H "Authorization: token ${GITHUB_TOKEN}" \
            -H "Accept: application/vnd.github.v3+json" \
            "$API_URL")
        
        # Check if we got a token
        if echo "$RESPONSE" | grep -q '"token"'; then
            TOKEN=$(echo "$RESPONSE" | jq -r .token)
            if [[ -n "$TOKEN" ]] && [[ "$TOKEN" != "null" ]]; then
                echo "Registration token obtained successfully"
                echo "$TOKEN"
                return 0
            fi
        fi
        
        # If we get here, something went wrong
        echo "Failed to get registration token"
        echo "API Response: $RESPONSE"
        
        if echo "$RESPONSE" | grep -q "404"; then
            echo ""
            echo "ERROR: 404 Not Found - Possible causes:"
            echo "1. Repository '${GITHUB_OWNER}/${GITHUB_REPOSITORY}' does not exist or is not accessible"
            echo "2. Your PAT does not have admin access to the repository"
            echo "3. Required PAT permissions:"
            echo "   - For public repos: 'public_repo' scope"
            echo "   - For private repos: 'repo' scope (all)"
            echo "   - Admin access to the repository is required"
            echo ""
            echo "To fix:"
            echo "1. Go to: https://github.com/settings/tokens"
            echo "2. Create a new token with 'repo' scope"
            echo "3. Make sure you have admin access to ${GITHUB_OWNER}/${GITHUB_REPOSITORY}"
        elif echo "$RESPONSE" | grep -q "403"; then
            echo ""
            echo "ERROR: 403 Forbidden - Your token lacks required permissions"
        elif echo "$RESPONSE" | grep -q "401"; then
            echo ""
            echo "ERROR: 401 Unauthorized - Your token is invalid or expired"
        fi
        
        exit 1
    else
        # Assume it's already a registration token
        echo "Using provided registration token directly"
        echo "$GITHUB_TOKEN"
    fi
}

# Get registration token
echo "Obtaining registration token..."
REG_TOKEN=$(get_registration_token)

if [[ -z "$REG_TOKEN" ]]; then
    echo "Error: Failed to obtain registration token"
    exit 1
fi

echo "Registration token obtained (length: ${#REG_TOKEN})"

# Configure the runner
echo ""
echo "Configuring GitHub Actions runner..."
echo "URL: https://github.com/${GITHUB_OWNER}/${GITHUB_REPOSITORY}"

# Check if config.sh exists
if [[ ! -f "./config.sh" ]]; then
    echo "Error: config.sh not found. Runner may not be properly installed."
    echo "Current directory: $(pwd)"
    echo "Files in directory:"
    ls -la
    exit 1
fi

# Run configuration
./config.sh \
    --url "https://github.com/${GITHUB_OWNER}/${GITHUB_REPOSITORY}" \
    --token "${REG_TOKEN}" \
    --name "${RUNNER_NAME}" \
    --labels "${RUNNER_LABELS}" \
    --work "${RUNNER_WORKDIR}" \
    --unattended \
    --replace \
    --ephemeral

# Check if configuration was successful
if [[ $? -ne 0 ]]; then
    echo "Error: Runner configuration failed"
    exit 1
fi

echo "Runner configured successfully"

# Start the runner
echo "Starting GitHub Actions runner..."
exec ./run.sh