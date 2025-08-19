#!/bin/bash
set -e

# Configuration
GITHUB_OWNER=${GITHUB_OWNER:-tricorefile}
GITHUB_REPOSITORY=${GITHUB_REPOSITORY:-aurelia}
RUNNER_NAME=${RUNNER_NAME:-docker-runner-$(hostname)}
RUNNER_LABELS=${RUNNER_LABELS:-self-hosted,linux,x64,docker,aurelia}
RUNNER_WORKDIR=${RUNNER_WORKDIR:-_work}

# Debug output
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
    echo "Please provide a personal access token or registration token"
    exit 1
fi

# Determine token type
TOKEN_LENGTH=${#GITHUB_TOKEN}
echo "Token length: $TOKEN_LENGTH characters"

# Function to get registration token from PAT
get_registration_token() {
    # Check if this is already a registration token
    # Registration tokens are typically very long (>100 chars) and contain 'A' patterns
    if [[ ${#GITHUB_TOKEN} -gt 100 ]] && [[ "$GITHUB_TOKEN" == *"AA"* ]]; then
        echo "Detected registration token (length: ${#GITHUB_TOKEN})"
        echo "$GITHUB_TOKEN"
    # Check if this is a classic PAT (starts with ghp_)
    elif [[ "$GITHUB_TOKEN" == ghp_* ]]; then
        echo "Detected Personal Access Token (PAT)"
        echo "Getting registration token from GitHub API..."
        
        # First, verify the PAT is valid
        echo "Verifying PAT..."
        AUTH_CHECK=$(curl -s -o /dev/null -w "%{http_code}" \
            -H "Authorization: token ${GITHUB_TOKEN}" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/user")
        
        if [[ "$AUTH_CHECK" != "200" ]]; then
            echo "Error: PAT authentication failed (HTTP $AUTH_CHECK)"
            echo "Please check your GitHub token permissions"
            exit 1
        fi
        
        echo "PAT verified successfully"
        
        # Get registration token
        API_URL="https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPOSITORY}/actions/runners/registration-token"
        echo "Requesting registration token from: $API_URL"
        
        RESPONSE=$(curl -sX POST \
            -H "Authorization: token ${GITHUB_TOKEN}" \
            -H "Accept: application/vnd.github.v3+json" \
            "$API_URL")
        
        # Debug output
        echo "API Response: $RESPONSE"
        
        TOKEN=$(echo "$RESPONSE" | jq -r .token)
        
        if [[ "$TOKEN" == "null" ]] || [[ -z "$TOKEN" ]]; then
            echo "Failed to get registration token"
            echo "Full response: $RESPONSE"
            
            # Check if it's a permission issue
            if [[ "$RESPONSE" == *"404"* ]] || [[ "$RESPONSE" == *"Not Found"* ]]; then
                echo ""
                echo "ERROR: 404 Not Found - Possible causes:"
                echo "1. Repository '${GITHUB_OWNER}/${GITHUB_REPOSITORY}' does not exist"
                echo "2. PAT does not have 'admin' permission on the repository"
                echo "3. PAT does not have 'Actions' permission"
                echo ""
                echo "Required PAT permissions:"
                echo "- Actions: Read"
                echo "- Administration: Read and Write"
                echo "- Metadata: Read"
            fi
            exit 1
        fi
        
        echo "Registration token obtained successfully"
        echo "$TOKEN"
    else
        # Unknown token format
        echo "Warning: Unknown token format, attempting to use as registration token"
        echo "$GITHUB_TOKEN"
    fi
}

# Function to cleanup runner
cleanup() {
    echo "Cleaning up runner..."
    if [[ -f ".runner" ]]; then
        # Try to get a removal token if we have a PAT
        if [[ "$GITHUB_TOKEN" == ghp_* ]]; then
            echo "Getting removal token..."
            REMOVE_TOKEN=$(curl -sX POST \
                -H "Authorization: token ${GITHUB_TOKEN}" \
                -H "Accept: application/vnd.github.v3+json" \
                "https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPOSITORY}/actions/runners/remove-token" \
                | jq -r .token)
            
            if [[ -n "$REMOVE_TOKEN" ]] && [[ "$REMOVE_TOKEN" != "null" ]]; then
                ./config.sh remove --token "${REMOVE_TOKEN}" || true
            fi
        else
            ./config.sh remove --token "${REG_TOKEN}" || true
        fi
    fi
}

# Set up cleanup trap
trap cleanup EXIT INT TERM

# Get registration token
echo "Obtaining registration token..."
REG_TOKEN=$(get_registration_token)

if [[ -z "$REG_TOKEN" ]]; then
    echo "Error: Failed to obtain registration token"
    exit 1
fi

echo "Registration token obtained (first 10 chars): ${REG_TOKEN:0:10}..."

# Configure the runner
echo ""
echo "Configuring GitHub Actions runner..."
echo "Repository: https://github.com/${GITHUB_OWNER}/${GITHUB_REPOSITORY}"
echo "Runner Name: ${RUNNER_NAME}"
echo "Labels: ${RUNNER_LABELS}"
echo ""

# Run config.sh with full error output
./config.sh \
    --url "https://github.com/${GITHUB_OWNER}/${GITHUB_REPOSITORY}" \
    --token "${REG_TOKEN}" \
    --name "${RUNNER_NAME}" \
    --labels "${RUNNER_LABELS}" \
    --work "${RUNNER_WORKDIR}" \
    --unattended \
    --replace

# Check if configuration was successful
if [[ ! -f ".runner" ]]; then
    echo "Error: Runner configuration failed"
    exit 1
fi

echo "Runner configured successfully"

# Start the runner
echo "Starting GitHub Actions runner..."
./run.sh

# Keep the container running if the runner exits
echo "Runner stopped. Container will stay alive for debugging."
tail -f /dev/null