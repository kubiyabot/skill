#!/usr/bin/env bash
#
# Security Testing Suite - Capability Enforcement
# Tests filesystem, network, and command capability controls
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
TEST_LOG="${TEST_OUTPUT_DIR}/security-capabilities-tests-$(date +%Y%m%d-%H%M%S).log"

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

# Security test - expect non-zero (blocked)
run_security_test() {
    local test_name="$1"
    local test_command="$2"
    local description="${3:-}"

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

    # For security tests, non-zero exit code = blocked = PASS
    if [[ $exit_code -ne 0 ]]; then
        log_success "$test_name - Capability denied (correct)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        log_error "$test_name - Capability NOT denied (security issue)!"
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
# Category 1: Filesystem Capability - Read
#

test_filesystem_read_capability() {
    log_info "=== Testing Filesystem Read Capability ==="

    # These tests require a skill with filesystem read capabilities configured
    skip_test "Read allowed path - success" "Requires test skill with read capability"
    skip_test "Read disallowed path - blocked" "Requires test skill with read capability"
    skip_test "Read outside allowed paths - blocked" "Requires test skill with read capability"
    skip_test "Read with relative path in allowed dir - success" "Requires test skill with read capability"
    skip_test "Read with absolute path outside allowed - blocked" "Requires test skill with read capability"
}

#
# Category 2: Filesystem Capability - Write
#

test_filesystem_write_capability() {
    log_info "=== Testing Filesystem Write Capability ==="

    # These tests require a skill with filesystem write capabilities configured
    skip_test "Write to allowed path - success" "Requires test skill with write capability"
    skip_test "Write to disallowed path - blocked" "Requires test skill with write capability"
    skip_test "Write outside allowed paths - blocked" "Requires test skill with write capability"
    skip_test "Create file in allowed dir - success" "Requires test skill with write capability"
    skip_test "Create file in disallowed dir - blocked" "Requires test skill with write capability"
    skip_test "Delete file in allowed dir - success" "Requires test skill with write capability"
    skip_test "Delete file in disallowed dir - blocked" "Requires test skill with write capability"
}

#
# Category 3: Filesystem Capability - Execute
#

test_filesystem_execute_capability() {
    log_info "=== Testing Filesystem Execute Capability ==="

    skip_test "Execute binary in allowed path - success" "Requires test skill with execute capability"
    skip_test "Execute binary in disallowed path - blocked" "Requires test skill with execute capability"
    skip_test "Execute script with allowed interpreter - success" "Requires test skill with execute capability"
    skip_test "Execute arbitrary command - blocked" "Requires test skill with execute capability"
}

#
# Category 4: Network Capability - Allowlist
#

test_network_allowlist() {
    log_info "=== Testing Network Capability Allowlist ==="

    # Requires skills with network capabilities configured
    skip_test "Connect to allowlisted domain - success" "Requires test skill with network capability"
    skip_test "Connect to non-allowlisted domain - blocked" "Requires test skill with network capability"
    skip_test "Connect via IP address - check policy" "Requires test skill with network capability"
    skip_test "DNS resolution of allowlisted domain - success" "Requires test skill with network capability"
    skip_test "DNS resolution of non-allowlisted - blocked" "Requires test skill with network capability"
}

#
# Category 5: Network Capability - Ports
#

test_network_ports() {
    log_info "=== Testing Network Port Restrictions ==="

    skip_test "Connect to allowed port - success" "Requires test skill with network capability"
    skip_test "Connect to disallowed port - blocked" "Requires test skill with network capability"
    skip_test "Listen on allowed port - success" "Requires test skill with network capability"
    skip_test "Listen on disallowed port - blocked" "Requires test skill with network capability"
    skip_test "Listen on privileged port - blocked" "Requires test skill with network capability"
}

#
# Category 6: Network Capability - Protocols
#

test_network_protocols() {
    log_info "=== Testing Network Protocol Restrictions ==="

    skip_test "HTTP request - check policy" "Requires test skill with network capability"
    skip_test "HTTPS request - check policy" "Requires test skill with network capability"
    skip_test "FTP connection - check policy" "Requires test skill with network capability"
    skip_test "SSH connection - check policy" "Requires test skill with network capability"
    skip_test "Raw socket - blocked" "Requires test skill with network capability"
}

#
# Category 7: Command Capability - Allowlist
#

test_command_allowlist() {
    log_info "=== Testing Command Allowlist ==="

    # Check if native skills with command allowlisting exist
    local native_skills_dir="$PROJECT_ROOT/examples/native-skills"

    if [[ -d "$native_skills_dir/kubernetes-skill" ]]; then
        # Kubernetes skill should only allow kubectl
        skip_test "Kubernetes skill runs kubectl - success" "Requires kubernetes cluster"
        skip_test "Kubernetes skill blocks other commands - blocked" "Requires runtime test"
    else
        skip_test "Kubernetes skill command tests" "kubernetes-skill not found"
    fi

    if [[ -d "$native_skills_dir/terraform-skill" ]]; then
        # Terraform skill should only allow terraform
        skip_test "Terraform skill runs terraform - success" "Requires terraform installed"
        skip_test "Terraform skill blocks other commands - blocked" "Requires runtime test"
    else
        skip_test "Terraform skill command tests" "terraform-skill not found"
    fi

    # General command allowlist tests
    skip_test "Allowlisted command executes - success" "Requires test skill with command allowlist"
    skip_test "Non-allowlisted command blocked" "Requires test skill with command allowlist"
}

#
# Category 8: Command Capability - Argument Validation
#

test_command_argument_validation() {
    log_info "=== Testing Command Argument Validation ==="

    skip_test "Allowed arguments - success" "Requires test skill with command allowlist"
    skip_test "Forbidden arguments - blocked" "Requires test skill with command allowlist"
    skip_test "Dangerous flag combinations - blocked" "Requires test skill with command allowlist"
    skip_test "Path arguments validated" "Requires test skill with command allowlist"
    skip_test "Command injection in arguments - blocked" "Requires test skill with command allowlist"
}

#
# Category 9: Environment Variable Capability
#

test_environment_variables() {
    log_info "=== Testing Environment Variable Control ==="

    skip_test "Access allowed environment variables - success" "Requires test skill with env access"
    skip_test "Access disallowed environment variables - blocked" "Requires test skill with env access"
    skip_test "Set environment variables - check policy" "Requires test skill with env access"
    skip_test "Inherit parent environment - check policy" "Requires test skill with env access"
    skip_test "PATH manipulation - blocked" "Requires test skill with env access"
    skip_test "LD_PRELOAD injection - blocked" "Requires test skill with env access"
}

#
# Category 10: Resource Capability
#

test_resource_capabilities() {
    log_info "=== Testing Resource Capability Limits ==="

    # Memory limits
    skip_test "Memory usage within limit - success" "Requires test skill with memory limit"
    skip_test "Memory usage exceeds limit - killed" "Requires test skill with memory limit"

    # CPU limits
    skip_test "CPU usage within limit - success" "Requires test skill with CPU limit"
    skip_test "CPU usage exceeds limit - throttled" "Requires test skill with CPU limit"

    # Time limits
    skip_test "Execution within timeout - success" "Requires test skill with timeout"
    skip_test "Execution exceeds timeout - killed" "Requires test skill with timeout"

    # File descriptor limits
    skip_test "Open files within limit - success" "Requires test skill with fd limit"
    skip_test "Open files exceeds limit - error" "Requires test skill with fd limit"
}

#
# Category 11: Capability Isolation
#

test_capability_isolation() {
    log_info "=== Testing Capability Isolation Between Skills ==="

    skip_test "Skill A capabilities don't affect Skill B" "Requires multiple test skills"
    skip_test "Instance-specific capabilities" "Requires skill with multiple instances"
    skip_test "Capability inheritance in subtasks" "Requires test skill with subtasks"
    skip_test "Capability escalation prevention" "Requires test skills"
}

#
# Main Execution
#

main() {
    log_info "Starting Security Testing - Capability Enforcement"
    log_info "Log file: $TEST_LOG"

    # Check prerequisites
    check_prerequisites

    log_warning ""
    log_warning "IMPORTANT: Capability tests verify security boundaries"
    log_warning "Allowed operations should succeed (exit 0)"
    log_warning "Denied operations should fail (exit non-zero)"
    log_warning ""

    # Run test categories
    test_filesystem_read_capability
    test_filesystem_write_capability
    test_filesystem_execute_capability
    test_network_allowlist
    test_network_ports
    test_network_protocols
    test_command_allowlist
    test_command_argument_validation
    test_environment_variables
    test_resource_capabilities
    test_capability_isolation

    # Summary
    echo ""
    log_info "=== Test Summary ==="
    log_info "Total Tests:   $TOTAL_TESTS"
    log_success "Passed:        $PASSED_TESTS (capabilities enforced)"
    log_error "Failed:        $FAILED_TESTS (capability violations)"
    log_warning "Skipped:       $SKIPPED_TESTS"

    if [[ $TOTAL_TESTS -gt 0 ]]; then
        local active_tests=$((TOTAL_TESTS - SKIPPED_TESTS))
        if [[ $active_tests -gt 0 ]]; then
            local pass_rate=$(( (PASSED_TESTS * 100) / active_tests ))
            log_info "Pass Rate:     ${pass_rate}% (capabilities correctly enforced)"
        fi
    fi

    log_info ""
    log_info "Note: Capability tests require test skills with various capability configurations"
    log_info "Create test fixtures in tests/fixtures/skills/ with different capability settings:"
    log_info "  - Filesystem read/write/execute capabilities"
    log_info "  - Network capabilities with domain/port allowlists"
    log_info "  - Command allowlists with argument restrictions"
    log_info "  - Environment variable access controls"
    log_info "  - Resource limits (memory, CPU, time, file descriptors)"

    # Exit with appropriate code
    if [[ $FAILED_TESTS -gt 0 ]]; then
        log_error "SECURITY ISSUE: Some capabilities were not properly enforced!"
        exit 1
    else
        log_success "All tested capabilities were properly enforced!"
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
