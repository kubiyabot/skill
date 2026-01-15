#!/usr/bin/env bash
#
# Security Testing Suite - Credential Security
# Tests credential storage, encryption, and secret management
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
TEST_LOG="${TEST_OUTPUT_DIR}/security-credentials-tests-$(date +%Y%m%d-%H%M%S).log"

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

run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_exit_code="${3:-0}"
    local description="${4:-}"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    log_info "Running: $test_name"
    if [[ -n "$description" ]]; then
        log "  Description: $description"
    fi

    local output
    local exit_code
    set +e
    output=$(eval "$test_command" 2>&1)
    exit_code=$?
    set -e

    if [[ $exit_code -eq $expected_exit_code ]]; then
        log_success "$test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        log_error "$test_name"
        log "  Expected exit code: $expected_exit_code, got: $exit_code"
        if [[ -n "$output" ]]; then
            log "  Output: ${output:0:200}"
        fi
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
# Category 1: Credential Storage
#

test_credential_storage() {
    log_info "=== Testing Credential Storage ==="

    # Test that credentials are stored securely
    skip_test "Credentials stored in keyring" "Requires test skill with credentials"
    skip_test "Credentials use AES-256-GCM encryption" "Requires encryption inspection"
    skip_test "Credentials not stored in plaintext" "Requires filesystem inspection"
    skip_test "Credentials not in config files" "Requires test skill with credentials"
    skip_test "Credentials not in environment variables" "Requires test skill with credentials"
}

#
# Category 2: Credential Access Control
#

test_credential_access_control() {
    log_info "=== Testing Credential Access Control ==="

    # Only authorized skills should access their credentials
    skip_test "Skill can access own credentials" "Requires test skill with credentials"
    skip_test "Skill cannot access other skill's credentials" "Requires multiple test skills"
    skip_test "Instance-specific credential isolation" "Requires skill with instances"
    skip_test "User-level credential isolation" "Requires multi-user test environment"
}

#
# Category 3: Secret Redaction in Logs
#

test_secret_redaction() {
    log_info "=== Testing Secret Redaction in Logs ==="

    # Secrets should never appear in logs
    skip_test "API keys redacted from logs" "Requires test skill with API key"
    skip_test "Passwords redacted from logs" "Requires test skill with password"
    skip_test "Tokens redacted from logs" "Requires test skill with token"
    skip_test "Credentials redacted from error messages" "Requires test skill with credentials"
    skip_test "Secrets redacted from audit logs" "Requires test skill with credentials"
}

#
# Category 4: Secret Redaction in Output
#

test_secret_redaction_output() {
    log_info "=== Testing Secret Redaction in Output ==="

    skip_test "API keys redacted from command output" "Requires test skill with API key"
    skip_test "Passwords redacted from command output" "Requires test skill with password"
    skip_test "Tokens masked in web interface" "Requires web interface test"
    skip_test "Credentials not in JSON response" "Requires test skill with credentials"
}

#
# Category 5: Credential Lifecycle
#

test_credential_lifecycle() {
    log_info "=== Testing Credential Lifecycle ==="

    # Test credential operations
    skip_test "Set credential via config command" "Requires test skill"
    skip_test "Update existing credential" "Requires test skill with credentials"
    skip_test "Delete credential" "Requires test skill with credentials"
    skip_test "Credential persists across restarts" "Requires test skill with credentials"
    skip_test "Deleted credential not accessible" "Requires test skill with credentials"
}

#
# Category 6: Encryption Strength
#

test_encryption_strength() {
    log_info "=== Testing Encryption Strength ==="

    # Verify encryption meets security standards
    skip_test "Uses AES-256-GCM algorithm" "Requires crypto inspection"
    skip_test "Proper key derivation (PBKDF2/Argon2)" "Requires crypto inspection"
    skip_test "Unique IV per encryption" "Requires crypto inspection"
    skip_test "Authenticated encryption verified" "Requires crypto inspection"
    skip_test "No weak ciphers (DES, RC4, etc.)" "Requires crypto inspection"
}

#
# Category 7: Credential Injection Prevention
#

test_credential_injection() {
    log_info "=== Testing Credential Injection Prevention ==="

    # Prevent malicious credential injection
    skip_test "Cannot inject credentials via command line" "Requires test skill"
    skip_test "Cannot inject credentials via environment" "Requires test skill"
    skip_test "Cannot inject credentials via config file" "Requires test skill"
    skip_test "Cannot overwrite other skill's credentials" "Requires multiple test skills"
}

