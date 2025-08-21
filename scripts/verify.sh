source .env

forge verify-contract --compiler-version "v0.8.27+commit.40a35a09" "${OPENRANK_MANAGER_ADDRESS}" contracts/src/OpenRankManager.sol:OpenRankManager --show-standard-json-input > standard-json-input.json
