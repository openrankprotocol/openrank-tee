# Use Node.js 20 Alpine image for smaller size
FROM node:20-alpine

# Set working directory
WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci --only=production

# Copy TypeScript configuration and source code
COPY tsconfig.json ./
COPY src/ ./src/

# Install dev dependencies needed for build
RUN npm install --save-dev typescript ts-node @types/node

# Build the TypeScript project
RUN npm run build

# Remove dev dependencies to reduce image size
RUN npm prune --production

# Create non-root user for security
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001

# Change ownership of the app directory
RUN chown -R nodejs:nodejs /app
USER nodejs

# Set environment variables
ENV NODE_ENV=production

# Run the application
CMD ["npm", "start"] 