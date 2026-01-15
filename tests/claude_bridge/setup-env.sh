#!/bin/bash
# Setup script for Claude Bridge test environment
# This script prepares the local development environment for testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    else
        echo "unknown"
    fi
}

OS=$(detect_os)
info "Detected OS: $OS"

# Check Rust installation
check_rust() {
    info "Checking Rust installation..."
    if ! command -v rustc &> /dev/null; then
        warn "Rust not found. Installing via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    RUST_VERSION=$(rustc --version | awk '{print $2}')
    info "Rust version: $RUST_VERSION"

    # Update to stable if needed
    info "Updating Rust to stable..."
    rustup update stable
    rustup default stable
}

# Install system dependencies
install_dependencies() {
    info "Installing system dependencies..."

    if [[ "$OS" == "macos" ]]; then
        # macOS dependencies via Homebrew
        if ! command -v brew &> /dev/null; then
            warn "Homebrew not found. Please install from https://brew.sh"
            return 1
        fi

        info "Installing macOS dependencies..."
        brew install openssl pkg-config

        # Set OpenSSL environment variables
        export OPENSSL_DIR=$(brew --prefix openssl)
        info "Set OPENSSL_DIR=$OPENSSL_DIR"

    elif [[ "$OS" == "linux" ]]; then
        # Linux dependencies via apt
        info "Installing Linux dependencies..."
        sudo apt-get update
        sudo apt-get install -y build-essential pkg-config libssl-dev curl git
    fi
}

# Create test directories
create_test_dirs() {
    info "Creating test directories..."

    mkdir -p tests/claude_bridge/{acceptance_tests,fixtures/{manifests,skills},scripts,performance,security}
    mkdir -p /tmp/skill-test-output
    mkdir -p ~/.claude/skills-test

    info "Test directories created"
}

# Install test dependencies
install_test_deps() {
    info "Installing test dependencies..."

    # cargo-tarpaulin for coverage
    if ! command -v cargo-tarpaulin &> /dev/null; then
        info "Installing cargo-tarpaulin..."
        cargo install cargo-tarpaulin
    fi

    # cargo-audit for security
    if ! command -v cargo-audit &> /dev/null; then
        info "Installing cargo-audit..."
        cargo install cargo-audit
    fi

    # cargo-flamegraph for profiling
    if ! command -v cargo-flamegraph &> /dev/null; then
        info "Installing cargo-flamegraph..."
        cargo install flamegraph
    fi
}

# Build skill-cli
build_skill_cli() {
    info "Building skill-cli..."

    cargo build -p skill-cli --release

    SKILL_BIN="./target/release/skill"
    if [[ ! -f "$SKILL_BIN" ]]; then
        error "Failed to build skill-cli binary"
    fi

    # Verify build
    SKILL_VERSION=$($SKILL_BIN --version 2>&1 | head -n1)
    info "Built skill-cli: $SKILL_VERSION"

    # Add to PATH (for current session)
    export PATH="$PATH:$(pwd)/target/release"
    info "Added skill-cli to PATH"
}

# Make test scripts executable
make_scripts_executable() {
    info "Making test scripts executable..."

    find tests/claude_bridge -name "*.sh" -type f -exec chmod +x {} \;

    info "Test scripts are now executable"
}

# Setup Docker (if available)
setup_docker() {
    info "Checking Docker installation..."

    if ! command -v docker &> /dev/null; then
        warn "Docker not found. Skipping Docker setup."
        warn "Install Docker from https://www.docker.com/get-started"
        return 0
    fi

    DOCKER_VERSION=$(docker --version | awk '{print $3}' | sed 's/,//')
    info "Docker version: $DOCKER_VERSION"

    # Check docker-compose
    if command -v docker-compose &> /dev/null; then
        COMPOSE_VERSION=$(docker-compose --version | awk '{print $4}' | sed 's/,//')
        info "docker-compose version: $COMPOSE_VERSION"
    else
        warn "docker-compose not found. Some tests may not work."
    fi

    # Test Docker build
    info "Testing Docker build..."
    if docker build -f tests/claude_bridge/Dockerfile -t skill-cli-test . > /dev/null 2>&1; then
        info "Docker build successful"
    else
        warn "Docker build failed. Check Dockerfile configuration."
    fi
}

# Create .envrc for direnv users (optional)
create_envrc() {
    if [[ -f ".envrc" ]]; then
        info ".envrc already exists, skipping"
        return 0
    fi

    info "Creating .envrc for environment variables..."

    cat > .envrc << 'EOF'
# Claude Bridge Test Environment Variables

# Add built binaries to PATH
export PATH="$PWD/target/release:$PATH"

# Rust environment
export RUST_BACKTRACE=1
export RUST_LOG=info

# Test mode flag
export SKILL_TEST_MODE=1

# OpenSSL (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    export OPENSSL_DIR=$(brew --prefix openssl 2>/dev/null || echo "/usr/local/opt/openssl")
fi

echo "Skill Engine test environment loaded"
EOF

    info "Created .envrc (use 'direnv allow' to enable)"
}

# Verify environment
verify_environment() {
    info "Verifying environment setup..."

    # Check Rust
    if ! command -v rustc &> /dev/null; then
        error "Rust installation failed"
    fi

    # Check build
    if [[ ! -f "./target/release/skill" ]]; then
        error "skill-cli build failed"
    fi

    # Check test directories
    if [[ ! -d "tests/claude_bridge" ]]; then
        error "Test directories not created"
    fi

    info "Environment verification successful!"
}

# Print summary
print_summary() {
    echo ""
    echo "======================================"
    echo " Claude Bridge Test Environment Ready"
    echo "======================================"
    echo ""
    echo "Next steps:"
    echo ""
    echo "  1. Run all tests:"
    echo "     ./tests/claude_bridge/test-all.sh"
    echo ""
    echo "  2. Run specific test suites:"
    echo "     cargo test -p skill-cli --lib -- claude_bridge"
    echo "     ./tests/claude_bridge/test-skill-generation.sh"
    echo ""
    echo "  3. Run with Docker:"
    echo "     docker-compose -f tests/claude_bridge/docker-compose.yml up"
    echo ""
    echo "  4. View coverage:"
    echo "     cargo tarpaulin --workspace --exclude-files 'tests/*' --out Html"
    echo "     open tarpaulin-report.html"
    echo ""
    echo "  5. Start Task Master workflow:"
    echo "     task-master show 1.2"
    echo "     task-master set-status --id=1.1 --status=done"
    echo ""
    echo "======================================"
}

# Main execution
main() {
    info "Starting Claude Bridge test environment setup..."
    echo ""

    check_rust
    install_dependencies
    create_test_dirs
    install_test_deps
    build_skill_cli
    make_scripts_executable
    setup_docker
    create_envrc
    verify_environment

    echo ""
    info "Setup complete!"
    print_summary
}

# Run main function
main "$@"
