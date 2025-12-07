# Build stage
FROM rust:1.86 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code and migrations
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install PostgreSQL client
RUN apt-get update && apt-get install -y \
    libpq5 \
    postgresql-client \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary from builder
COPY --from=builder /app/target/release/emr-api /app/emr-api

# Copy migrations SQL files
COPY migrations /app/migrations

# Create a startup script
COPY scripts/docker-entrypoint.sh /app/docker-entrypoint.sh
RUN chmod +x /app/docker-entrypoint.sh

EXPOSE 8080

ENTRYPOINT ["/app/docker-entrypoint.sh"]