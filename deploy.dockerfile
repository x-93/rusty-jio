# syntax=docker/dockerfile:1

# ------------------------------------------------------------------------------
# Build Stage
# ------------------------------------------------------------------------------
FROM rust:alpine AS builder

RUN apk add --no-cache \
    musl-dev \
    gcc \
    make \
    perl \
    protobuf-dev \
    git \
    pkgconfig

WORKDIR /usr/src/rusty-jio

# Copy entire repository and workspace
COPY . .

# Build workspace release packages
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
    mkdir -p /app/bin /app/data && \
    chown -R jio:jio /app

WORKDIR /app

# Copy compiled artifacts from builder stage
COPY --from=builder /usr/src/rusty-jio/target/release/ /app/bin/

# Default Ports: P2P (29111), gRPC RPC (29110), wRPC (29112), Metrics (29113)
EXPOSE 29111 29110 29112 29113

VOLUME ["/app/data"]

USER jio

ENTRYPOINT ["/bin/sh"]
CMD ["-c", "if [ -f /app/bin/jiod ]; then exec /app/bin/jiod --appdir=/app/data; else echo 'Rusty-Jio core libraries container ready.'; fi"]
