FROM ubuntu:22.04 as builder
SHELL ["bash", "-c"]

ARG git_commit_id
ARG rust_version=1.70.0
# ENV NODE_VERSION=14.15.4

ENV GIT_COMMIT_ID=$git_commit_id
ENV TZ=UTC

RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone && \
    apt -yq update && \
    apt -yqq install --no-install-recommends curl ca-certificates \
        build-essential libssl-dev llvm-dev liblmdb-dev clang cmake \
        git pkg-config

# Install Rust and Cargo in /opt
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/cargo/bin:/opt/cargo/bin:$PATH

RUN curl --fail https://sh.rustup.rs -sSf \
        | sh -s -- -y --default-toolchain ${rust_version}-x86_64-unknown-linux-gnu --no-modify-path && \
    rustup default ${rust_version}-x86_64-unknown-linux-gnu && \
    rustup target add wasm32-unknown-unknown

ENV CARGO_HOME=/cargo \
    CARGO_TARGET_DIR=/cargo_target \
    PATH=/cargo/bin:$PATH

# Install IC CDK optimizer
# (keep version in sync with src/internet_identity/build.sh)
# RUN cargo install ic-cdk-optimizer --version 0.3.1
RUN cargo install ic-wasm --version 0.6.0
COPY . /build
WORKDIR /build

RUN cargo build --release -j1 --target wasm32-unknown-unknown --manifest-path=./Cargo.toml --target-dir=./target;
RUN ls target/wasm32-unknown-unknown
RUN ls target/wasm32-unknown-unknown/release
RUN ls target/wasm32-unknown-unknown/release

RUN ic-wasm target/wasm32-unknown-unknown/release/test-canister.wasm  -o test-canister-opt.wasm shrink
RUN ic-wasm target/wasm32-unknown-unknown/release/simple-validator.wasm  -o simple-validator-opt.wasm shrink
RUN ic-wasm target/wasm32-unknown-unknown/release/nx-gov-main.wasm  -o nx-gov-main-opt.wasm shrink
RUN ic-wasm target/wasm32-unknown-unknown/release/multisig-voting.wasm  -o multisig-voting-opt.wasm shrink

RUN sha256sum test-canister-opt.wasm
RUN sha256sum simple-validator-opt.wasm
RUN sha256sum nx-gov-main-opt.wasm
RUN sha256sum multisig-voting-opt.wasm

RUN gzip -c test-canister-opt.wasm > test-canister-opt.wasm.gz
RUN gzip -c simple-validator-opt.wasm > simple-validator-opt.wasm.gz
RUN gzip -c nx-gov-main-opt.wasm > nx-gov-main-opt.wasm.gz
RUN gzip -c multisig-voting-opt.wasm > multisig-voting-opt.wasm.gz
RUN sha256sum test-canister-opt.wasm.gz
RUN sha256sum simple-validator-opt.wasm.gz
RUN sha256sum nx-gov-main-opt.wasm.gz
RUN sha256sum multisig-voting-opt.wasm.gz
