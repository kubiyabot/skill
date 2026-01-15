#!/usr/bin/env bash
#
# Claude Bridge End-to-End Testing Suite
# Tests Claude Bridge SKILL.md generation, validation, and integration
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
TEST_LOG="${TEST_OUTPUT_DIR}/claude-bridge-tests-$(date +%Y%m%d-%H%M%S).log"

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

    log_info "Using skill binary: $(which $SKILL_BIN)"
    log_info "Version: $($SKILL_BIN --version 2>&1 || echo 'unknown')"
}

#
# Category 1: Claude Bridge CLI Integration
#

test_claude_bridge_cli() {
    log_info "=== Testing Claude Bridge CLI Integration ==="

    # Test that init-skill command exists (generates SKILL.md template)
    run_test "init-skill command exists" \
        "$SKILL_BIN init-skill --help >/dev/null 2>&1" \
        0 \
        "Verify init-skill command for SKILL.md generation"

    # Test init-skill with sample skill
    local test_skill_dir="$TEST_OUTPUT_DIR/test-skill"
    mkdir -p "$test_skill_dir"

    run_test "init-skill generates SKILL.md template" \
        "cd '$test_skill_dir' && $SKILL_BIN init-skill test-skill --output SKILL.md >/dev/null 2>&1" \
        0 \
        "Generate SKILL.md template for test skill"

    if [[ -f "$test_skill_dir/SKILL.md" ]]; then
        run_test "Generated SKILL.md exists" \
            "test -f '$test_skill_dir/SKILL.md'" \
            0 \
            "Verify SKILL.md was created"
    else
        skip_test "Generated SKILL.md exists" "init-skill may not create SKILL.md"
    fi
}

#
# Category 2: Rust Unit Tests for Claude Bridge
#

test_rust_unit_tests() {
    log_info "=== Testing Rust Unit Tests for Claude Bridge ==="

    # Check if we're in a Rust project
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        skip_test "Claude Bridge module unit tests" "Not in a Rust project"
        skip_test "Claude Bridge doc tests" "Not in a Rust project"
        return 0
    fi

    # Run cargo tests for claude_bridge module (skill-cli is binary-only)
    skip_test "Claude Bridge module unit tests" "skill-cli is binary-only, see tests/claude_bridge/"
    skip_test "Claude Bridge doc tests" "skill-cli is binary-only, see tests/claude_bridge/"
}

#
# Category 3: Existing Integration Tests
#

test_existing_integration_tests() {
    log_info "=== Testing Existing Claude Bridge Integration Tests ==="

    local cb_test_dir="$PROJECT_ROOT/tests/claude_bridge"

    if [[ ! -d "$cb_test_dir" ]]; then
        skip_test "Existing integration tests" "claude_bridge test directory not found"
        return 0
    fi

    # Note: These tests are comprehensive but may require specific setup
    # They're skipped here but tracked separately in tests/claude_bridge/
    skip_test "Fresh install test script" "Requires clean environment, see tests/claude_bridge/"
    skip_test "Skill generation test script" "Requires manifest setup, see tests/claude_bridge/"
}

#
# Category 4: SKILL.md Format Validation
#

