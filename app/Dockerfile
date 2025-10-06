FROM rust:1.84-bullseye AS builder

WORKDIR /app

# Install system dependencies including OpenSSL static libraries
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libssl1.1 \
    protobuf-compiler \
    ca-certificates \
    musl-tools \
    musl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install nightly toolchain
RUN rustup toolchain install nightly-2024-12-01 --component rust-src
RUN rustup default nightly-2024-12-01

# Copy the workspace
COPY . .

# Set environment variables for OpenSSL static linking
ENV OPENSSL_STATIC=1
ENV OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV RUSTC_BOOTSTRAP=1

# Build the application with vendored OpenSSL
RUN cargo build --release --package openrank-app

# Runtime stage
FROM debian:bullseye-slim

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/openrank-app ./openrank-app

# Make it executable
RUN chmod +x openrank-app

# Set the entrypoint
ENTRYPOINT ["./openrank-app"]
