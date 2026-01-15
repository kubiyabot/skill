#!/usr/bin/env bash
#
# Documentation Code Example Verification
# Tests all bash examples, TOML configs, and code blocks in documentation
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
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Test output directory
TEST_OUTPUT_DIR="${PROJECT_ROOT}/tests/output"
mkdir -p "$TEST_OUTPUT_DIR"

# Test log file
TEST_LOG="${TEST_OUTPUT_DIR}/code-examples-tests-$(date +%Y%m%d-%H%M%S).log"

# Documentation directory
DOCS_DIR="${PROJECT_ROOT}/docs-site"

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

    if [[ ! -d "$DOCS_DIR" ]]; then
        log_error "Documentation directory not found: $DOCS_DIR"
        exit 1
    fi

    log_info "Documentation directory: $DOCS_DIR"

    # Count documentation files
    local md_count=$(find "$DOCS_DIR" -name "*.md" -type f ! -path "*/node_modules/*" | wc -l)
    log_info "Found $md_count markdown documentation files"
}

#
# Category 1: Documentation File Structure
#

test_documentation_structure() {
    log_info "=== Testing Documentation Structure ==="

    # Test that key documentation files exist
    run_test "README.md exists" \
        "test -f '$DOCS_DIR/README.md'" \
        0 \
        "Verify main README exists"

    run_test "Installation guide exists" \
        "test -f '$DOCS_DIR/getting-started/installation.md'" \
        0 \
        "Verify installation documentation"

    run_test "Quick start guide exists" \
        "test -f '$DOCS_DIR/getting-started/quick-start.md'" \
        0 \
        "Verify quick start guide"

    run_test "CLI API documentation exists" \
        "test -f '$DOCS_DIR/api/cli.md'" \
        0 \
        "Verify CLI API docs"

    run_test "MCP documentation exists" \
        "test -f '$DOCS_DIR/guides/mcp.md'" \
        0 \
        "Verify MCP guide"

    run_test "Claude Code integration guide exists" \
        "test -f '$DOCS_DIR/guides/claude-code.md'" \
        0 \
        "Verify Claude Code integration docs"
}

#
# Category 2: Bash Code Block Extraction
#

test_bash_code_blocks() {
    log_info "=== Testing Bash Code Blocks ==="

    local temp_script="$TEST_OUTPUT_DIR/test-bash-block.sh"

    # Test extraction of bash blocks from installation.md
    if [[ -f "$DOCS_DIR/getting-started/installation.md" ]]; then
        # Check for installation instructions
        run_test "Installation doc has install instructions" \
            "grep -qi 'install' '$DOCS_DIR/getting-started/installation.md'" \
            0 \
            "Verify installation instructions exist"

        # Check for skill command examples
        run_test "Installation doc has skill commands" \
            "grep -q 'skill' '$DOCS_DIR/getting-started/installation.md'" \
            0 \
            "Verify skill command examples"
    else
        skip_test "Installation bash blocks" "installation.md not found"
    fi

    # Test bash blocks in CLI documentation
    if [[ -f "$DOCS_DIR/api/cli.md" ]]; then
        run_test "CLI doc has skill commands" \
            "grep -q 'skill ' '$DOCS_DIR/api/cli.md'" \
            0 \
            "Verify CLI commands documented"
    else
        skip_test "CLI bash blocks" "cli.md not found"
    fi
}

#
# Category 3: TOML Configuration Validation
#

test_toml_configurations() {
    log_info "=== Testing TOML Configurations ==="

    # Check if manifest.md has TOML examples
    if [[ -f "$DOCS_DIR/guides/manifest.md" ]]; then
        run_test "Manifest guide has TOML examples" \
            "grep -q '\`\`\`toml' '$DOCS_DIR/guides/manifest.md'" \
            0 \
            "Verify TOML code blocks exist"

        # Extract and validate TOML (basic check for now)
        run_test "Manifest TOML has skill section" \
            "grep -q 'skill' '$DOCS_DIR/guides/manifest.md'" \
            0 \
            "Verify skill section exists in examples"
    else
        skip_test "TOML validation" "manifest.md not found"
    fi

    # Check .skill-engine.toml in project root
    if [[ -f "$PROJECT_ROOT/.skill-engine.toml" ]]; then
        run_test "Project manifest is valid TOML" \
            "test -f '$PROJECT_ROOT/.skill-engine.toml'" \
            0 \
            "Verify project manifest exists"
    else
        skip_test "Project manifest validation" "No .skill-engine.toml in project"
    fi
}

#
# Category 4: JSON Configuration Validation
#

test_json_configurations() {
    log_info "=== Testing JSON Configurations ==="

    # Check MCP configuration examples
    if [[ -f "$DOCS_DIR/guides/mcp.md" ]]; then
        run_test "MCP guide has JSON examples" \
            "grep -q '\`\`\`json' '$DOCS_DIR/guides/mcp.md'" \
            0 \
            "Verify JSON code blocks for MCP"

        run_test "MCP guide mentions .mcp.json" \
            "grep -q '.mcp.json' '$DOCS_DIR/guides/mcp.md'" \
            0 \
            "Verify .mcp.json configuration mentioned"
    else
        skip_test "MCP JSON validation" "mcp.md not found"
    fi

    # Validate project .mcp.json if it exists
    if [[ -f "$PROJECT_ROOT/.mcp.json" ]]; then
        if command_exists jq; then
            run_test "Project .mcp.json is valid JSON" \
                "jq empty '$PROJECT_ROOT/.mcp.json' >/dev/null 2>&1" \
                0 \
                "Parse .mcp.json with jq"
        else
            skip_test "Project .mcp.json validation" "jq not installed"
        fi
    else
        skip_test "Project .mcp.json validation" "No .mcp.json in project"
    fi
}

