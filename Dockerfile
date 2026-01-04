# Build stage
FROM rust:1.81-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/homeautomation-rs

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy source file to build dependencies and cache them
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/homeautomation_rs*

# Copy the actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/homeautomation-rs/target/release/homeautomation-rs .

# Set environment variables (defaults or placeholders)
# These should be provided at runtime
ENV HASS_URL=""
ENV HASS_TOKEN=""
ENV MQTT_HOST=""

CMD ["./homeautomation-rs"]
