FROM rust:1.74-slim as builder
WORKDIR /usr/src/rusty-jio
COPY . .
RUN cargo build --release --bin jiopad --bin jio-cli

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rusty-jio/target/release/jiopad /usr/local/bin/jiopad
COPY --from=builder /usr/src/rusty-jio/target/release/jio-cli /usr/local/bin/jio-cli
EXPOSE 16110 16111
CMD ["jiopad"]
