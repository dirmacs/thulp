# Thulp CLI Docker Image
# Multi-stage build for minimal final image size

# Build stage
FROM rust:1.75-slim as builder

WORKDIR /usr/src/thulp

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY examples ./examples

# Build for release
RUN cargo build --release --bin thulp

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from builder
COPY --from=builder /usr/src/thulp/target/release/thulp /usr/local/bin/thulp

# Set up a non-root user
RUN useradd -m -u 1000 thulp
USER thulp
WORKDIR /home/thulp

# Set the binary as entrypoint
ENTRYPOINT ["/usr/local/bin/thulp"]
CMD ["--help"]

# Labels
LABEL org.opencontainers.image.title="Thulp CLI"
LABEL org.opencontainers.image.description="Execution context engineering platform for AI agents"
LABEL org.opencontainers.image.url="https://github.com/dirmacs/thulp"
LABEL org.opencontainers.image.source="https://github.com/dirmacs/thulp"
LABEL org.opencontainers.image.version="0.1.0"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
