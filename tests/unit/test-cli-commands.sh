#!/usr/bin/env bash
#
# CLI Command Testing Framework
# Tests all documented CLI commands, flags, and options
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Test results array
declare -a TEST_RESULTS=()

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Check if skill binary exists
SKILL_BIN="${SKILL_BIN:-skill}"

# Test output directory
TEST_OUTPUT_DIR="${PROJECT_ROOT}/tests/output"
mkdir -p "$TEST_OUTPUT_DIR"

# Test log file
TEST_LOG="${TEST_OUTPUT_DIR}/cli-tests-$(date +%Y%m%d-%H%M%S).log"

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

# Run a test case
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

    # Run command and capture output and exit code
    local output
    local exit_code
    set +e
    output=$(eval "$test_command" 2>&1)
    exit_code=$?
    set -e

    # Check exit code
    if [[ $exit_code -eq $expected_exit_code ]]; then
        log_success "$test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        TEST_RESULTS+=("PASS|$test_name|$test_command")
        return 0
    else
        log_error "$test_name"
        log "  Expected exit code: $expected_exit_code, got: $exit_code"
        log "  Output: $output"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        TEST_RESULTS+=("FAIL|$test_name|$test_command|Expected: $expected_exit_code, Got: $exit_code")
        return 1
    fi
}

