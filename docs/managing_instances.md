
# Managing TEE Instances

Use these commands to monitor and manage your deployed TEE instances:

### View Instance Information
```bash
# Show detailed information about your TEE instance
make instance/describe

# Use a different context (default: testnet)
make instance/describe CONTEXT=mainnet
```

This displays:
- Instance ID, name, and status
- Machine type and zone
- Public IP address
- AVS and admin addresses
- Creation timestamp

### View Instance Logs
```bash
# Stream logs from your running TEE instance
make instance/logs

# Use a different context
make instance/logs CONTEXT=mainnet
```

This fetches real-time logs from your TEE instance's guest agent.

### View Instance Keys
```bash
# Display TEE-generated keys and addresses
make instance/keys

# Use a different context
make instance/keys CONTEXT=mainnet
```

This displays:
- AVS configuration addresses
- TEE-generated EVM and Solana addresses (BIP39 derived)
- AGE public key used to encrypt .env.private variables

### View Attestation Quote
```bash
# Display the TDX attestation quote
make instance/attestation

# Use a different context
make instance/attestation CONTEXT=mainnet
```

This shows the Intel TDX quote that cryptographically attests to:
- TEE-generated addresses
- Encryption keys
- Docker Compose application hash

**Note**: For local development (`CONTEXT=devnet`), these commands show appropriate messages since devnet runs locally without TEE features.