# syntax=docker/dockerfile:1.7
# Thulp CLI Docker image.
#
# Multi-stage cargo-chef build: dependencies are cooked from a recipe that only
# captures manifests, so the dependency layer is cached across source edits and
# only the final `cargo build` recompiles the workspace crates themselves.
#
#   docker build -t thulp:local .

ARG RUST_IMAGE=rust:1.91-slim
ARG RUNTIME_IMAGE=debian:bookworm-slim

FROM ${RUST_IMAGE} AS chef
RUN cargo install cargo-chef --locked
WORKDIR /workspace/thulp

FROM chef AS planner
COPY . /workspace/thulp/
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV CARGO_TERM_COLOR=always
# OpenSSL is built from source (openssl-sys vendored) by the crate graph.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
# OpenSSL is built from source (openssl-sys vendored) by the crate graph, and
# rs-utcp compiles gRPC protos at build time, so protoc is required.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
COPY --from=planner /workspace/thulp/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/workspace/thulp/target \
    cargo chef cook --release --recipe-path recipe.json

COPY . /workspace/thulp/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/workspace/thulp/target \
    cargo build --release --bin thulp \
    && mkdir -p /artifacts \
    && cp target/release/thulp /artifacts/thulp

FROM ${RUNTIME_IMAGE} AS runtime

# Runtime dependencies.
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1000 thulp

COPY --from=builder /artifacts/thulp /usr/local/bin/thulp

USER thulp
WORKDIR /home/thulp

ENTRYPOINT ["/usr/local/bin/thulp"]
CMD ["--help"]

LABEL org.opencontainers.image.title="Thulp CLI"
LABEL org.opencontainers.image.description="Execution context engineering platform for AI agents"
LABEL org.opencontainers.image.url="https://github.com/dirmacs/thulp"
LABEL org.opencontainers.image.source="https://github.com/dirmacs/thulp"
LABEL org.opencontainers.image.version="0.1.0"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
