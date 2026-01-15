# Claude Bridge Test Environment
# Using nightly for edition2024 support

FROM rustlang/rust:nightly-slim

# Install system dependencies required for building skill-cli
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /workspace

# Create directories for test artifacts
RUN mkdir -p /tmp/skill-test-output \
    && mkdir -p /root/.claude/skills-test

# Copy Cargo workspace files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build dependencies (cached layer)
RUN cargo fetch

# Copy test files
COPY tests/ ./tests/

# Set environment variables for testing
ENV RUST_BACKTRACE=1
ENV RUST_LOG=info
ENV SKILL_TEST_MODE=1

# Default command: run all tests
CMD ["cargo", "test", "--workspace", "--", "--test-threads=1"]

# Health check: verify Rust toolchain
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD cargo --version || exit 1

# Labels for container identification
LABEL maintainer="Skill Engine Team"
LABEL description="Test environment for Claude Bridge integration"
LABEL version="1.0"
