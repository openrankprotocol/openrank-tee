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

# Handle devnet context
if [[ "$CONTEXT" == "devnet" ]]; then
    echo "═══════════════════════════════════════════════════════════════════"
    echo "  TEE AVS Keys - Context: devnet"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""
    echo "Key information is not available for local devnet instances."
    echo "Devnet uses local development keys that are generated at runtime."
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

# Call the API endpoint for AVS keys
API_URL="${COORDINATOR_API}/avs/${AVS_ADDRESS}/keys"
RESPONSE=$(curl -s "$API_URL")

# Check if curl was successful
if [[ $? -ne 0 ]]; then
    echo "Error: Failed to fetch keys data from $API_URL" >&2
    exit 1
fi

# Check if we got an error response
if echo "$RESPONSE" | jq -e '.error' >/dev/null 2>&1; then
    echo "Error: Failed to fetch keys for AVS address: $AVS_ADDRESS" >&2
    echo "The instance may still be provisioning. Please wait a few minutes and try again." >&2
    exit 1
fi

# Extract key information
echo "═══════════════════════════════════════════════════════════════════"
echo "  TEE AVS Keys - Context: $CONTEXT"
echo "═══════════════════════════════════════════════════════════════════"
echo ""
echo "AVS Configuration:"
echo "  AVS Address:     $(echo "$RESPONSE" | jq -r '.avs_address')"
echo "  Admin Address:   $(echo "$RESPONSE" | jq -r '.admin_address')"
echo "  Creator Address: $(echo "$RESPONSE" | jq -r '.creator_address')"
echo ""
echo "TEE-Generated Addresses (BIP39):"
echo "  EVM Address:     $(echo "$RESPONSE" | jq -r '.evm_address')"
echo "                   (m/44'/60'/0'/0/0)"
echo "  Solana Address:  $(echo "$RESPONSE" | jq -r '.svm_address')"
echo "                   (ed25519 from seed)"
echo ""
echo "  These addresses are derived from the TEE's internal BIP39 mnemonic."
echo "  Private keys are only accessible within the TEE via MNEMONIC env var."
echo ""
echo "Environment Encryption:"
echo "  Public Key:      $(echo "$RESPONSE" | jq -r '.public_key')"
echo "                   (AGE format)"
echo ""
echo "  Your .env.private variables are encrypted with this public key."
echo "  Only the TEE holds the private key needed to decrypt them."
echo ""
echo "Metadata:"
echo "  URI:             $(echo "$RESPONSE" | jq -r '.metadata_uri')"
echo "  Created:         $(echo "$RESPONSE" | jq -r '.created_at')"
echo "═══════════════════════════════════════════════════════════════════"

# Write key information to YAML file
OUTPUT_DIR="output"
mkdir -p "$OUTPUT_DIR"
OUTPUT_FILE="${OUTPUT_DIR}/tee-keys-${CONTEXT}.yaml"

cat > "$OUTPUT_FILE" << EOF
# TEE AVS Key Information - Context: $CONTEXT
# Generated: $(date -u '+%Y-%m-%d %H:%M:%S UTC')

avs:
  address: "$(echo "$RESPONSE" | jq -r '.avs_address')"
  admin_address: "$(echo "$RESPONSE" | jq -r '.admin_address')"
  creator_address: "$(echo "$RESPONSE" | jq -r '.creator_address')"

tee_generated_addresses:
  evm:
    address: "$(echo "$RESPONSE" | jq -r '.evm_address')"
    derivation_path: "m/44'/60'/0'/0/0"
  solana:
    address: "$(echo "$RESPONSE" | jq -r '.svm_address')"
    derivation: "ed25519 from seed"

encryption:
  public_key: "$(echo "$RESPONSE" | jq -r '.public_key')"
  format: "AGE"
  note: "Used to encrypt .env.private variables - only TEE holds private key"

metadata:
  uri: "$(echo "$RESPONSE" | jq -r '.metadata_uri')"
  created_at: "$(echo "$RESPONSE" | jq -r '.created_at')"

notes:
  - "Private keys are only accessible within the TEE via MNEMONIC environment variable"
  - "These addresses are derived from the TEE's internal BIP39 mnemonic"
  - ".env.private variables are encrypted with the public key above"
EOF

echo ""
echo "✅ Key information saved to: $OUTPUT_FILE"
