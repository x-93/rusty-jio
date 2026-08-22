# ==========================================
# Stage 1: Build the Rust binaries
# ==========================================
FROM rust:1.81-slim AS builder

# Install system dependencies needed for compiling (protobuf compiler, pkg-config, etc.)
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    git \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Set up active workspace
WORKDIR /usr/src/app

# Copy the entire source tree into the build container
COPY . .

# Build all the binaries mentioned in your CI pipeline in release mode
RUN cargo build --bin jiopad --bin simpa --bin rothschild --bin jio-wallet --release

# ==========================================
# Stage 2: Minimalist runtime environment
# ==========================================
FROM debian:bookworm-slim

# Install SSL runtime dependencies for network requests
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binaries cleanly from the builder stage
COPY --from=builder /usr/src/app/target/release/jiopad /app/jiopad
COPY --from=builder /usr/src/app/target/release/simpa /app/simpa
COPY --from=builder /usr/src/app/target/release/rothschild /app/rothschild
COPY --from=builder /usr/src/app/target/release/jio-wallet /app/jio-wallet

# Expose the network port your service listens on (Change 8080 to your actual port if different)
EXPOSE 8080

# Default execution engine (Launches jiopad by default; you can change this to any binary)
CMD ["./jiopad"]