# Skip a test with reason
skip_test() {
    local test_name="$1"
    local reason="$2"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    SKIPPED_TESTS=$((SKIPPED_TESTS + 1))

    log_skip "$test_name - $reason"
    TEST_RESULTS+=("SKIP|$test_name|$reason")
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if skill binary is available
check_skill_binary() {
    if ! command_exists "$SKILL_BIN"; then
        log_error "skill binary not found at: $SKILL_BIN"
        log "Set SKILL_BIN environment variable to the correct path"
        exit 1
    fi

    log_info "Using skill binary: $(which $SKILL_BIN)"
    log_info "Version: $($SKILL_BIN --version 2>&1 || echo 'unknown')"
}

#
# Test Categories
#

# Category 1: Installation & Setup Commands
test_installation_setup() {
    log_info "=== Testing Installation & Setup Commands ==="

    run_test "skill --version" \
        "$SKILL_BIN --version" \
        0 \
        "Display version information"

    run_test "skill --help" \
        "$SKILL_BIN --help" \
        0 \
        "Display help information"

    run_test "skill help" \
        "$SKILL_BIN help" \
        0 \
        "Display help via subcommand"

    # Test setup command
    run_test "skill setup --help" \
        "$SKILL_BIN setup --help" \
        0 \
        "Display setup help"

    # Test upgrade command
    run_test "skill upgrade --help" \
        "$SKILL_BIN upgrade --help" \
        0 \
        "Display upgrade help"

    # Test auth command
    run_test "skill auth --help" \
        "$SKILL_BIN auth --help" \
        0 \
        "Display auth help"
}

# Category 2: Skill Discovery Commands
test_skill_discovery() {
    log_info "=== Testing Skill Discovery Commands ==="

    run_test "skill list" \
        "$SKILL_BIN list" \
        0 \
        "List all skills"

    run_test "skill list --help" \
        "$SKILL_BIN list --help" \
        0 \
        "Display list help"

    # Test info command
    run_test "skill info --help" \
        "$SKILL_BIN info --help" \
        0 \
        "Display info help"

    # Test find command
    run_test "skill find --help" \
        "$SKILL_BIN find --help" \
        0 \
        "Display find help"

    run_test "skill search --help" \
        "$SKILL_BIN search --help" \
        0 \
        "Display search help"
}

# Category 3: Skill Execution Commands
test_skill_execution() {
    log_info "=== Testing Skill Execution Commands ==="

    run_test "skill run --help" \
        "$SKILL_BIN run --help" \
        0 \
        "Display run help"

    run_test "skill exec --help" \
        "$SKILL_BIN exec --help" \
        0 \
        "Display exec help"

    # Test that run requires skill argument
    run_test "skill run (no args)" \
        "$SKILL_BIN run 2>&1; test \$? -ne 0" \
        0 \
        "Run command should fail without arguments"
}

# Category 4: Skill Management Commands
test_skill_management() {
    log_info "=== Testing Skill Management Commands ==="

    run_test "skill install --help" \
        "$SKILL_BIN install --help" \
        0 \
        "Display install help"

    run_test "skill remove --help" \
        "$SKILL_BIN remove --help" \
        0 \
        "Display remove help"

    run_test "skill init --help" \
        "$SKILL_BIN init --help" \
        0 \
        "Display init help"

}

# Category 5: Configuration Commands
test_configuration() {
    log_info "=== Testing Configuration Commands ==="

    run_test "skill config --help" \
        "$SKILL_BIN config --help" \
        0 \
        "Display config help"
}

# Category 6: MCP Server Commands
test_mcp_server() {
    log_info "=== Testing MCP Server Commands ==="

    run_test "skill serve --help" \
        "$SKILL_BIN serve --help" \
        0 \
        "Display serve help"

    # Test claude commands if they exist
    if $SKILL_BIN help 2>&1 | grep -q "claude"; then
        run_test "skill claude --help" \
            "$SKILL_BIN claude --help" \
            0 \
            "Display claude help"
    else
        skip_test "skill claude" "Claude commands not available"
    fi
}

# Category 7: Web Interface Commands
test_web_interface() {
    log_info "=== Testing Web Interface Commands ==="

    run_test "skill web --help" \
        "$SKILL_BIN web --help" \
        0 \
        "Display web help"
}

# Category 8: Enhancement Commands
test_enhancement() {
    log_info "=== Testing Enhancement Commands ==="

    # Test enhance and init-skill which are documented
    run_test "skill enhance --help" \
        "$SKILL_BIN enhance --help" \
        0 \
        "Display enhance help"

    run_test "skill init-skill --help" \
        "$SKILL_BIN init-skill --help" \
        0 \
        "Display init-skill help"
}

# Category 9: History Commands
test_history() {
    log_info "=== Testing History Commands ==="

    # Check if history command exists
    if $SKILL_BIN help 2>&1 | grep -q "history"; then
        run_test "skill history --help" \
            "$SKILL_BIN history --help" \
            0 \
            "Display history help"
    else
        skip_test "skill history" "History command not available"
    fi
}

# Category 10: Error Handling
test_error_handling() {
    log_info "=== Testing Error Handling ==="

    run_test "invalid command" \
        "$SKILL_BIN invalid-command-xyz 2>&1; test \$? -ne 0" \
        0 \
        "Invalid command should fail"

    run_test "invalid flag" \
        "$SKILL_BIN --invalid-flag-xyz 2>&1; test \$? -ne 0" \
        0 \
        "Invalid flag should fail"
}

#
# Test Report Generation
#

generate_report() {
    local report_file="${TEST_OUTPUT_DIR}/test-report.html"

    log_info "Generating HTML test report: $report_file"

    cat > "$report_file" <<EOF
<!DOCTYPE html>
<html>
<head>
    <title>Skill Engine CLI Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #333; }
        .summary { background: #f5f5f5; padding: 15px; border-radius: 5px; margin: 20px 0; }
        .summary-item { display: inline-block; margin-right: 30px; }
        .pass { color: #28a745; font-weight: bold; }
        .fail { color: #dc3545; font-weight: bold; }
        .skip { color: #ffc107; font-weight: bold; }
        table { border-collapse: collapse; width: 100%; margin-top: 20px; }
        th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }
        th { background-color: #4CAF50; color: white; }
        tr:nth-child(even) { background-color: #f2f2f2; }
        .status-pass { background-color: #d4edda; }
        .status-fail { background-color: #f8d7da; }
        .status-skip { background-color: #fff3cd; }
    </style>
</head>
<body>
    <h1>Skill Engine CLI Test Report</h1>
    <p>Generated: $(date)</p>

    <div class="summary">
        <h2>Summary</h2>
        <div class="summary-item">Total Tests: <strong>$TOTAL_TESTS</strong></div>
        <div class="summary-item pass">Passed: $PASSED_TESTS</div>
        <div class="summary-item fail">Failed: $FAILED_TESTS</div>
        <div class="summary-item skip">Skipped: $SKIPPED_TESTS</div>
        <div class="summary-item">Pass Rate: <strong>$(( TOTAL_TESTS > 0 ? (PASSED_TESTS * 100) / TOTAL_TESTS : 0 ))%</strong></div>
    </div>

    <h2>Test Results</h2>
    <table>
        <tr>
            <th>Status</th>
            <th>Test Name</th>
            <th>Command</th>
            <th>Details</th>
        </tr>
EOF

    for result in "${TEST_RESULTS[@]}"; do
        IFS='|' read -r status name command details <<< "$result"
        local row_class=""
        case "$status" in
            PASS) row_class="status-pass" ;;
            FAIL) row_class="status-fail" ;;
            SKIP) row_class="status-skip" ;;
        esac

        cat >> "$report_file" <<EOF
        <tr class="$row_class">
            <td><span class="$(echo $status | tr '[:upper:]' '[:lower:]')">$status</span></td>
            <td>$name</td>
            <td><code>$command</code></td>
            <td>${details:-}</td>
        </tr>
EOF
    done

    cat >> "$report_file" <<EOF
    </table>
</body>
</html>
EOF

    log_info "Report generated: $report_file"
}

#
# Main Execution
#

main() {
    log_info "Starting Skill Engine CLI Command Tests"
    log_info "Log file: $TEST_LOG"

    # Check prerequisites
    check_skill_binary

    # Run test categories
    test_installation_setup
    test_skill_discovery
    test_skill_execution
    test_skill_management
    test_configuration
    test_mcp_server
    test_web_interface
    test_enhancement
    test_history
    test_error_handling

    # Generate report
    echo ""
    log_info "=== Test Summary ==="
    log_info "Total Tests:   $TOTAL_TESTS"
    log_success "Passed:        $PASSED_TESTS"
    log_error "Failed:        $FAILED_TESTS"
    log_warning "Skipped:       $SKIPPED_TESTS"

    if [[ $TOTAL_TESTS -gt 0 ]]; then
        local pass_rate=$(( (PASSED_TESTS * 100) / TOTAL_TESTS ))
        log_info "Pass Rate:     ${pass_rate}%"
    fi

    # Generate HTML report
    generate_report

    # Exit with appropriate code
    if [[ $FAILED_TESTS -gt 0 ]]; then
        log_error "Some tests failed!"
        exit 1
    else
        log_success "All tests passed!"
        exit 0
    fi
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
