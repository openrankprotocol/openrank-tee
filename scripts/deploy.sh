source .env.private
source .env.public
source .env

# cast send --private-key "$PRIVATE_KEY" \
#   --rpc-url "$CHAIN_RPC_URL" \
#   "$TEE_EVM_ADDRESS" \
#   --value=0.005ether

forge script \
  --rpc-url "$CHAIN_RPC_URL" \
  --private-key "$PRIVATE_KEY" \
  contracts/script/DeployOpenRank.s.sol:DeployOpenRank \
  --broadcast
