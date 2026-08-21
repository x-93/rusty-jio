# syntax=docker/dockerfile:1

# ------------------------------------------------------------------------------
# Build Stage
# ------------------------------------------------------------------------------
FROM rust:1.85-alpine AS builder

RUN apk add --no-cache \
    musl-dev \
    gcc \
    make \
    perl \
    protobuf-dev \
    git

WORKDIR /usr/src/rusty-jio

# Copy entire repository and workspace
COPY . .

# Build workspace release binaries
RUN cargo build --workspace --release

# ------------------------------------------------------------------------------
# Runtime Stage
# ------------------------------------------------------------------------------
FROM alpine:3.21 AS runner

RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    libstdc++ \
    su-exec

# Create non-privileged user and data directory
RUN addgroup -g 1000 jio && \
    adduser -u 1000 -G jio -s /bin/sh -D jio && \
    mkdir -p /app/data && \
    chown -R jio:jio /app

WORKDIR /app

# Copy compiled binaries from builder stage
COPY --from=builder /usr/src/rusty-jio/target/release/ /app/bin/

# Default Ports: P2P, gRPC, wRPC, Metrics
EXPOSE 16111 16110 17110 2112

VOLUME ["/app/data"]

USER jio

ENTRYPOINT ["/app/bin/jiod"]
CMD ["--appdir=/app/data", "--rpclisten=0.0.0.0:16110", "--wrpclisten=0.0.0.0:17110"]
