#!/usr/bin/env bash
#
# WASM Runtime Testing Suite
# Tests security, performance, and functionality of WASM skills
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
TEST_LOG="${TEST_OUTPUT_DIR}/wasm-runtime-tests-$(date +%Y%m%d-%H%M%S).log"

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
    log "  Command: $test_command"

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
        log "  Output: $output"
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

    # Check if WASM skills are built
    local wasm_skill_dir="${PROJECT_ROOT}/examples/wasm-skills"
    if [[ ! -d "$wasm_skill_dir" ]]; then
        log_error "WASM skills directory not found: $wasm_skill_dir"
        exit 1
    fi

    # Check for test WASM skill (will be created)
    local test_skill_dir="${PROJECT_ROOT}/tests/fixtures/skills/test-wasm-skill"
    if [[ ! -d "$test_skill_dir" ]]; then
        log_warning "Test WASM skill not found at: $test_skill_dir"
        log_info "Run 'make build-test-skills' to create test fixtures"
    fi

    log_info "Using skill binary: $(which $SKILL_BIN)"
    log_info "Version: $($SKILL_BIN --version 2>&1 || echo 'unknown')"
}

#
# Category 1: Security Tests
#

test_filesystem_isolation() {
    log_info "=== Testing Filesystem Isolation ==="

    # TODO: Implement when test WASM skill is ready
    skip_test "Filesystem path traversal prevention" "Test skill not built"
    skip_test "Read access outside sandbox" "Test skill not built"
    skip_test "Write access outside sandbox" "Test skill not built"
    skip_test "Symlink following prevention" "Test skill not built"
}

test_network_capabilities() {
    log_info "=== Testing Network Capabilities ==="

    # TODO: Implement when test WASM skill is ready
    skip_test "Network allowlist enforcement" "Test skill not built"
    skip_test "Blocked domain access" "Test skill not built"
    skip_test "DNS resolution in sandbox" "Test skill not built"
}

test_resource_limits() {
    log_info "=== Testing Resource Limits ==="

    # TODO: Implement when test WASM skill is ready
    skip_test "Memory limit enforcement (16MB)" "Test skill not built"
    skip_test "Timeout enforcement (30s default)" "Test skill not built"
    skip_test "Memory allocation beyond limit" "Test skill not built"
}

#
# Category 2: Performance Tests
#

test_compilation_performance() {
    log_info "=== Testing Compilation Performance ==="

    # TODO: Implement with hyperfine benchmarking
    skip_test "Cold start compilation time" "Benchmarking not implemented"
    skip_test "Warm start execution time" "Benchmarking not implemented"
    skip_test "Cache verification" "Benchmarking not implemented"
    skip_test "Recompilation triggers" "Benchmarking not implemented"
}

#
# Category 3: Functionality Tests
#

test_parameter_passing() {
    log_info "=== Testing Parameter Passing ==="

    # Check if a working WASM skill exists
    local test_skill="github"  # Using github as test skill

    if $SKILL_BIN list 2>&1 | grep -q "$test_skill"; then
        log_info "Testing with $test_skill skill..."

        # Test help command works
        run_test "WASM skill help" \
            "$SKILL_BIN run $test_skill --help >/dev/null 2>&1" \
            0 \
            "Verify WASM skill can be invoked"
    else
        skip_test "Parameter passing tests" "No built WASM skills available"
    fi
}

test_error_handling() {
    log_info "=== Testing Error Handling ==="

    # Test with non-existent tool
    run_test "Non-existent WASM tool error" \
        "$SKILL_BIN run github:nonexistent-tool 2>&1; test \$? -ne 0" \
        0 \
        "Verify clear error for non-existent tool"

    # Test with invalid parameters
    skip_test "Invalid parameter types" "Test skill not built"
    skip_test "Missing required parameters" "Test skill not built"
}

#
# Category 4: Component Model Tests
#

test_component_model() {
    log_info "=== Testing WASM Component Model ==="

    skip_test "Component interface execution" "Test skill not built"
    skip_test "WIT file parsing" "Test skill not built"
    skip_test "Component linking" "Test skill not built"
}

#
# Main Execution
#

main() {
    log_info "Starting WASM Runtime Tests"
    log_info "Log file: $TEST_LOG"

    # Check prerequisites
    check_prerequisites

    # Run test categories
    test_filesystem_isolation
    test_network_capabilities
    test_resource_limits
    test_compilation_performance
    test_parameter_passing
    test_error_handling
    test_component_model

    # Summary
    echo ""
    log_info "=== Test Summary ==="
    log_info "Total Tests:   $TOTAL_TESTS"
    log_success "Passed:        $PASSED_TESTS"
    log_error "Failed:        $FAILED_TESTS"
    log_warning "Skipped:       $SKIPPED_TESTS"

    if [[ $TOTAL_TESTS -gt 0 ]]; then
        local pass_rate=$(( PASSED_TESTS > 0 && TOTAL_TESTS > 0 ? (PASSED_TESTS * 100) / (TOTAL_TESTS - SKIPPED_TESTS) : 0 ))
        log_info "Pass Rate:     ${pass_rate}% (excluding skipped)"
    fi

    # Exit with appropriate code
    if [[ $FAILED_TESTS -gt 0 ]]; then
        log_error "Some tests failed!"
        exit 1
    elif [[ $SKIPPED_TESTS -eq $TOTAL_TESTS ]]; then
        log_warning "All tests were skipped - WASM skills need to be built"
        log_info "Next steps:"
        log_info "  1. Build WASM skills: cd examples/wasm-skills/simple-skill && npm install && npm run build"
        log_info "  2. Create test fixtures: make build-test-skills (when Makefile target exists)"
        log_info "  3. Re-run tests: $0"
        exit 0
    else
        log_success "All tests passed!"
        exit 0
    fi
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
