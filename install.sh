#!/bin/sh
# Skill Engine Installer (Security Hardened)
# Usage: curl -fsSL https://raw.githubusercontent.com/kubiyabot/skill/main/install.sh | sh
#
# Environment Variables:
#   SKILL_INSTALL_DIR  - Installation directory (default: ~/.skill-engine/bin)
#   SKILL_NO_MODIFY_PATH - Don't modify PATH in shell rc files
#   SKILL_VERSION      - Specific version to install (default: latest)
#   SKILL_SKIP_VERIFY  - Skip checksum verification (NOT RECOMMENDED)

set -e

# Colors (disabled in non-interactive shells)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# Configuration
GITHUB_REPO="kubiyabot/skill"
INSTALL_DIR="${SKILL_INSTALL_DIR:-$HOME/.skill-engine/bin}"
BINARY_NAME="skill"

# Security: Validate HOME directory exists
if [ -z "$HOME" ] || [ ! -d "$HOME" ]; then
    printf "${RED}Error:${NC} HOME directory is not set or does not exist\n" >&2
    exit 1
fi

# Logging functions
info() {
    printf "${BLUE}==>${NC} %s\n" "$1"
}

success() {
    printf "${GREEN}==>${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}Warning:${NC} %s\n" "$1"
}

error() {
    printf "${RED}Error:${NC} %s\n" "$1" >&2
    exit 1
}

# Detect platform
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $ARCH"
            ;;
    esac

    case "$OS" in
        Darwin)
            OS="apple-darwin"
            ;;
        Linux)
            OS="unknown-linux-musl"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            error "Windows is not supported. Please use WSL2."
            ;;
        *)
            error "Unsupported OS: $OS"
            ;;
    esac

    PLATFORM="${ARCH}-${OS}"
    info "Detected platform: $PLATFORM"
}

# Check for required tools
check_dependencies() {
    if ! command -v curl >/dev/null 2>&1; then
        error "curl is required but not installed. Please install curl first."
    fi

    if ! command -v tar >/dev/null 2>&1; then
        error "tar is required but not installed. Please install tar first."
    fi

    # Check for sha256sum or shasum for checksum verification
    if ! command -v sha256sum >/dev/null 2>&1 && ! command -v shasum >/dev/null 2>&1; then
        if [ -z "$SKILL_SKIP_VERIFY" ]; then
            error "sha256sum or shasum is required for verification. Install it or set SKILL_SKIP_VERIFY=1 (not recommended)"
        else
            warn "Checksum verification tools not found - skipping verification (INSECURE)"
        fi
    fi
}

# Verify file checksum
verify_checksum() {
    local file="$1"
    local expected_checksum="$2"
    
    if [ -z "$SKILL_SKIP_VERIFY" ]; then
        info "Verifying download integrity..."
        
        # Try sha256sum first, fall back to shasum
        if command -v sha256sum >/dev/null 2>&1; then
            actual_checksum=$(sha256sum "$file" | awk '{print $1}')
        elif command -v shasum >/dev/null 2>&1; then
            actual_checksum=$(shasum -a 256 "$file" | awk '{print $1}')
        else
            warn "Cannot verify checksum - no tool available"
            return 0
        fi
        
        if [ "$actual_checksum" != "$expected_checksum" ]; then
            error "Checksum verification failed!
Expected: $expected_checksum
Actual: $actual_checksum
The download may be corrupted or tampered with."
        fi
        
        success "Checksum verified successfully"
    else
        warn "Skipping checksum verification (INSECURE - set by SKILL_SKIP_VERIFY)"
    fi
}

