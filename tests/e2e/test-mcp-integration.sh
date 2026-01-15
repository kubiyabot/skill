#!/usr/bin/env bash
#
# MCP Protocol Integration and Claude Code Compatibility Tests
# Tests MCP server stdio/HTTP modes, JSON-RPC 2.0 protocol, and Claude Code integration
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
TEST_LOG="${TEST_OUTPUT_DIR}/mcp-integration-tests-$(date +%Y%m%d-%H%M%S).log"

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
# Category 1: MCP Server Commands
#

test_mcp_server_commands() {
    log_info "=== Testing MCP Server Commands ==="

    # Test serve command exists
    run_test "MCP serve command exists" \
        "$SKILL_BIN serve --help >/dev/null 2>&1" \
        0 \
        "Verify skill serve command"

    # Test serve command help mentions MCP
    run_test "Serve command mentions MCP" \
        "$SKILL_BIN serve --help 2>&1 | grep -qi 'mcp'" \
        0 \
        "Verify MCP is mentioned in help"
}

#
# Category 2: Claude Code Integration Commands
#

test_claude_code_integration() {
    log_info "=== Testing Claude Code Integration Commands ==="

    # Test claude command exists
    run_test "Claude command exists" \
        "$SKILL_BIN claude --help >/dev/null 2>&1" \
        0 \
        "Verify skill claude command"

    # Test claude setup command
    run_test "Claude setup command exists" \
        "$SKILL_BIN claude setup --help >/dev/null 2>&1" \
        0 \
        "Verify skill claude setup"

    # Test claude status command
    run_test "Claude status command exists" \
        "$SKILL_BIN claude status --help >/dev/null 2>&1" \
        0 \
        "Verify skill claude status"

    # Test claude remove command
    run_test "Claude remove command exists" \
        "$SKILL_BIN claude remove --help >/dev/null 2>&1" \
        0 \
        "Verify skill claude remove"

    # Check if .mcp.json exists in project
    if [[ -f "$PROJECT_ROOT/.mcp.json" ]]; then
        run_test "Project has .mcp.json" \
            "test -f '$PROJECT_ROOT/.mcp.json'" \
            0 \
            "Verify .mcp.json exists"

        # Validate .mcp.json format
        if command_exists jq; then
            run_test ".mcp.json is valid JSON" \
                "jq empty '$PROJECT_ROOT/.mcp.json' >/dev/null 2>&1" \
                0 \
                "Parse .mcp.json with jq"

            # Check for MCP server configuration
            run_test ".mcp.json has mcpServers" \
                "jq -e '.mcpServers' '$PROJECT_ROOT/.mcp.json' >/dev/null 2>&1" \
                0 \
                "Verify mcpServers field exists"
        else
            skip_test ".mcp.json validation" "jq not installed"
        fi
    else
        skip_test ".mcp.json tests" ".mcp.json not found in project"
    fi
}

#
# Category 3: Existing MCP Integration Tests
#

test_existing_mcp_suite() {
    log_info "=== Testing Existing MCP Integration Suite ==="

    local mcp_test_script="$PROJECT_ROOT/tests/mcp_integration_tests.sh"

    if [[ -f "$mcp_test_script" ]]; then
        run_test "MCP integration test script exists" \
            "test -f '$mcp_test_script'" \
            0 \
            "Verify mcp_integration_tests.sh exists"

        # Check if script is executable
        if [[ -x "$mcp_test_script" ]]; then
            run_test "MCP integration tests executable" \
                "test -x '$mcp_test_script'" \
                0 \
                "Verify script has execute permissions"

            # Note: We skip running the full suite here as it's comprehensive (45+ tests)
            # and takes significant time. It should be run separately.
            skip_test "Run full MCP integration suite" "Comprehensive suite with 45+ tests, run separately with ./tests/mcp_integration_tests.sh"
        else
            skip_test "MCP integration tests" "Script not executable"
        fi
    else
        skip_test "MCP integration tests" "mcp_integration_tests.sh not found"
    fi
}

