#!/bin/bash
# test-fresh-install.sh - Test fresh installation workflow
# Tests that skill-cli can be installed and used from scratch

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_OUTPUT_DIR="${TEST_OUTPUT_DIR:-/tmp/skill-test-output}"
REPORT_FILE="$SCRIPT_DIR/reports/fresh-install-report.json"
VERBOSE="${VERBOSE:-false}"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0
START_TIME=$(date +%s)

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

verbose() {
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${BLUE}[DEBUG]${NC} $1"
    fi
}

run_test() {
    local test_name="$1"
    local test_command="$2"

    ((TESTS_RUN++))
    verbose "Running test: $test_name"
    verbose "Command: $test_command"

    if eval "$test_command" > /dev/null 2>&1; then
        log_success "$test_name"
        return 0
    else
        log_fail "$test_name"
        return 1
    fi
}

# Setup
setup() {
    log_info "Setting up fresh install test environment..."

    # Create test directories
    mkdir -p "$TEST_OUTPUT_DIR"
    mkdir -p "$(dirname "$REPORT_FILE")"

    # Clean previous test artifacts
    rm -f "$REPORT_FILE"

    log_info "Test environment ready"
}

# Test: skill-cli binary exists
test_binary_exists() {
    run_test "skill-cli binary exists" "[ -f '$PROJECT_ROOT/target/release/skill' ]"
}

# Test: skill binary is executable
test_binary_executable() {
    run_test "skill-cli binary is executable" "[ -x '$PROJECT_ROOT/target/release/skill' ]"
}

# Test: skill --version works
test_version_command() {
    local output
    ((TESTS_RUN++))

    if output=$("$PROJECT_ROOT/target/release/skill" --version 2>&1); then
        verbose "Version output: $output"
        if [[ "$output" =~ skill.*[0-9]+\.[0-9]+\.[0-9]+ ]]; then
            log_success "skill --version returns valid version"
            return 0
        else
            log_fail "skill --version output format invalid: $output"
            return 1
        fi
    else
        log_fail "skill --version command failed"
        return 1
    fi
}

# Test: skill --help works
test_help_command() {
    local output
    ((TESTS_RUN++))

    if output=$("$PROJECT_ROOT/target/release/skill" --help 2>&1); then
        verbose "Help output length: ${#output} characters"
        if [[ "$output" =~ Usage ]]; then
            log_success "skill --help returns usage information"
            return 0
        else
            log_fail "skill --help missing Usage section"
            return 1
        fi
    else
        log_fail "skill --help command failed"
        return 1
    fi
}

# Test: skill claude --help works
test_claude_help_command() {
    local output
    ((TESTS_RUN++))

    if output=$("$PROJECT_ROOT/target/release/skill" claude --help 2>&1); then
        verbose "Claude help output length: ${#output} characters"
        if [[ "$output" =~ generate ]] || [[ "$output" =~ Claude ]]; then
            log_success "skill claude --help returns subcommand information"
            return 0
        else
            log_fail "skill claude --help missing expected content"
            return 1
        fi
    else
        log_fail "skill claude --help command failed"
        return 1
    fi
}

# Test: skill can be added to PATH
test_path_integration() {
    ((TESTS_RUN++))

    local temp_path="$PROJECT_ROOT/target/release:$PATH"
    if PATH="$temp_path" command -v skill > /dev/null 2>&1; then
        log_success "skill-cli can be found in PATH"
        return 0
    else
        log_fail "skill-cli not found in PATH after adding"
        return 1
    fi
}

# Test: Basic skill generation works
test_basic_generation() {
    ((TESTS_RUN++))

    local test_output="$TEST_OUTPUT_DIR/fresh-install-test"
    rm -rf "$test_output"
    mkdir -p "$test_output"

    verbose "Testing generation to: $test_output"

    if "$PROJECT_ROOT/target/release/skill" claude generate \
        --output "$test_output" \
        --dry-run > /dev/null 2>&1; then
        log_success "skill claude generate --dry-run succeeds"
        return 0
    else
        log_fail "skill claude generate --dry-run failed"
        return 1
    fi
}

# Test: skill-cli can read manifest
test_manifest_reading() {
    ((TESTS_RUN++))

    # Check if manifest exists
    if [[ ! -f "$PROJECT_ROOT/.skill-engine.toml" ]]; then
        log_warn "No manifest found at $PROJECT_ROOT/.skill-engine.toml (skipping test)"
        return 0
    fi

    # Try to list skills (requires valid manifest)
    if "$PROJECT_ROOT/target/release/skill" list > /dev/null 2>&1; then
        log_success "skill-cli can read and parse manifest"
        return 0
    else
        log_fail "skill-cli failed to read manifest"
        return 1
    fi
}

