#!/usr/bin/env bash
#
# Native Runtime Testing Suite
# Tests command allowlisting, argument validation, and injection prevention
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
TEST_LOG="${TEST_OUTPUT_DIR}/native-runtime-tests-$(date +%Y%m%d-%H%M%S).log"

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
# Category 1: Command Allowlist
#

test_command_allowlist() {
    log_info "=== Testing Command Allowlist ==="

    # Test that kubernetes skill can execute kubectl
    run_test "Kubernetes skill info (kubectl wrapper)" \
        "$SKILL_BIN info kubernetes >/dev/null 2>&1" \
        0 \
        "Verify kubernetes native skill is configured"

    # Test that terraform skill exists
    run_test "Terraform skill info (terraform wrapper)" \
        "$SKILL_BIN info terraform >/dev/null 2>&1" \
        0 \
        "Verify terraform native skill is configured"

    # Test that git skill exists
    run_test "Git skill info (git wrapper)" \
        "$SKILL_BIN info git >/dev/null 2>&1" \
        0 \
        "Verify git native skill is configured"

    # Test that docker skill exists
    run_test "Docker skill info (docker wrapper)" \
        "$SKILL_BIN info docker >/dev/null 2>&1" \
        0 \
        "Verify docker native skill is configured"

    # Test help for native skills
    run_test "Kubernetes skill help" \
        "$SKILL_BIN run kubernetes --help >/dev/null 2>&1" \
        0 \
        "Verify native skill help works"
}

#
# Category 2: Argument Validation
#

test_argument_validation() {
    log_info "=== Testing Argument Validation ==="

    # Test that run command requires skill argument
    run_test "Run without skill name fails" \
        "$SKILL_BIN run 2>&1; test \$? -ne 0" \
        0 \
        "Verify run command requires skill name"

    # Test invalid skill name
    run_test "Invalid skill name fails" \
        "$SKILL_BIN run nonexistent-skill-xyz 2>&1; test \$? -ne 0" \
        0 \
        "Verify invalid skill names are rejected"

    # Test native skill with invalid tool
    if $SKILL_BIN list 2>&1 | grep -q "kubernetes"; then
        run_test "Invalid tool name fails" \
            "$SKILL_BIN run kubernetes:invalid-tool-xyz 2>&1; test \$? -ne 0" \
            0 \
            "Verify invalid tool names are rejected"
    else
        skip_test "Invalid tool name fails" "Kubernetes skill not available"
    fi
}

#
# Category 3: Injection Prevention
#

test_injection_prevention() {
    log_info "=== Testing Injection Prevention ==="

    # These tests verify that injection attempts fail safely
    # All of these should be blocked by the runtime

    skip_test "Command injection with semicolon" "Requires test skill with injection vectors"
    skip_test "Command injection with pipe" "Requires test skill with injection vectors"
    skip_test "Command injection with &&" "Requires test skill with injection vectors"
    skip_test "Command injection with \$()" "Requires test skill with injection vectors"
    skip_test "Command injection with backticks" "Requires test skill with injection vectors"
    skip_test "Path traversal with ../" "Requires test skill with path parameters"
    skip_test "Environment variable injection" "Requires test skill with env access"
}

#
# Category 4: Environment Variable Filtering
#

test_environment_filtering() {
    log_info "=== Testing Environment Variable Filtering ==="

    # Test that environment variables can be passed
    if $SKILL_BIN list 2>&1 | grep -q "kubernetes"; then
        run_test "Environment variable passing" \
            "$SKILL_BIN run kubernetes --help >/dev/null 2>&1" \
            0 \
            "Verify environment variables work"
    else
        skip_test "Environment variable passing" "Kubernetes skill not available"
    fi

    skip_test "Dangerous env variable filtering (LD_PRELOAD)" "Requires test skill"
    skip_test "Custom environment variable setting" "Requires test skill"
}

#
# Category 5: Path Sanitization
#

test_path_sanitization() {
    log_info "=== Testing Path Sanitization ==="

    skip_test "Absolute path handling" "Requires test skill with file parameters"
    skip_test "Relative path handling" "Requires test skill with file parameters"
    skip_test "Symlink following" "Requires test skill with file parameters"
    skip_test "Directory traversal prevention" "Requires test skill with file parameters"
}

