# RandomnessBeacon Poster

A TypeScript application that generates cryptographically secure random uint128 values and posts them to a RandomnessBeacon smart contract on the Sepolia testnet.

## Setup

1. Install dependencies:
```bash
npm install
```

2. Create a `.env` file with the required environment variables:
```bash
MNEMONIC="your twelve word mnemonic phrase goes here and should be kept secret always"
RANDOMNESS_BEACON_ADDRESS="0x1234567890123456789012345678901234567890"
RPC_URL="https://sepolia.infura.io/v3/your-project-id"
```

## Environment Variables

- **MNEMONIC** (required): Your 12 or 24-word mnemonic phrase for the wallet that will post randomness
- **RANDOMNESS_BEACON_ADDRESS** (required): The deployed RandomnessBeacon contract address on Sepolia
- **RPC_URL** (optional): Sepolia RPC endpoint URL (defaults to `http://localhost:8545` for local development)

## Usage

### Development Mode
Run with TypeScript directly (no build step required):
```bash
npm run dev
```

### Production Mode
Build the project and run the compiled JavaScript:
```bash
npm run build
npm start
```

## What it does

1. **Reads configuration**: Loads environment variables for mnemonic, contract address, and RPC URL
2. **Derives account**: Uses viem to derive an Ethereum account from the mnemonic
3. **Connects to Sepolia**: Establishes connection to the Sepolia testnet
4. **Generates random uint128**: Creates cryptographically secure 128-bit random numbers using Node.js crypto
5. **Posts to contract**: Calls the `postRandomness(uint128)` function on the RandomnessBeacon contract
6. **Waits for confirmation**: Monitors transaction confirmation and logs gas usage
7. **Repeats continuously**: Posts new randomness every 5 seconds

## Output Example

```
Initializing mnemonic signer...
RandomnessBeacon address: 0x1234567890123456789012345678901234567890
RPC URL: https://sepolia.infura.io/v3/your-project-id
Account address: 0xabcdef1234567890abcdef1234567890abcdef12
Starting random number generation and posting to chain...

---
Timestamp: 2024-01-15T10:30:45.123Z
Random uint128: 123456789012345678901234567890123456789
Random uint128 (hex): 0x5d2e8f4a9b7c1e3f6d8a2c4e7b9f1a3d
Posting to RandomnessBeacon at 0x1234567890123456789012345678901234567890...
Transaction hash: 0xabc123def456789...
Transaction confirmed in block: 1234567
Gas used: 45678
Signer Address: 0xabcdef1234567890abcdef1234567890abcdef12
```

## RandomnessBeacon Contract

The application interacts with a RandomnessBeacon smart contract that:
- Has a `postRandomness(uint128 randomness)` function
- Is restricted to the contract owner
- Stores randomness with timestamps in a struct array
- Emits `RandomnessPosted` events for each submission

## Dependencies

- **viem**: Modern Ethereum library for blockchain interaction and account management
- **dotenv**: Environment variable management
- **TypeScript**: Type-safe JavaScript development

## Troubleshooting

- **"Only owner" errors**: Ensure the wallet derived from your mnemonic is the owner of the RandomnessBeacon contract
- **Gas estimation failures**: Check that you have sufficient Sepolia ETH for transaction fees
- **Connection issues**: Verify your RPC_URL is correct and accessible
- **Contract not found**: Confirm the RANDOMNESS_BEACON_ADDRESS is correctly deployed on Sepolia

## Stopping the Application

Press `Ctrl+C` to stop the application gracefully. 