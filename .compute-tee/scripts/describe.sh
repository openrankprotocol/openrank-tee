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
    echo "  TEE AVS Local Development Instance - Context: devnet"
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
    
    echo "Docker Compose Project: $PROJECT_NAME"
    echo ""
    echo "Running Services:"
    docker compose -p "$PROJECT_NAME" ps --format "table {{.Service}}\t{{.Status}}\t{{.Ports}}"
    echo ""
    echo "Commands:"
    echo "  View logs:  docker compose -p $PROJECT_NAME logs -f"
    echo "  Stop:       docker compose -p $PROJECT_NAME down"
    echo "═══════════════════════════════════════════════════════════════════"
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

# Extract and format instance information
echo "═══════════════════════════════════════════════════════════════════"
echo "  TEE AVS Instance Information - Context: $CONTEXT"
echo "═══════════════════════════════════════════════════════════════════"
echo ""
echo "Instance Details:"
echo "  ID:           $(echo "$INSTANCE_DATA" | jq -r '.id')"
echo "  Name:         $(echo "$INSTANCE_DATA" | jq -r '.name')"
echo "  Request ID:   $(echo "$INSTANCE_DATA" | jq -r '.request_id')"
echo "  Status:       $(echo "$INSTANCE_DATA" | jq -r '.status')"
echo "  Machine Type: $(echo "$INSTANCE_DATA" | jq -r '.machine_type')"
echo "  Zone:         $(echo "$INSTANCE_DATA" | jq -r '.zone')"
echo "  Public IP:    $(echo "$INSTANCE_DATA" | jq -r '.public_ip')"
echo "  Created:      $(echo "$INSTANCE_DATA" | jq -r '.created_at')"
echo ""
echo "Addresses:"
echo "  AVS:          $(echo "$INSTANCE_DATA" | jq -r '.metadata."avs-address"')"
echo "  Creator:      $(echo "$INSTANCE_DATA" | jq -r '.metadata."creator-address"')"
echo "  Admin:        $(echo "$INSTANCE_DATA" | jq -r '.metadata."admin-address"')"
echo ""
echo "Metadata:"
echo "  URI:          $(echo "$INSTANCE_DATA" | jq -r '.metadata."metadata-uri"')"
echo "═══════════════════════════════════════════════════════════════════"
exit 0