# Test: Environment detection works
test_environment_detection() {
    ((TESTS_RUN++))

    local os_type=$(uname -s)
    verbose "Detected OS: $os_type"

    case "$os_type" in
        Darwin)
            log_success "Environment detection: macOS"
            return 0
            ;;
        Linux)
            log_success "Environment detection: Linux"
            return 0
            ;;
        *)
            log_warn "Environment detection: Unknown OS ($os_type)"
            return 0
            ;;
    esac
}

# Test: Dependencies are available
test_dependencies() {
    ((TESTS_RUN++))

    local missing_deps=()

    # Check for required system tools
    for cmd in cargo rustc; do
        if ! command -v "$cmd" > /dev/null 2>&1; then
            missing_deps+=("$cmd")
        fi
    done

    if [[ ${#missing_deps[@]} -eq 0 ]]; then
        log_success "All required dependencies are installed"
        return 0
    else
        log_fail "Missing dependencies: ${missing_deps[*]}"
        return 1
    fi
}

# Generate JSON report
generate_report() {
    local end_time=$(date +%s)
    local duration=$((end_time - START_TIME))
    local success_rate=0

    if [[ $TESTS_RUN -gt 0 ]]; then
        success_rate=$(awk "BEGIN {printf \"%.2f\", ($TESTS_PASSED / $TESTS_RUN) * 100}")
    fi

    cat > "$REPORT_FILE" << EOF
{
  "test_suite": "fresh-install",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "duration_seconds": $duration,
  "summary": {
    "total": $TESTS_RUN,
    "passed": $TESTS_PASSED,
    "failed": $TESTS_FAILED,
    "success_rate": "$success_rate%"
  },
  "environment": {
    "os": "$(uname -s)",
    "os_version": "$(uname -r)",
    "shell": "$SHELL",
    "project_root": "$PROJECT_ROOT"
  },
  "tests": [
    {"name": "binary_exists", "status": "$([ -f "$PROJECT_ROOT/target/release/skill" ] && echo passed || echo failed)"},
    {"name": "binary_executable", "status": "$([ -x "$PROJECT_ROOT/target/release/skill" ] && echo passed || echo failed)"},
    {"name": "version_command", "status": "passed_or_failed"},
    {"name": "help_command", "status": "passed_or_failed"},
    {"name": "claude_help_command", "status": "passed_or_failed"},
    {"name": "path_integration", "status": "passed_or_failed"},
    {"name": "basic_generation", "status": "passed_or_failed"},
    {"name": "manifest_reading", "status": "passed_or_failed"},
    {"name": "environment_detection", "status": "passed_or_failed"},
    {"name": "dependencies", "status": "passed_or_failed"}
  ]
}
EOF

    verbose "Report generated: $REPORT_FILE"
}

# Print summary
print_summary() {
    echo ""
    echo "=================================="
    echo " Fresh Install Test Results"
    echo "=================================="
    echo ""
    echo "  Total Tests:  $TESTS_RUN"
    echo "  Passed:       $GREEN$TESTS_PASSED$NC"
    echo "  Failed:       $RED$TESTS_FAILED$NC"
    echo ""

    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "${GREEN}✓ All tests passed!${NC}"
        echo ""
        echo "Next steps:"
        echo "  1. Run: ./tests/claude_bridge/test-skill-generation.sh"
        echo "  2. Or run all tests: ./tests/claude_bridge/test-all.sh"
    else
        echo -e "${RED}✗ Some tests failed${NC}"
        echo ""
        echo "Check the report: $REPORT_FILE"
    fi

    echo "=================================="
}

# Cleanup
cleanup() {
    verbose "Cleaning up test artifacts..."
    # Keep test output for debugging
    verbose "Test output preserved in: $TEST_OUTPUT_DIR"
}

# Main execution
main() {
    log_info "Starting Fresh Install Tests..."
    log_info "Project: $PROJECT_ROOT"
    echo ""

    setup

    # Run all tests
    test_binary_exists
    test_binary_executable
    test_version_command
    test_help_command
    test_claude_help_command
    test_path_integration
    test_basic_generation
    test_manifest_reading
    test_environment_detection
    test_dependencies

    # Generate report and summary
    generate_report
    print_summary
    cleanup

    # Exit with appropriate code
    if [[ $TESTS_FAILED -gt 0 ]]; then
        exit 1
    else
        exit 0
    fi
}

# Handle arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [-v|--verbose] [-h|--help]"
            echo ""
            echo "Options:"
            echo "  -v, --verbose    Enable verbose output"
            echo "  -h, --help       Show this help message"
            exit 0
            ;;
        *)
            log_warn "Unknown option: $1"
            shift
            ;;
    esac
done

# Run main
main "$@"