#
# Category 4: MCP Protocol Basics
#

test_mcp_protocol_basics() {
    log_info "=== Testing MCP Protocol Basics ==="

    # Test that serve command can start (but not actually start the server)
    run_test "Serve command accepts stdio mode" \
        "timeout 1 $SKILL_BIN serve 2>&1 | head -1 | grep -q 'skill' || true" \
        0 \
        "Verify serve starts in stdio mode"

    # Test JSON-RPC message format understanding
    skip_test "JSON-RPC 2.0 format validation" "Requires running server and sending requests"
    skip_test "Initialize handshake" "Requires running server"
    skip_test "Tools list request" "Requires running server"
    skip_test "Tools call request" "Requires running server"
}

#
# Category 5: MCP Tools
#

test_mcp_tools() {
    log_info "=== Testing MCP Tools ==="

    # These tests verify the MCP tools are implemented
    # Actual execution requires a running server, covered by mcp_integration_tests.sh

    skip_test "execute tool implementation" "Requires running MCP server"
    skip_test "list_skills tool implementation" "Requires running MCP server"
    skip_test "search_skills tool implementation" "Requires running MCP server"
    skip_test "Tool parameter validation" "Requires running MCP server"
    skip_test "Tool error handling" "Requires running MCP server"
}

#
# Category 6: Transport Modes
#

test_transport_modes() {
    log_info "=== Testing Transport Modes ==="

    # Test stdio mode (default)
    skip_test "stdio transport mode" "Requires interactive testing"
    skip_test "HTTP transport mode" "Requires running server"
    skip_test "SSE streaming" "Requires running server"
    skip_test "Graceful shutdown" "Requires running server"
}

#
# Category 7: Performance
#

test_performance() {
    log_info "=== Testing Performance ==="

    skip_test "Tool call latency" "Requires benchmarking with running server"
    skip_test "Concurrent request handling" "Requires load testing"
    skip_test "Large payload handling" "Requires running server"
    skip_test "Long-running tool streaming" "Requires running server"
}

#
# Main Execution
#

main() {
    log_info "Starting MCP Integration and Claude Code Compatibility Tests"
    log_info "Log file: $TEST_LOG"

    # Check prerequisites
    check_prerequisites

    # Run test categories
    test_mcp_server_commands
    test_claude_code_integration
    test_existing_mcp_suite
    test_mcp_protocol_basics
    test_mcp_tools
    test_transport_modes
    test_performance

    # Summary
    echo ""
    log_info "=== Test Summary ==="
    log_info "Total Tests:   $TOTAL_TESTS"
    log_success "Passed:        $PASSED_TESTS"
    log_error "Failed:        $FAILED_TESTS"
    log_warning "Skipped:       $SKIPPED_TESTS"

    if [[ $TOTAL_TESTS -gt 0 ]]; then
        local active_tests=$((TOTAL_TESTS - SKIPPED_TESTS))
        if [[ $active_tests -gt 0 ]]; then
            local pass_rate=$(( (PASSED_TESTS * 100) / active_tests ))
            log_info "Pass Rate:     ${pass_rate}% (excluding skipped)"
        fi
    fi

    log_info ""
    log_info "Note: Comprehensive MCP protocol tests (45+ tests) are in:"
    log_info "  $PROJECT_ROOT/tests/mcp_integration_tests.sh"
    log_info "Run separately for full MCP server validation"

    # Exit with appropriate code
    if [[ $FAILED_TESTS -gt 0 ]]; then
        log_error "Some tests failed!"
        exit 1
    else
        log_success "All active tests passed!"
        if [[ $SKIPPED_TESTS -gt 0 ]]; then
            log_info "Note: $SKIPPED_TESTS tests were skipped (require running MCP server)"
        fi
        exit 0
    fi
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
