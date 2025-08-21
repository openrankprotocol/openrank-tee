FROM messense/rust-musl-cross:x86_64-musl AS builder

WORKDIR /app

COPY . .

RUN sudo apt-get update && apt-get install pkg-config -y
RUN apt-get install protobuf-compiler -y
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target=x86_64-unknown-linux-musl

# Runtime stage - builds the final image that other Dockerfiles can use as base
FROM alpine:latest AS runtime
WORKDIR /app

# Copy the binaries from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/openrank-app .

ENTRYPOINT ["/app/openrank-app"]
