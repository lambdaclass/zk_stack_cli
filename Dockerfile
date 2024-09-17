# The image generated weights ~6GB, not ideal, but musl is not working.
#
# FROM clux/muslrust:amd64-1.83.0-nightly-2024-09-06 AS builder
# 
# WORKDIR /usr/src/zk_stack_cli
# 
# COPY Cargo.toml Cargo.lock ./
# COPY ./src ./src
# 
# RUN cargo build --release
# 
# FROM debian:buster-slim
# 
# RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
# 
# COPY --from=builder /usr/src/zk_stack_cli/target/x86_64-unknown-linux-musl/release/zks /usr/local/bin/zks
# CMD ["zks ${COMMAND}"]

FROM rust:bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev musl-tools musl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/zk_stack_cli

COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY ./src ./src

RUN cargo install --path .

ENV ETH_CLIENT_WEB3_URL=http://localhost:8545 \
    ETH_CLIENT_CHAIN_ID=9 \
    DATABASE_URL=ppostgres://postgres:notsecurepassword@localhost/zksync_local \
    DATABASE_PROVER_URL=postgres://postgres:notsecurepassword@localhost/prover_local \
    API_WEB3_JSON_RPC_HTTP_URL=http://localhost:3050  \
    CHAIN_ETH_ZKSYNC_NETWORK_ID=270


# Set environment variables (you can also pass these at runtime)
ENV WALLET_ADDR=0xa13c10c0d5bd6f79041b9835c63f91de35a15883  \
    WALLET_PK=0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8  \
    L1_EXPLORER_URL=placeholder \
    L2_EXPLORER_URL=http://localhost:3010 \ 
    GOVERNANCE_ADDRESS=0xa13c10c0d5bd6f79041b9835c63f91de35a15883  \
    GOVERNANCE_OWNER_PK=0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8  \
    BRIDGEHUB_ADMIN_PK=0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8  \
    BRIDGEHUB_OWNER_PK=0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8 

ENV COMMAND="--help"

CMD zks config create-from-env local && zks $COMMAND
