source .env.private
source .env.public
source .env

forge script \
  --rpc-url "https://eth-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}" \
  --private-key "$PRIVATE_KEY" \
  contracts/script/DeployOpenRank.s.sol:DeployOpenRank \
  --broadcast \
  --verify \
  --etherscan-api-key "${ETHERSCAN_API_KEY}" \
