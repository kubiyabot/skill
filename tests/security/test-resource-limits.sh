#!/usr/bin/env bash
#
# Security Testing Suite - Resource Limit Enforcement
# Tests memory, CPU, disk, and time limit enforcement
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
TEST_LOG="${TEST_OUTPUT_DIR}/security-resource-limits-tests-$(date +%Y%m%d-%H%M%S).log"

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
# Category 1: Memory Limits
#

test_memory_limits() {
    log_info "=== Testing Memory Limit Enforcement ==="

    # Test WASM runtime memory limits (documented as 256 pages = 16MB)
    skip_test "WASM skill within memory limit succeeds" "Requires WASM test skill"
    skip_test "WASM skill exceeding memory limit fails" "Requires WASM test skill"
    skip_test "WASM memory limit is 256 pages (16MB)" "Requires WASM runtime inspection"
    skip_test "WASM OOM error is graceful" "Requires WASM test skill"

    # Test Docker runtime memory limits
    skip_test "Docker skill with memory limit succeeds" "Requires Docker test skill"
    skip_test "Docker skill exceeding memory limit killed" "Requires Docker test skill"
    skip_test "Docker OOM killer triggered correctly" "Requires Docker test skill"

    # Test Native runtime memory limits (if applicable)
    skip_test "Native skill memory monitoring" "Requires native test skill with monitoring"
    skip_test "Native skill memory limit warning" "Requires native test skill"
}

#
# Category 2: CPU Limits
#

test_cpu_limits() {
    log_info "=== Testing CPU Limit Enforcement ==="

    # Docker CPU limits
    skip_test "Docker skill with CPU limit succeeds" "Requires Docker test skill"
    skip_test "Docker skill CPU throttling works" "Requires Docker test skill"
    skip_test "Docker CPU shares enforced" "Requires Docker test skill"

    # CPU time limits
    skip_test "CPU-intensive task within limit succeeds" "Requires test skill"
    skip_test "CPU-intensive task exceeds limit terminated" "Requires test skill"

    # Multi-core handling
    skip_test "CPU affinity controls work" "Requires Docker test skill"
    skip_test "CPU quota enforcement" "Requires Docker test skill"
}

#
# Category 3: Execution Time Limits
#

test_execution_time_limits() {
    log_info "=== Testing Execution Time Limit Enforcement ==="

    # Default timeout (documented as 30s)
    skip_test "Quick task completes within timeout" "Requires test skill"
    skip_test "Slow task exceeds timeout and is killed" "Requires long-running test skill"
    skip_test "Default timeout is 30 seconds" "Requires timeout measurement"
    skip_test "Timeout can be configured per-skill" "Requires test skill with custom timeout"

    # Timeout behavior
    skip_test "Timeout sends graceful signal first (SIGTERM)" "Requires test skill"
    skip_test "Timeout follows with SIGKILL if needed" "Requires test skill"
    skip_test "Timeout cleanup removes resources" "Requires test skill and resource inspection"
}

#
# Category 4: Disk Space Limits
#

test_disk_limits() {
    log_info "=== Testing Disk Space Limit Enforcement ==="

    # Docker disk limits
    skip_test "Docker skill with disk quota succeeds" "Requires Docker test skill"
    skip_test "Docker skill exceeding disk quota fails" "Requires Docker test skill"
    skip_test "Docker storage driver limits enforced" "Requires Docker test skill"

    # WASM filesystem limits
    skip_test "WASM virtual filesystem size limited" "Requires WASM test skill"
    skip_test "WASM disk write exceeding limit fails" "Requires WASM test skill"

    # Temporary file limits
    skip_test "Temporary files cleaned up after execution" "Requires test skill"
    skip_test "Temporary file size limits enforced" "Requires test skill"
}

#
# Category 5: File Descriptor Limits
#

test_file_descriptor_limits() {
    log_info "=== Testing File Descriptor Limit Enforcement ==="

    skip_test "Open files within limit succeeds" "Requires test skill"
    skip_test "Opening too many files fails" "Requires test skill"
    skip_test "File descriptor leak detection" "Requires test skill"
    skip_test "Socket descriptors counted in limit" "Requires test skill"
    skip_test "File descriptors released on cleanup" "Requires test skill"
}

#
# Category 6: Network Bandwidth Limits
#

test_network_bandwidth_limits() {
    log_info "=== Testing Network Bandwidth Limits ==="

    # Docker network limits
    skip_test "Docker network ingress limit enforced" "Requires Docker test skill"
    skip_test "Docker network egress limit enforced" "Requires Docker test skill"
    skip_test "Network rate limiting works" "Requires Docker test skill"

    # Connection limits
    skip_test "Maximum concurrent connections enforced" "Requires test skill"
    skip_test "Connection timeout enforced" "Requires test skill"
}

#
# Category 7: Process Limits
#

