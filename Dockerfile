# Build stage
FROM rust:1.85-bookworm as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary (CLI server only — skip WASM crate)
RUN cargo build --release -p policycheck

# Runtime stage
FROM debian:bookworm-slim

# Install CA certificates for HTTPS
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/policycheck /usr/local/bin/policycheck

# Expose port
EXPOSE 3000

# Run the server
CMD ["policycheck", "serve", "--host", "0.0.0.0", "--port", "3000"]
