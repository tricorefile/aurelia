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
    # Check token type by prefix
    # PAT tokens: ghp_* (classic) or github_pat_* (fine-grained)
    # Registration tokens: Usually start with A* and are longer
    if [[ "$GITHUB_TOKEN" == ghp_* || "$GITHUB_TOKEN" == github_pat_* ]]; then
        echo "Detected Personal Access Token (PAT)" >&2
        echo "Getting registration token from GitHub API..." >&2
        
        # First verify the PAT works
        echo "Verifying PAT..." >&2
        AUTH_CHECK=$(curl -s -o /dev/null -w "%{http_code}" \
            -H "Authorization: token ${GITHUB_TOKEN}" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/user")
        
        if [[ "$AUTH_CHECK" != "200" ]]; then
            echo "Error: PAT authentication failed (HTTP $AUTH_CHECK)" >&2
            echo "Please check:" >&2
            echo "1. Your token is valid and not expired" >&2
            echo "2. Your token has the required permissions" >&2
            exit 1
        fi
        
        echo "PAT verified successfully" >&2
        
        # Get registration token - CORRECT API ENDPOINT
        API_URL="https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPOSITORY}/actions/runners/registration-token"
        echo "Requesting registration token from: $API_URL" >&2
        
        RESPONSE=$(curl -sX POST \
            -H "Authorization: token ${GITHUB_TOKEN}" \
            -H "Accept: application/vnd.github.v3+json" \
            "$API_URL")
        
        # Check if we got a token
        if echo "$RESPONSE" | grep -q '"token"'; then
            TOKEN=$(echo "$RESPONSE" | jq -r .token)
            if [[ -n "$TOKEN" ]] && [[ "$TOKEN" != "null" ]]; then
                echo "Registration token obtained successfully" >&2
                # Only output the token to stdout
                echo "$TOKEN"
                return 0
            fi
        fi
        
        # If we get here, something went wrong
        echo "Failed to get registration token" >&2
        echo "API Response: $RESPONSE" >&2
        
        if echo "$RESPONSE" | grep -q "404"; then
            echo "" >&2
            echo "ERROR: 404 Not Found - Possible causes:" >&2
            echo "1. Repository '${GITHUB_OWNER}/${GITHUB_REPOSITORY}' does not exist or is not accessible" >&2
            echo "2. Your PAT does not have admin access to the repository" >&2
            echo "3. Required PAT permissions:" >&2
            echo "   - For public repos: 'public_repo' scope" >&2
            echo "   - For private repos: 'repo' scope (all)" >&2
            echo "   - Admin access to the repository is required" >&2
            echo "" >&2
            echo "To fix:" >&2
            echo "1. Go to: https://github.com/settings/tokens" >&2
            echo "2. Create a new token with 'repo' scope" >&2
            echo "3. Make sure you have admin access to ${GITHUB_OWNER}/${GITHUB_REPOSITORY}" >&2
        elif echo "$RESPONSE" | grep -q "403"; then
            echo "" >&2
            echo "ERROR: 403 Forbidden - Your token lacks required permissions" >&2
        elif echo "$RESPONSE" | grep -q "401"; then
            echo "" >&2
            echo "ERROR: 401 Unauthorized - Your token is invalid or expired" >&2
        fi
        
        exit 1
    elif [[ "$GITHUB_TOKEN" == A* && ${#GITHUB_TOKEN} -gt 40 ]]; then
        # Registration tokens typically start with A and are longer than 40 chars
        echo "Detected Registration Token (will expire in 1 hour)" >&2
        # Only output the token to stdout
        echo "$GITHUB_TOKEN"
    else
        # Unknown format, try to use as-is
        echo "Token format unknown, attempting to use as registration token" >&2
        # Only output the token to stdout
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