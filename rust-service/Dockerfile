# Builder stage
FROM rust:slim as builder

# Install system dependencies required by Tectonic native libraries
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config \
      build-essential \
      libssl-dev \
      libharfbuzz-dev \
      libfreetype6-dev \
      libgraphite2-dev \
      libicu-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/rust-service

# Copy manifests and source
COPY Cargo.toml ./
RUN cargo generate-lockfile
COPY src ./src

# Build in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Copy the built binary
COPY --from=builder /usr/src/rust-service/target/release/rust-service /usr/local/bin/rust-service

# Set entrypoint
ENTRYPOINT ["rust-service"]