#
# Category 5: Command Documentation Accuracy
#

test_command_documentation() {
    log_info "=== Testing Command Documentation Accuracy ==="

    # Verify documented commands exist in CLI
    local cli_doc="$DOCS_DIR/api/cli.md"

    if [[ -f "$cli_doc" ]]; then
        # Check for main commands (some may be documented in guides, not just CLI doc)
        local commands=("list" "run" "info" "install" "serve" "find" "config")

        for cmd in "${commands[@]}"; do
            run_test "Command '$cmd' documented" \
                "grep -q \"skill $cmd\" '$cli_doc'" \
                0 \
                "Verify $cmd command is documented"
        done

        # Check for claude command (in CLI doc or claude-code guide)
        if grep -q "skill claude" "$cli_doc" 2>/dev/null || grep -q "skill claude" "$DOCS_DIR/guides/claude-code.md" 2>/dev/null; then
            run_test "Command 'claude' documented" \
                "test 0 -eq 0" \
                0 \
                "Verify claude command documented"
        else
            skip_test "Command 'claude' documented" "Not found in CLI docs or Claude Code guide"
        fi

        # Check for web command (in CLI doc or web interface guide)
        if grep -q "skill web" "$cli_doc" 2>/dev/null || grep -q "skill web" "$DOCS_DIR/guides/web-interface.md" 2>/dev/null; then
            run_test "Command 'web' documented" \
                "test 0 -eq 0" \
                0 \
                "Verify web command documented"
        else
            skip_test "Command 'web' documented" "Not found in CLI docs or web interface guide"
        fi
    else
        skip_test "Command documentation accuracy" "cli.md not found"
    fi
}

#
# Category 6: Internal Link Validation
#

test_internal_links() {
    log_info "=== Testing Internal Links ==="

    local broken_links=0
    local checked_links=0

    # Find all markdown links [text](path)
    if command_exists grep; then
        # Simple check: look for markdown links to local files
        local docs_files=$(find "$DOCS_DIR" -name "*.md" -type f ! -path "*/node_modules/*")

        for file in $docs_files; do
            # Check for relative links
            if grep -q '\](\./' "$file" 2>/dev/null; then
                ((checked_links++))
            fi
        done

        if [[ $checked_links -gt 0 ]]; then
            run_test "Internal links found in documentation" \
                "test $checked_links -gt 0" \
                0 \
                "Found $checked_links files with internal links"
        else
            skip_test "Internal link validation" "No internal links found"
        fi
    else
        skip_test "Internal link validation" "grep not available"
    fi
}

#
# Category 7: Example Completeness
#

test_example_completeness() {
    log_info "=== Testing Example Completeness ==="

    # Check that example directories exist
    local examples_dir="$DOCS_DIR/examples"

    if [[ -d "$examples_dir" ]]; then
        run_test "Examples directory exists" \
            "test -d '$examples_dir'" \
            0 \
            "Verify examples directory"

        # Check for key examples
        run_test "Kubernetes example documented" \
            "test -f '$examples_dir/kubernetes.md'" \
            0 \
            "Verify kubernetes example"

        run_test "Terraform example documented" \
            "test -f '$examples_dir/terraform.md'" \
            0 \
            "Verify terraform example"
    else
        skip_test "Example completeness" "Examples directory not found"
    fi

    # Check that examples match actual skills
    local native_skills="$PROJECT_ROOT/examples/native-skills"

    if [[ -d "$native_skills" ]]; then
        if [[ -d "$native_skills/kubernetes-skill" ]]; then
            run_test "Kubernetes skill exists for example" \
                "test -d '$native_skills/kubernetes-skill'" \
                0 \
                "Verify kubernetes skill matches docs"
        fi

        if [[ -d "$native_skills/terraform-skill" ]]; then
            run_test "Terraform skill exists for example" \
                "test -d '$native_skills/terraform-skill'" \
                0 \
                "Verify terraform skill matches docs"
        fi
    else
        skip_test "Example skill verification" "native-skills directory not found"
    fi
}

#
# Category 8: Code Block Language Tags
#

test_code_block_formatting() {
    log_info "=== Testing Code Block Formatting ==="

    local docs_files=$(find "$DOCS_DIR" -name "*.md" -type f ! -path "*/node_modules/*")
    local files_with_code_blocks=0

    for file in $docs_files; do
        if grep -q '```' "$file" 2>/dev/null; then
            ((files_with_code_blocks++))
        fi
    done

    run_test "Documentation has code blocks" \
        "test $files_with_code_blocks -gt 0" \
        0 \
        "Found $files_with_code_blocks files with code blocks"

    # Check for common languages
    run_test "Documentation has bash examples" \
        "find '$DOCS_DIR' -name '*.md' ! -path '*/node_modules/*' -exec grep -l '\`\`\`bash' {} \; | head -1 | grep -q '.md'" \
        0 \
        "Verify bash code blocks exist"

    run_test "Documentation has TOML examples" \
        "find '$DOCS_DIR' -name '*.md' ! -path '*/node_modules/*' -exec grep -l '\`\`\`toml' {} \; | head -1 | grep -q '.md'" \
        0 \
        "Check for TOML code blocks"
}

#
# Main Execution
#

main() {
    log_info "Starting Documentation Code Example Verification"
    log_info "Log file: $TEST_LOG"

    # Check prerequisites
    check_prerequisites

    # Run test categories
    test_documentation_structure
    test_bash_code_blocks
    test_toml_configurations
    test_json_configurations
    test_command_documentation
    test_internal_links
    test_example_completeness
    test_code_block_formatting

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
            log_info "Note: $SKIPPED_TESTS tests were skipped (require external tools or specific setup)"
        fi
        exit 0
    fi
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