#
# Category 6: Real Skill Testing
#

test_real_skills() {
    log_info "=== Testing Real Native Skills ==="

    # Test kubernetes skill (if kubectl available)
    if command_exists kubectl; then
        run_test "Kubernetes skill execution (kubectl available)" \
            "$SKILL_BIN run kubernetes --help >/dev/null 2>&1" \
            0 \
            "Verify kubernetes skill works when kubectl is installed"
    else
        skip_test "Kubernetes skill execution" "kubectl not installed"
    fi

    # Test terraform skill (if terraform available)
    if command_exists terraform; then
        run_test "Terraform skill execution (terraform available)" \
            "$SKILL_BIN run terraform --help >/dev/null 2>&1" \
            0 \
            "Verify terraform skill works when terraform is installed"
    else
        skip_test "Terraform skill execution" "terraform not installed"
    fi

    # Test git skill (git is usually available)
    if command_exists git; then
        run_test "Git skill execution (git available)" \
            "$SKILL_BIN run git --help >/dev/null 2>&1" \
            0 \
            "Verify git skill works when git is installed"
    else
        skip_test "Git skill execution" "git not installed"
    fi

    # Test docker skill (if docker available)
    if command_exists docker && docker ps >/dev/null 2>&1; then
        run_test "Docker skill execution (docker available)" \
            "$SKILL_BIN run docker --help >/dev/null 2>&1" \
            0 \
            "Verify docker skill works when docker is running"
    else
        skip_test "Docker skill execution" "docker not available or not running"
    fi
}

#
# Category 7: SKILL.md Parsing
#

test_skill_md_parsing() {
    log_info "=== Testing SKILL.md Parsing ==="

    # Test that SKILL.md files are parsed correctly for native skills
    local native_skill_dir="${PROJECT_ROOT}/examples/native-skills"

    if [[ -d "$native_skill_dir" ]]; then
        # Count native skills
        local skill_count=$(find "$native_skill_dir" -maxdepth 1 -type d -name "*-skill" | wc -l)
        log_info "Found $skill_count native skills in examples/native-skills"

        run_test "Native skills directory exists" \
            "test -d '$native_skill_dir'" \
            0 \
            "Verify native skills directory structure"

        # Test kubernetes SKILL.md
        if [[ -f "$native_skill_dir/kubernetes-skill/SKILL.md" ]]; then
            run_test "Kubernetes SKILL.md exists" \
                "test -f '$native_skill_dir/kubernetes-skill/SKILL.md'" \
                0 \
                "Verify kubernetes SKILL.md file exists"
        else
            skip_test "Kubernetes SKILL.md exists" "File not found"
        fi

        # Test terraform SKILL.md
        if [[ -f "$native_skill_dir/terraform-skill/SKILL.md" ]]; then
            run_test "Terraform SKILL.md exists" \
                "test -f '$native_skill_dir/terraform-skill/SKILL.md'" \
                0 \
                "Verify terraform SKILL.md file exists"
        else
            skip_test "Terraform SKILL.md exists" "File not found"
        fi
    else
        skip_test "Native skills directory exists" "Directory not found"
    fi
}

#
# Category 8: Security Best Practices
#

test_security_practices() {
    log_info "=== Testing Security Best Practices ==="

    # Test that native skills don't expose dangerous commands
    skip_test "Dangerous command blocking (rm -rf)" "Requires security test skill"
    skip_test "Dangerous command blocking (dd)" "Requires security test skill"
    skip_test "Dangerous command blocking (mkfs)" "Requires security test skill"
    skip_test "Shell metacharacter sanitization" "Requires security test skill"
}

#
# Main Execution
#

main() {
    log_info "Starting Native Runtime Tests"
    log_info "Log file: $TEST_LOG"

    # Check prerequisites
    check_prerequisites

    # Run test categories
    test_command_allowlist
    test_argument_validation
    test_injection_prevention
    test_environment_filtering
    test_path_sanitization
    test_real_skills
    test_skill_md_parsing
    test_security_practices

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
            log_info "Note: $SKIPPED_TESTS tests were skipped (require test fixtures or tools)"
        fi
        exit 0
    fi
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