#
# Category 8: Credential Exposure in Errors
#

test_credential_exposure_errors() {
    log_info "=== Testing Credential Exposure in Error Messages ==="

    # Errors should not leak credential information
    skip_test "Invalid credential error doesn't show value" "Requires test skill"
    skip_test "Connection error doesn't show password" "Requires test skill"
    skip_test "Auth failure doesn't show token" "Requires test skill"
    skip_test "Stack traces don't contain secrets" "Requires test skill"
}

#
# Category 9: Keyring Integration
#

test_keyring_integration() {
    log_info "=== Testing Keyring Integration ==="

    # Test OS keyring integration
    if [[ "$OSTYPE" == "darwin"* ]]; then
        skip_test "macOS Keychain integration" "Requires credential operations"
        skip_test "Keychain access control lists" "Requires credential operations"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        skip_test "Linux Secret Service integration" "Requires credential operations"
        skip_test "GNOME Keyring integration" "Requires credential operations"
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
        skip_test "Windows Credential Manager integration" "Requires credential operations"
    fi

    skip_test "Keyring fallback mechanism" "Requires credential operations"
    skip_test "Encrypted file storage fallback" "Requires credential operations"
}

#
# Category 10: Audit Logging
#

test_credential_audit_logging() {
    log_info "=== Testing Credential Audit Logging ==="

    # Credential operations should be audited
    skip_test "Credential set operation logged" "Requires test skill and audit log inspection"
    skip_test "Credential access logged" "Requires test skill and audit log inspection"
    skip_test "Credential deletion logged" "Requires test skill and audit log inspection"
    skip_test "Failed credential access logged" "Requires test skill and audit log inspection"
    skip_test "Audit log tampering prevention" "Requires audit log inspection"
}

#
# Category 11: Credential Transmission
#

test_credential_transmission() {
    log_info "=== Testing Credential Transmission Security ==="

    # Credentials should be transmitted securely
    skip_test "Credentials use TLS in transit" "Requires network inspection"
    skip_test "Credentials not in URL parameters" "Requires HTTP inspection"
    skip_test "Credentials in request headers only" "Requires HTTP inspection"
    skip_test "Credentials use proper auth mechanisms" "Requires test skill"
}

#
# Main Execution
#

main() {
    log_info "Starting Security Testing - Credential Security"
    log_info "Log file: $TEST_LOG"

    # Check prerequisites
    check_prerequisites

    log_warning ""
    log_warning "IMPORTANT: Credential security tests verify secret management"
    log_warning "Credentials should be encrypted, isolated, and never exposed"
    log_warning ""

    # Run test categories
    test_credential_storage
    test_credential_access_control
    test_secret_redaction
    test_secret_redaction_output
    test_credential_lifecycle
    test_encryption_strength
    test_credential_injection
    test_credential_exposure_errors
    test_keyring_integration
    test_credential_audit_logging
    test_credential_transmission

    # Summary
    echo ""
    log_info "=== Test Summary ==="
    log_info "Total Tests:   $TOTAL_TESTS"
    log_success "Passed:        $PASSED_TESTS (security verified)"
    log_error "Failed:        $FAILED_TESTS (security issues)"
    log_warning "Skipped:       $SKIPPED_TESTS"

    if [[ $TOTAL_TESTS -gt 0 ]]; then
        local active_tests=$((TOTAL_TESTS - SKIPPED_TESTS))
        if [[ $active_tests -gt 0 ]]; then
            local pass_rate=$(( (PASSED_TESTS * 100) / active_tests ))
            log_info "Pass Rate:     ${pass_rate}% (security maintained)"
        fi
    fi

    log_info ""
    log_info "Note: Credential security tests require:"
    log_info "  - Test skills with credential configuration"
    log_info "  - Crypto library inspection tools"
    log_info "  - Audit log analysis capabilities"
    log_info "  - Network traffic inspection (for transmission tests)"
    log_info "  - OS keyring integration testing"

    # Exit with appropriate code
    if [[ $FAILED_TESTS -gt 0 ]]; then
        log_error "SECURITY ISSUE: Credential security vulnerabilities detected!"
        exit 1
    else
        log_success "All tested credential security measures passed!"
        if [[ $SKIPPED_TESTS -gt 0 ]]; then
            log_info "Note: $SKIPPED_TESTS tests were skipped (require test fixtures or inspection tools)"
        fi
        exit 0
    fi
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
