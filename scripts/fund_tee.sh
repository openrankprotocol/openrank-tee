source .env.private
source .env.public

cast send --private-key "$PRIVATE_KEY" \
  --rpc-url "https://eth-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}" \
  "${TEE_ADDRESS}" \
  --value=0.1ether
