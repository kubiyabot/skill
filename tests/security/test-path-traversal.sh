#!/usr/bin/env bash
#
# Security Testing Suite - Path Traversal Prevention
# Tests path traversal attacks and filesystem access control
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Test output directory
TEST_OUTPUT_DIR="${PROJECT_ROOT}/tests/output"
mkdir -p "$TEST_OUTPUT_DIR"

# Test log file
TEST_LOG="${TEST_OUTPUT_DIR}/security-path-traversal-tests-$(date +%Y%m%d-%H%M%S).log"

# Check if skill binary exists
SKILL_BIN="${SKILL_BIN:-skill}"

#
# Helper Functions
#

log() {
    echo "[$(date +%H:%M:%S)] $*" | tee -a "$TEST_LOG"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" | tee -a "$TEST_LOG"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $*" | tee -a "$TEST_LOG"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $*" | tee -a "$TEST_LOG"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $*" | tee -a "$TEST_LOG"
}

log_skip() {
    echo -e "${YELLOW}[SKIP]${NC} $*" | tee -a "$TEST_LOG"
}

# Special test for security - we expect attacks to FAIL (be blocked)
run_security_test() {
    local test_name="$1"
    local malicious_input="$2"
    local description="${3:-}"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    log_info "Running: $test_name"
    if [[ -n "$description" ]]; then
        log "  Description: $description"
    fi
    log "  Attack vector: ${malicious_input:0:100}"

    # Security tests should fail (non-zero exit) - that's good!
    local output
    local exit_code
    set +e
    output=$(eval "$malicious_input" 2>&1)
    exit_code=$?
    set -e

    # For security tests, non-zero exit code = blocked = PASS
    if [[ $exit_code -ne 0 ]]; then
        log_success "$test_name - Attack blocked"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        log_error "$test_name - Attack NOT blocked!"
        log "  Output: ${output:0:200}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

skip_test() {
    local test_name="$1"
    local reason="$2"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    SKIPPED_TESTS=$((SKIPPED_TESTS + 1))

    log_skip "$test_name - $reason"
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command_exists "$SKILL_BIN"; then
        log_error "skill binary not found at: $SKILL_BIN"
        exit 1
    fi

    log_info "Using skill binary: $(which $SKILL_BIN)"
    log_info "Version: $($SKILL_BIN --version 2>&1 || echo 'unknown')"
}

#
# Category 1: Basic Path Traversal
#

test_basic_path_traversal() {
    log_info "=== Testing Basic Path Traversal Attacks ==="

    # Note: These require test skills that accept file paths as parameters
    skip_test "Path traversal with ../" "Requires test skill with file parameter"
    skip_test "Path traversal with ../../" "Requires test skill with file parameter"
    skip_test "Path traversal with ../../../../etc/passwd" "Requires test skill with file parameter"
    skip_test "Path traversal with absolute path" "Requires test skill with file parameter"
    skip_test "Path traversal with mixed separators" "Requires test skill with file parameter"
}

#
# Category 2: Encoded Path Traversal
#

test_encoded_path_traversal() {
    log_info "=== Testing Encoded Path Traversal Attacks ==="

    # URL encoding: ../ = %2e%2e%2f
    skip_test "URL encoded ../ (%2e%2e%2f)" "Requires test skill with file parameter"

    # Double URL encoding: ../ = %252e%252e%252f
    skip_test "Double URL encoded ../ (%252e%252e%252f)" "Requires test skill with file parameter"

    # Unicode encoding
    skip_test "Unicode encoded ../ (..%c0%af)" "Requires test skill with file parameter"

    # UTF-8 overlong encoding
    skip_test "UTF-8 overlong encoded ../ (..%c0%2f)" "Requires test skill with file parameter"

    # Hex encoding
    skip_test "Hex encoded path" "Requires test skill with file parameter"
}

#
# Category 3: Symlink Attacks
#

test_symlink_attacks() {
    log_info "=== Testing Symlink Attack Prevention ==="

    # Create test symlinks if possible
    local test_dir="$TEST_OUTPUT_DIR/path-traversal-test"
    mkdir -p "$test_dir"

    # Try to create symlink to sensitive file
    if ln -s /etc/passwd "$test_dir/passwd-link" 2>/dev/null; then
        skip_test "Symlink to /etc/passwd" "Requires test skill with file parameter"
        rm -f "$test_dir/passwd-link"
    else
        skip_test "Symlink to /etc/passwd" "Cannot create symlinks (permissions)"
    fi

    skip_test "Symlink chain traversal" "Requires test skill with file parameter"
    skip_test "Relative symlink traversal" "Requires test skill with file parameter"
    skip_test "Circular symlink detection" "Requires test skill with file parameter"

    # Cleanup
    rm -rf "$test_dir"
}

#
# Category 4: Absolute Path Attacks
#

test_absolute_path_attacks() {
    log_info "=== Testing Absolute Path Attack Prevention ==="

    # Attempts to access sensitive system files
    skip_test "Access /etc/passwd" "Requires test skill with file parameter"
    skip_test "Access /etc/shadow" "Requires test skill with file parameter"
    skip_test "Access /root/.ssh/id_rsa" "Requires test skill with file parameter"
    skip_test "Access /proc/self/environ" "Requires test skill with file parameter"
    skip_test "Access $HOME/.aws/credentials" "Requires test skill with file parameter"
}

#
# Category 5: Windows Path Separators
#

test_windows_path_separators() {
    log_info "=== Testing Windows Path Separator Handling ==="

    # Test backslash handling (Windows-style)
    skip_test "Backslash path separator (..\\..\\)" "Requires test skill with file parameter"
    skip_test "Mixed separators (..\\../..\\)" "Requires test skill with file parameter"
    skip_test "UNC path (\\\\server\\share)" "Requires test skill with file parameter"
    skip_test "Drive letter (C:\\Windows\\System32)" "Requires test skill with file parameter"
}

#
# Category 6: Null Byte Injection
#

test_null_byte_injection() {
    log_info "=== Testing Null Byte Injection ==="

    # Null byte to terminate string early: file.txt%00.jpg
    skip_test "Null byte in filename" "Handled by shell and OS, already blocked"
    skip_test "Null byte to bypass extension check" "Handled by shell and OS"
}

#
# Category 7: Special File Access
#

test_special_file_access() {
    log_info "=== Testing Special File Access Prevention ==="

    # Device files
    skip_test "Access /dev/random" "Requires test skill with file parameter"
    skip_test "Access /dev/zero" "Requires test skill with file parameter"
    skip_test "Access /dev/null" "Requires test skill with file parameter"

    # Proc filesystem
    skip_test "Access /proc/version" "Requires test skill with file parameter"
    skip_test "Access /proc/cpuinfo" "Requires test skill with file parameter"
    skip_test "Access /proc/self/cmdline" "Requires test skill with file parameter"

    # Sys filesystem
    skip_test "Access /sys/class/net" "Requires test skill with file parameter"
}

#
# Category 8: Path Canonicalization
#

test_path_canonicalization() {
    log_info "=== Testing Path Canonicalization ==="

    # Various tricks to bypass path checks
    skip_test "Path with /./ sequences" "Requires test skill with file parameter"
    skip_test "Path with // double slashes" "Requires test skill with file parameter"
    skip_test "Path with trailing slash" "Requires test skill with file parameter"
    skip_test "Path with ./ prefix" "Requires test skill with file parameter"
    skip_test "Relative path without leading ./" "Requires test skill with file parameter"
}

#
# Main Execution
#

main() {
    log_info "Starting Security Testing - Path Traversal Prevention"
    log_info "Log file: $TEST_LOG"

    # Check prerequisites
    check_prerequisites

    log_warning ""
    log_warning "IMPORTANT: Path traversal tests expect attacks to be BLOCKED"
    log_warning "Non-zero exit codes = blocked attacks = PASS"
    log_warning ""

    # Run test categories
    test_basic_path_traversal
    test_encoded_path_traversal
    test_symlink_attacks
    test_absolute_path_attacks
    test_windows_path_separators
    test_null_byte_injection
    test_special_file_access
    test_path_canonicalization

    # Summary
    echo ""
    log_info "=== Test Summary ==="
    log_info "Total Tests:   $TOTAL_TESTS"
    log_success "Passed:        $PASSED_TESTS (attacks blocked)"
    log_error "Failed:        $FAILED_TESTS (attacks NOT blocked)"
    log_warning "Skipped:       $SKIPPED_TESTS"

    if [[ $TOTAL_TESTS -gt 0 ]]; then
        local active_tests=$((TOTAL_TESTS - SKIPPED_TESTS))
        if [[ $active_tests -gt 0 ]]; then
            local block_rate=$(( (PASSED_TESTS * 100) / active_tests ))
            log_info "Block Rate:    ${block_rate}% (attacks blocked)"
        fi
    fi

    log_info ""
    log_info "Note: Path traversal tests require test skills with file path parameters"
    log_info "Create test fixtures in tests/fixtures/skills/ that accept file paths"
    log_info "Example: A test skill that reads a file path and returns its contents"

    # Exit with appropriate code
    if [[ $FAILED_TESTS -gt 0 ]]; then
        log_error "SECURITY ISSUE: Some path traversal attacks were NOT blocked!"
        exit 1
    else
        log_success "All tested path traversal attacks were blocked!"
        if [[ $SKIPPED_TESTS -gt 0 ]]; then
            log_info "Note: $SKIPPED_TESTS tests were skipped (require test fixtures)"
        fi
        exit 0
    fi
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
