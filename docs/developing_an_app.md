# Developing an Application

## Overview

The TEE AVS template lets you deploy containerized applications in Trusted Execution Environments using Docker Compose. You can iterate on the existing randomness beacon app or replace it entirely with your own project.

> **New to Docker Compose?** Docker Compose is a tool for defining and running multi-container applications. [Learn more about Docker Compose →](https://docs.docker.com/compose/)

## Environment Variables

### `MNEMONIC`
- **Source**: Auto-generated during deployment
- **Usage**: Available as environment variable in your containers
- **Purpose**: Derive EVM/Solana addresses and sign transactions
- **Example**: See `docker-compose.yml`. Then use with viem (typescript) library: `mnemonicToAccount(process.env.MNEMONIC)`

### `.env.public` vs `.env.private`
- **`.env.public`**: Plaintext environment variables visible to anyone
  - Use for: Contract addresses, public configuration
  - Example: `RANDOMNESS_BEACON_ADDRESS=0x1234...`
  
- **`.env.private`**: Encrypted with TEE's public key, only decryptable by your TEE
  - Use for: API keys, sensitive configuration
  - Example: `ALCHEMY_API_KEY=your_key_here`

## Development Workflow

### 1. Iterate on Existing App
Edit the app in `app`.

### 2. Replace with New Project
```bash
rm -rf app
mkdir app
cd app
# e.g. for a go application
go mod init github.com/username/app-name
```

**Example Dockerfile** (since the original is removed above):
```dockerfile
# Use appropriate base image for your language
FROM node:20-alpine

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci --only=production

# Copy source code
COPY . .

# Build if needed
RUN npm run build

# Create non-root user for security
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001
RUN chown -R nodejs:nodejs /app
USER nodejs

# Set environment variables
ENV NODE_ENV=production

# Run the application
CMD ["npm", "start"]
```

### 3. Build and Publish
```bash
# Build your image for linux
docker build -t username/app-name:latest app/ --platform linux/amd64

# Push to public registry  
docker push username/app-name:latest

# Get image digest
docker inspect username/app-name:latest --format='{{.Id}}'
```

### 5. Update docker-compose.yml
Replace the image reference with SHA256-qualified hash:
```yaml
services:
  your-app:
    image: username/app-name@sha256:abc123...
    environment:
      - MNEMONIC=${MNEMONIC}
      - YOUR_CONFIG=${YOUR_CONFIG}
    restart: unless-stopped
```

**Why use SHA256 hash instead of tags?**
Using the SHA256 digest (e.g., `@sha256:abc123...`) instead of a tag (e.g., `:latest` or `:v1`) ensures:
- **Immutability**: The exact same image is always deployed, preventing unexpected changes
- **Security**: Cryptographic verification that the image hasn't been tampered with
- **Reproducibility**: TEE attestations can verify the exact code being executed

> **Note:** In the future, the system will enforce hash-based image references only.

### 6. Deploy

Deploy your application to the TEE using [Devkit CLI](https://github.com/Layr-Labs/devkit-cli):

```bash
# Build with new configuration
devkit avs build --context=testnet

# Publish to TEE
devkit avs release publish \
--context=testnet \
--registry=docker.io/username/app-name
```

**What these commands do:**
- **`devkit avs build`**: Packages your docker-compose.yml and environment files (.env.public and .env.private) into a deployable artifact. The .env.private variables are encrypted with the TEE's public key.
- **`devkit avs release publish`**: Publishes the build artifacts to your specified Docker registry and deploys them to your TEE instance, where they will be securely executed.
## More

### Logging
- Use structured logging for better observability
- Log important state changes and transactions
- Avoid logging sensitive data

### Testing

Test your application locally before deploying to TEE environments. The `docker-compose.local.yml` file provides a template for local development with the same environment structure as production.

```bash
devkit avs run
```

> **Note:** If you want to test against a forked environment using anvil, you can optionally run `export SKIP_DEVNET_FUNDING=true` then `devkit avs devnet start` before running the above command. You would also need to point your app's RPCs at the local chain.

## Next Steps

1. Develop and test your app locally
2. Build and push Docker image to registry
3. Update `docker-compose.yml` with image hash
4. Deploy using `devkit avs build` and `devkit avs release publish`
5. Monitor with `make instance/logs`
