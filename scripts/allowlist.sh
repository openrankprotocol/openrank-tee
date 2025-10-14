source .env

forge script \
  --rpc-url "https://eth-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}" \
  --private-key "$PRIVATE_KEY" \
  contracts/script/AllowlistComputer.s.sol:AllowlistComputer \
  --broadcast \
  --verify \
  --etherscan-api-key "${ETHERSCAN_API_KEY}" \
