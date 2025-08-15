#!/bin/bash

# Parse command line arguments
CONTEXT=""

# Check for --context flag or positional argument
if [[ "$1" == "--context" ]]; then
    CONTEXT="$2"
elif [[ -n "$1" ]]; then
    CONTEXT="$1"
fi

# Verify context was provided
if [[ -z "$CONTEXT" ]]; then
    echo "Error: No context provided. Usage: $0 <context> or $0 --context <context>" >&2
    exit 1
fi

# Handle devnet context differently (Docker containers)
if [[ "$CONTEXT" == "devnet" ]]; then
    PROJECT_NAME="tee-avs-devnet"
    
    echo "═══════════════════════════════════════════════════════════════════"
    echo "  TEE AVS Local Development Logs - Context: devnet"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""
    
    # Check if containers are running
    RUNNING_CONTAINERS=$(docker compose -p "$PROJECT_NAME" ps --format json 2>/dev/null || echo "")
    
    if [[ -z "$RUNNING_CONTAINERS" ]]; then
        echo "No devnet instance is currently running."
        echo ""
        echo "Start a devnet instance with: 'devkit avs start --context=devnet'"
        exit 0
    fi
    
    echo "Fetching logs from Docker Compose project: $PROJECT_NAME"
    echo "═══════════════════════════════════════════════════════════════════"
    docker compose -p "$PROJECT_NAME" logs --tail=100
    exit 0
fi

# Path to context config file
CONFIG_FILE="config/contexts/${CONTEXT}.yaml"

# Check if config file exists
if [[ ! -f "$CONFIG_FILE" ]]; then
    echo "Error: Context configuration file not found: $CONFIG_FILE" >&2
    exit 1
fi

# Extract avs.address from config
AVS_ADDRESS=$(yq eval '.context.avs.address // ""' "$CONFIG_FILE" 2>/dev/null)

# Check if avs.address was found
if [[ -z "$AVS_ADDRESS" ]]; then
    echo "Error: avs.address not found in $CONFIG_FILE" >&2
    echo "" >&2
    echo "Deploy your AVS first: 'devkit avs deploy contracts l1 --context=$CONTEXT'" >&2
    exit 1
fi

# Read coordinator API from compute-tee context file
TEE_CONTEXT_FILE=".compute-tee/context/${CONTEXT}.yaml"
if [[ ! -f "$TEE_CONTEXT_FILE" ]]; then
    echo "Error: TEE context file not found: $TEE_CONTEXT_FILE" >&2
    exit 1
fi

# Extract coordinator API URL
COORDINATOR_API=$(yq eval '.apis.coordinator // ""' "$TEE_CONTEXT_FILE" 2>/dev/null)
if [[ -z "$COORDINATOR_API" ]]; then
    echo "Error: apis.coordinator not found in $TEE_CONTEXT_FILE" >&2
    exit 1
fi

# Call the API endpoint for specific AVS instance
API_URL="${COORDINATOR_API}/instances/avs/${AVS_ADDRESS}"
RESPONSE=$(curl -s "$API_URL")

# Check if curl was successful
if [[ $? -ne 0 ]]; then
    echo "Error: Failed to fetch instance data from $API_URL" >&2
    exit 1
fi

# Check if we got an error response
if echo "$RESPONSE" | jq -e '.error' >/dev/null 2>&1; then
    INSTANCE_DATA=""
else
    INSTANCE_DATA="$RESPONSE"
fi

# Check if instance was found
if [[ -z "$INSTANCE_DATA" ]]; then
    echo "Error: No instance found with avs.address: $AVS_ADDRESS" >&2
    echo "" >&2
    echo "The instance may still be provisioning. Please wait a few minutes and try again." >&2
    exit 1
fi

# Extract public IP
PUBLIC_IP=$(echo "$INSTANCE_DATA" | jq -r '.public_ip')

# Check if public IP was found
if [[ -z "$PUBLIC_IP" ]] || [[ "$PUBLIC_IP" == "null" ]]; then
    echo "Error: No public IP found for instance" >&2
    exit 1
fi

# Fetch logs from the instance
echo "═══════════════════════════════════════════════════════════════════"
echo "  TEE AVS Instance Logs - Context: $CONTEXT"
echo "═══════════════════════════════════════════════════════════════════"
echo ""
echo "Instance: $(echo "$INSTANCE_DATA" | jq -r '.name')"
echo "Public IP: $PUBLIC_IP"
echo "AVS Address: $AVS_ADDRESS"
echo ""
echo "Fetching logs..."
echo "═══════════════════════════════════════════════════════════════════"

# Call the logs endpoint
LOGS_URL="http://${PUBLIC_IP}:31634/logs"
LOGS_RESPONSE=$(curl -s "$LOGS_URL")

# Check if curl was successful
if [[ $? -ne 0 ]]; then
    echo "Error: Failed to fetch logs from $LOGS_URL" >&2
    echo "The instance may not be fully initialized yet." >&2
    exit 1
fi

# Extract and display logs
LOGS=$(echo "$LOGS_RESPONSE" | jq -r '.data.logs // empty' 2>/dev/null)

if [[ -z "$LOGS" ]]; then
    echo "No logs available or unable to parse logs response." >&2
    echo "Response: $LOGS_RESPONSE" >&2
    exit 1
fi

echo "$LOGS"
