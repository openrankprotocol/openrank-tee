import 'dotenv/config';
import { mnemonicToAccount } from 'viem/accounts';
import { createPublicClient, createWalletClient, http, parseAbi } from 'viem';
import { sepolia } from 'viem/chains';
import { randomBytes } from 'crypto';

// RandomnessBeacon contract ABI - only the function we need
const RANDOMNESS_BEACON_ABI = parseAbi([
  'function postRandomness(uint128 randomness) public',
  'event RandomnessPosted((uint32 timestamp, uint128 randomness) randomness)'
]);

async function main() {
  // Check for required environment variables
  const mnemonic = process.env.MNEMONIC;
  const beaconAddress = process.env.RANDOMNESS_BEACON_ADDRESS;
  const rpcUrl = process.env.RPC_URL || 'http://localhost:8545';
  
  if (!mnemonic) {
    console.error('Error: MNEMONIC environment variable is required');
    process.exit(1);
  }

  if (!beaconAddress) {
    console.error('Error: RANDOMNESS_BEACON_ADDRESS environment variable is required');
    process.exit(1);
  }

  console.log('Initializing mnemonic signer...');
  console.log(`RandomnessBeacon address: ${beaconAddress}`);
  console.log(`RPC URL: ${rpcUrl}`);
  
  try {
    // Derive account from mnemonic
    const account = mnemonicToAccount(mnemonic);
    
    // Create blockchain clients
    const publicClient = createPublicClient({
      chain: sepolia, // Uses chain ID 31337 for local development
      transport: http(rpcUrl)
    });

    const walletClient = createWalletClient({
      account,
      chain: sepolia,
      transport: http(rpcUrl)
    });
    
    console.log(`Account address: ${account.address}`);
    console.log('Starting random number generation and posting to chain...\n');
    
    while (true) {
      try {
        // Generate random uint128 (max value: 2^128 - 1)
        // We'll use crypto.randomBytes for better randomness
        const randomBytesBuffer = randomBytes(16);
        let randomUint128 = 0n;
        
        // Convert bytes to bigint (uint128)
        for (let i = 0; i < 16; i++) {
          randomUint128 = (randomUint128 << 8n) + BigInt(randomBytesBuffer[i]!);
        }
        
        const timestamp = Date.now();
        
        console.log('---');
        console.log(`Timestamp: ${new Date(timestamp).toISOString()}`);
        console.log(`Random uint128: ${randomUint128.toString()}`);
        console.log(`Random uint128 (hex): 0x${randomUint128.toString(16)}`);
        console.log(`Posting to RandomnessBeacon at ${beaconAddress}...`);
        
        // Call postRandomness function on the contract
        const hash = await walletClient.writeContract({
          address: beaconAddress as `0x${string}`,
          abi: RANDOMNESS_BEACON_ABI,
          functionName: 'postRandomness',
          args: [randomUint128 as bigint]
        });
        
        console.log(`Transaction hash: ${hash}`);
        
        // Wait for transaction confirmation
        const receipt = await publicClient.waitForTransactionReceipt({ 
          hash,
          timeout: 30_000 // 30 second timeout
        });
        
        console.log(`Transaction confirmed in block: ${receipt.blockNumber}`);
        console.log(`Gas used: ${receipt.gasUsed}`);
        console.log(`Signer Address: ${account.address}`);
        
      } catch (error) {
        console.error('Error posting randomness to chain:', error);
      }

      await new Promise(resolve => setTimeout(resolve, 5000));
    }
  } catch (error) {
    console.error('Error initializing account from mnemonic:', error);
    process.exit(1);
  }
}

// Handle graceful shutdown
process.on('SIGINT', () => {
  console.log('\nShutting down gracefully...');
  process.exit(0);
});

process.on('SIGTERM', () => {
  console.log('\nShutting down gracefully...');
  process.exit(0);
});

main().catch((error) => {
  console.error('Unhandled error:', error);
  process.exit(1);
}); 