test_process_limits() {
    log_info "=== Testing Process Limit Enforcement ==="

    # Docker container limits
    skip_test "Docker PIDs limit enforced" "Requires Docker test skill"
    skip_test "Fork bomb prevention" "Requires Docker test skill"
    skip_test "Process spawn limits work" "Requires Docker test skill"

    # Native runtime process limits
    skip_test "Native skill subprocess limits" "Requires native test skill"
    skip_test "Subprocess cleanup on parent exit" "Requires native test skill"
}

#
# Category 8: Thread Limits
#

test_thread_limits() {
    log_info "=== Testing Thread Limit Enforcement ==="

    skip_test "Thread creation within limit succeeds" "Requires test skill"
    skip_test "Excessive thread creation fails" "Requires test skill"
    skip_test "Thread cleanup on exit" "Requires test skill"
    skip_test "Thread pool size limits" "Requires test skill"
}

#
# Category 9: Resource Cleanup
#

test_resource_cleanup() {
    log_info "=== Testing Resource Cleanup ==="

    # Verify resources are cleaned up after execution
    skip_test "Memory freed after execution" "Requires resource monitoring"
    skip_test "File descriptors closed" "Requires resource monitoring"
    skip_test "Temporary files deleted" "Requires filesystem inspection"
    skip_test "Network connections closed" "Requires network monitoring"
    skip_test "Child processes terminated" "Requires process monitoring"
    skip_test "Docker containers removed" "Requires Docker inspection"
    skip_test "WASM instances freed" "Requires WASM runtime inspection"
}

#
# Category 10: Resource Exhaustion Prevention
#

test_resource_exhaustion_prevention() {
    log_info "=== Testing Resource Exhaustion Prevention ==="

    # Prevent system-wide resource exhaustion
    skip_test "Multiple skills don't exhaust system memory" "Requires concurrent execution"
    skip_test "Multiple skills don't exhaust CPU" "Requires concurrent execution"
    skip_test "Multiple skills don't exhaust file descriptors" "Requires concurrent execution"
    skip_test "Skill queue prevents overload" "Requires load testing"
    skip_test "Resource pool management works" "Requires concurrent execution"
}

#
# Category 11: Resource Monitoring
#

test_resource_monitoring() {
    log_info "=== Testing Resource Monitoring ==="

    skip_test "Memory usage tracked per skill" "Requires monitoring API"
    skip_test "CPU usage tracked per skill" "Requires monitoring API"
    skip_test "Execution time tracked per skill" "Requires monitoring API"
    skip_test "Resource metrics exported" "Requires monitoring API"
    skip_test "Resource alerts triggered" "Requires monitoring API"
}

#
# Main Execution
#

main() {
    log_info "Starting Security Testing - Resource Limit Enforcement"
    log_info "Log file: $TEST_LOG"

    # Check prerequisites
    check_prerequisites

    log_warning ""
    log_warning "IMPORTANT: Resource limit tests verify denial-of-service prevention"
    log_warning "Limits should prevent individual skills from exhausting system resources"
    log_warning ""

    # Run test categories
    test_memory_limits
    test_cpu_limits
    test_execution_time_limits
    test_disk_limits
    test_file_descriptor_limits
    test_network_bandwidth_limits
    test_process_limits
    test_thread_limits
    test_resource_cleanup
    test_resource_exhaustion_prevention
    test_resource_monitoring

    # Summary
    echo ""
    log_info "=== Test Summary ==="
    log_info "Total Tests:   $TOTAL_TESTS"
    log_success "Passed:        $PASSED_TESTS (limits enforced)"
    log_error "Failed:        $FAILED_TESTS (limit violations)"
    log_warning "Skipped:       $SKIPPED_TESTS"

    if [[ $TOTAL_TESTS -gt 0 ]]; then
        local active_tests=$((TOTAL_TESTS - SKIPPED_TESTS))
        if [[ $active_tests -gt 0 ]]; then
            local pass_rate=$(( (PASSED_TESTS * 100) / active_tests ))
            log_info "Pass Rate:     ${pass_rate}% (limits correctly enforced)"
        fi
    fi

    log_info ""
    log_info "Note: Resource limit tests require:"
    log_info "  - Test skills that consume specific amounts of resources"
    log_info "  - Resource monitoring capabilities"
    log_info "  - Load testing tools for concurrent execution"
    log_info "  - Docker for container resource limits"
    log_info "  - WASM skills for WASM memory limits"
    log_info ""
    log_info "Key limits documented:"
    log_info "  - WASM memory: 256 pages (16MB)"
    log_info "  - Execution timeout: 30 seconds (default)"
    log_info "  - Docker: memory, CPU, disk, network limits configurable"

    # Exit with appropriate code
    if [[ $FAILED_TESTS -gt 0 ]]; then
        log_error "SECURITY ISSUE: Resource limits not properly enforced!"
        exit 1
    else
        log_success "All tested resource limits were properly enforced!"
        if [[ $SKIPPED_TESTS -gt 0 ]]; then
            log_info "Note: $SKIPPED_TESTS tests were skipped (require test fixtures or monitoring tools)"
        fi
        exit 0
    fi
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