# Get latest version from GitHub
get_latest_version() {
    if [ -n "$SKILL_VERSION" ]; then
        echo "$SKILL_VERSION"
        return
    fi

    info "Fetching latest version..."

    # Try to get latest release with TLS cert validation
    LATEST_RELEASE=$(curl -fsSL --tlsv1.2 --proto "=https" \
        "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" 2>/dev/null || echo "")

    if [ -z "$LATEST_RELEASE" ]; then
        # Fallback to tags if no releases
        warn "No releases found, falling back to tags..."
        VERSION=$(curl -fsSL --tlsv1.2 --proto "=https" \
            "https://api.github.com/repos/${GITHUB_REPO}/tags" 2>/dev/null | \
            grep '"name":' | head -1 | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^v//')
    else
        VERSION=$(echo "$LATEST_RELEASE" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^v//')
    fi

    if [ -z "$VERSION" ]; then
        error "Could not determine latest version. Please specify SKILL_VERSION."
    fi

    echo "$VERSION"
}

# Download and install
install() {
    VERSION=$(get_latest_version)
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/skill-${PLATFORM}.tar.gz"
    CHECKSUM_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/skill-${PLATFORM}.tar.gz.sha256"

    info "Installing Skill Engine v${VERSION}..."

    # Create install directory with secure permissions
    mkdir -p "$INSTALL_DIR"
    chmod 755 "$INSTALL_DIR"

    # Create temp directory with secure permissions
    TMP_DIR=$(mktemp -d)
    chmod 700 "$TMP_DIR"
    trap "rm -rf $TMP_DIR" EXIT

    # Download with TLS validation
    info "Downloading from $DOWNLOAD_URL"
    if ! curl -fsSL --tlsv1.2 --proto "=https" "$DOWNLOAD_URL" -o "$TMP_DIR/skill.tar.gz"; then
        error "Download failed. Please check the version and try again."
    fi

    # Download checksum file
    if [ -z "$SKILL_SKIP_VERIFY" ]; then
        info "Downloading checksum..."
        if curl -fsSL --tlsv1.2 --proto "=https" "$CHECKSUM_URL" -o "$TMP_DIR/skill.tar.gz.sha256" 2>/dev/null; then
            EXPECTED_CHECKSUM=$(cat "$TMP_DIR/skill.tar.gz.sha256" | awk '{print $1}')
            verify_checksum "$TMP_DIR/skill.tar.gz" "$EXPECTED_CHECKSUM"
        else
            warn "Checksum file not available - skipping verification"
        fi
    fi

    # Extract
    info "Extracting..."
    tar -xzf "$TMP_DIR/skill.tar.gz" -C "$TMP_DIR"

    # Find the binary (might be in a subdirectory)
    if [ -f "$TMP_DIR/skill" ]; then
        BINARY_PATH="$TMP_DIR/skill"
    elif [ -f "$TMP_DIR/$BINARY_NAME" ]; then
        BINARY_PATH="$TMP_DIR/$BINARY_NAME"
    else
        # Search for it
        BINARY_PATH=$(find "$TMP_DIR" -name "skill" -type f 2>/dev/null | head -1)
        if [ -z "$BINARY_PATH" ]; then
            error "Could not find skill binary in archive."
        fi
    fi

    # Move to install directory
    mv "$BINARY_PATH" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod 755 "${INSTALL_DIR}/${BINARY_NAME}"

    # Verify installation
    if ! "${INSTALL_DIR}/${BINARY_NAME}" --version >/dev/null 2>&1; then
        error "Installation verification failed. Binary may be corrupted."
    fi

    success "Installed to: ${INSTALL_DIR}/${BINARY_NAME}"
}

# Add to PATH
setup_path() {
    if [ -n "$SKILL_NO_MODIFY_PATH" ]; then
        info "Skipping PATH modification (SKILL_NO_MODIFY_PATH is set)"
        return
    fi

    EXPORT_LINE="export PATH=\"${INSTALL_DIR}:\$PATH\""
    SHELL_RC_MODIFIED=false

    for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
        if [ -f "$rc" ]; then
            if grep -q "skill-engine" "$rc" 2>/dev/null; then
                info "PATH already configured in $rc"
            else
                echo "" >> "$rc"
                echo "# Skill Engine" >> "$rc"
                echo "$EXPORT_LINE" >> "$rc"
                info "Added to PATH in $rc"
                SHELL_RC_MODIFIED=true
            fi
        fi
    done

    if [ "$SHELL_RC_MODIFIED" = true ]; then
        echo ""
        info "Restart your shell or run:"
        echo "  $EXPORT_LINE"
    elif [ "$SHELL_RC_MODIFIED" = false ] && [ ! -f "$HOME/.bashrc" ] && [ ! -f "$HOME/.zshrc" ]; then
        echo ""
        info "Add to your PATH manually:"
        echo "  $EXPORT_LINE"
    fi
}

# Print success message
print_success() {
    echo ""
    success "Skill Engine installed successfully!"
    echo ""
    echo "Get started:"
    echo "  ${INSTALL_DIR}/${BINARY_NAME} --help"
    echo "  ${INSTALL_DIR}/${BINARY_NAME} install ./my-skill"
    echo "  ${INSTALL_DIR}/${BINARY_NAME} serve  # Start MCP server"
    echo ""

    # Show version
    VERSION=$("${INSTALL_DIR}/${BINARY_NAME}" --version 2>/dev/null || echo "unknown")
    info "Installed version: $VERSION"
}

# Parse arguments
parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --version)
                shift
                SKILL_VERSION="$1"
                ;;
            --install-dir)
                shift
                SKILL_INSTALL_DIR="$1"
                INSTALL_DIR="$1"
                ;;
            --no-modify-path)
                SKILL_NO_MODIFY_PATH=1
                ;;
            -h|--help)
                echo "Skill Engine Installer"
                echo ""
                echo "Usage: curl -fsSL https://skill.sh/install | sh [-s -- OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --version VERSION      Install specific version"
                echo "  --install-dir DIR      Installation directory"
                echo "  --no-modify-path       Don't modify shell rc files"
                echo "  -h, --help             Show this help"
                echo ""
                echo "Environment Variables:"
                echo "  SKILL_VERSION          Same as --version"
                echo "  SKILL_INSTALL_DIR      Same as --install-dir"
                echo "  SKILL_NO_MODIFY_PATH   Same as --no-modify-path (set to 1)"
                echo "  SKILL_SKIP_VERIFY      Skip checksum verification (NOT RECOMMENDED)"
                exit 0
                ;;
            *)
                warn "Unknown option: $1"
                ;;
        esac
        shift
    done
}

# Main
main() {
    parse_args "$@"

    echo ""
    echo "  Skill Engine Installer (Security Hardened)"
    echo "  ------------------------------------------"
    echo ""

    check_dependencies
    detect_platform
    install
    setup_path
    print_success
}

main "$@"
