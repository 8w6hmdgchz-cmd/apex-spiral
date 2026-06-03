# Dockerfile for NanoGPT-Claw - Multi-Architecture Build

# Build stage - uses build platform for faster builds
FROM --platform=$BUILDPLATFORM rust:1.90-slim-bullseye AS builder

# Enable cross-compilation
ARG TARGETPLATFORM
ARG BUILDPLATFORM

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    curl \
    gcc \
    && rm -rf /var/lib/apt/lists/*

# Copy cargo config and source
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config.toml ./

# Set up cross-compilation if needed
RUN if [ "$TARGETPLATFORM" != "$BUILDPLATFORM" ]; then \
      echo "Cross-compiling from $BUILDPLATFORM to $TARGETPLATFORM"; \
    fi

# Build release
RUN cargo build --release

# Runtime stage
FROM --platform=$TARGETPLATFORM debian:bullseye-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/nano-gpt-claw /usr/local/bin/nano-gpt-claw
COPY --from=builder /app/config.toml /app/config.toml

# Create data directories
RUN mkdir -p /app/data /var/log/nano-gpt-claw

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=0

# Expose ports if needed (webhooks, etc.)
EXPOSE 8000 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD nano-gpt-claw status || exit 1

# Default command
CMD ["nano-gpt-claw", "start"]