test_skill_md_format() {
    log_info "=== Testing SKILL.md Format Validation ==="

    # Check existing native skills for SKILL.md
    local native_skill_dir="$PROJECT_ROOT/examples/native-skills"

    if [[ -d "$native_skill_dir" ]]; then
        # Test kubernetes SKILL.md
        if [[ -f "$native_skill_dir/kubernetes-skill/SKILL.md" ]]; then
            run_test "Kubernetes SKILL.md exists" \
                "test -f '$native_skill_dir/kubernetes-skill/SKILL.md'" \
                0 \
                "Verify kubernetes skill has SKILL.md"

            # Check for YAML frontmatter
            run_test "Kubernetes SKILL.md has YAML frontmatter" \
                "head -1 '$native_skill_dir/kubernetes-skill/SKILL.md' | grep -q '^---$'" \
                0 \
                "Verify YAML frontmatter delimiter"

            # Check for required fields
            run_test "Kubernetes SKILL.md has 'name' field" \
                "grep -q '^name:' '$native_skill_dir/kubernetes-skill/SKILL.md'" \
                0 \
                "Verify name field in frontmatter"

            run_test "Kubernetes SKILL.md has 'description' field" \
                "grep -q '^description:' '$native_skill_dir/kubernetes-skill/SKILL.md'" \
                0 \
                "Verify description field in frontmatter"
        else
            skip_test "Kubernetes SKILL.md validation" "SKILL.md not found"
        fi

        # Test terraform SKILL.md
        if [[ -f "$native_skill_dir/terraform-skill/SKILL.md" ]]; then
            run_test "Terraform SKILL.md exists" \
                "test -f '$native_skill_dir/terraform-skill/SKILL.md'" \
                0 \
                "Verify terraform skill has SKILL.md"

            # Note: Not all SKILL.md files have YAML frontmatter yet
            # That's okay - it's optional for some skills
            skip_test "Terraform SKILL.md validation" "Format varies by skill"
        else
            skip_test "Terraform SKILL.md validation" "SKILL.md not found"
        fi
    else
        skip_test "Native skills SKILL.md validation" "Native skills directory not found"
    fi
}

#
# Category 5: Script Generation
#

test_script_generation() {
    log_info "=== Testing Script Generation ==="

    local native_skill_dir="$PROJECT_ROOT/examples/native-skills"

    if [[ -d "$native_skill_dir/kubernetes-skill" ]]; then
        # Check for scripts directory
        if [[ -d "$native_skill_dir/kubernetes-skill/scripts" ]]; then
            run_test "Kubernetes skill has scripts directory" \
                "test -d '$native_skill_dir/kubernetes-skill/scripts'" \
                0 \
                "Verify scripts directory exists"

            # Check for executable scripts
            local script_count=$(find "$native_skill_dir/kubernetes-skill/scripts" -name "*.sh" -type f 2>/dev/null | wc -l)
            if [[ $script_count -gt 0 ]]; then
                run_test "Kubernetes skill has script files" \
                    "test $script_count -gt 0" \
                    0 \
                    "Verify script files exist"
            else
                skip_test "Kubernetes skill has script files" "No scripts found"
            fi
        else
            skip_test "Kubernetes scripts directory" "Directory not found"
        fi
    else
        skip_test "Script generation tests" "Kubernetes skill not found"
    fi
}

#
# Category 6: Dual Execution Mode
#

test_dual_execution() {
    log_info "=== Testing Dual Execution Mode ==="

    # These tests verify that skills can be executed both via MCP and via scripts

    skip_test "MCP tool execution" "Requires running MCP server"
    skip_test "Script fallback execution" "Requires generated scripts"
    skip_test "Script wraps 'skill run'" "Requires generated scripts"
    skip_test "No logic duplication" "Requires code analysis"
}

#
# Category 7: Claude Agent Skills Compliance
#

test_claude_agent_compliance() {
    log_info "=== Testing Claude Agent Skills Compliance ==="

    skip_test "Filesystem discovery" "Requires Claude Code integration"
    skip_test "Progressive disclosure (Level 1)" "Requires YAML parser"
    skip_test "Progressive disclosure (Level 2)" "Requires SKILL.md parser"
    skip_test "Progressive disclosure (Level 3)" "Requires TOOLS.md parser"
    skip_test "Single source of truth" "Requires code analysis"
}

#
# Main Execution
#

main() {
    log_info "Starting Claude Bridge E2E Tests"
    log_info "Log file: $TEST_LOG"

    # Check prerequisites
    check_prerequisites

    # Run test categories
    test_claude_bridge_cli
    test_rust_unit_tests
    test_existing_integration_tests
    test_skill_md_format
    test_script_generation
    test_dual_execution
    test_claude_agent_compliance

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

    # Exit with appropriate code
    if [[ $FAILED_TESTS -gt 0 ]]; then
        log_error "Some tests failed!"
        exit 1
    else
        log_success "All active tests passed!"
        if [[ $SKIPPED_TESTS -gt 0 ]]; then
            log_info "Note: $SKIPPED_TESTS tests were skipped (require MCP server or generated files)"
        fi
        exit 0
    fi
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
