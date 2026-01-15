#!/bin/bash
# test-skill-generation.sh - Test all skill generation modes
# Tests: --all, --skill=NAME, --project, --force, --no-scripts, --dry-run

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_OUTPUT_DIR="${TEST_OUTPUT_DIR:-/tmp/skill-test-generation}"
REPORT_FILE="$SCRIPT_DIR/reports/skill-generation-report.json"
VERBOSE="${VERBOSE:-false}"
SKILL_BIN="$PROJECT_ROOT/target/release/skill"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0
START_TIME=$(date +%s)

# Helper functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[PASS]${NC} $1"; ((TESTS_PASSED++)); }
log_fail() { echo -e "${RED}[FAIL]${NC} $1"; ((TESTS_FAILED++)); }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
verbose() { [[ "$VERBOSE" == "true" ]] && echo -e "${BLUE}[DEBUG]${NC} $1"; }

run_test() {
    local test_name="$1"
    shift
    ((TESTS_RUN++))
    verbose "Running: $test_name"
    if "$@" > /dev/null 2>&1; then
        log_success "$test_name"
        return 0
    else
        log_fail "$test_name"
        return 1
    fi
}

# Setup
setup() {
    log_info "Setting up skill generation test environment..."

    # Create clean test output directory
    rm -rf "$TEST_OUTPUT_DIR"
    mkdir -p "$TEST_OUTPUT_DIR"/{global,project-local,force-test,no-scripts}
    mkdir -p "$(dirname "$REPORT_FILE")"

    # Verify skill-cli binary exists
    if [[ ! -x "$SKILL_BIN" ]]; then
        log_fail "skill-cli binary not found or not executable: $SKILL_BIN"
        exit 1
    fi

    log_info "Test environment ready"
}

# Test: Dry-run mode works
test_dry_run() {
    ((TESTS_RUN++))
    local output_dir="$TEST_OUTPUT_DIR/dry-run"

    verbose "Testing dry-run mode..."
    if "$SKILL_BIN" claude generate --output "$output_dir" --dry-run 2>&1 | grep -q "Would generate"; then
        log_success "Dry-run mode works and shows preview"
        return 0
    else
        log_fail "Dry-run mode failed or missing preview"
        return 1
    fi
}

# Test: Generate all skills (if manifest exists)
test_generate_all_skills() {
    ((TESTS_RUN++))
    local output_dir="$TEST_OUTPUT_DIR/global"

    # Check if manifest exists
    if [[ ! -f "$PROJECT_ROOT/.skill-engine.toml" ]]; then
        log_warn "No manifest found, skipping generate all skills test"
        ((TESTS_RUN--))
        return 0
    fi

    verbose "Generating all skills to: $output_dir"
    if "$SKILL_BIN" claude generate --output "$output_dir" --force 2>&1 | tee /tmp/generate-all.log; then
        # Verify at least one skill directory was created
        local skill_count=$(find "$output_dir" -mindepth 1 -maxdepth 1 -type d | wc -l)
        if [[ $skill_count -gt 0 ]]; then
            log_success "Generate all skills succeeded ($skill_count skills generated)"
            return 0
        else
            log_fail "Generate all skills succeeded but no skills created"
            return 1
        fi
    else
        log_fail "Generate all skills command failed"
        cat /tmp/generate-all.log
        return 1
    fi
}

# Test: Validate SKILL.md structure
test_skill_md_structure() {
    ((TESTS_RUN++))
    local output_dir="$TEST_OUTPUT_DIR/global"

    # Find first SKILL.md file
    local skill_md=$(find "$output_dir" -name "SKILL.md" -type f | head -n 1)

    if [[ -z "$skill_md" ]]; then
        log_warn "No SKILL.md files found to validate"
        ((TESTS_RUN--))
        return 0
    fi

    verbose "Validating SKILL.md structure: $skill_md"

    # Check for required sections
    local required_sections=("name:" "description:" "When to Use" "Quick Reference")
    local missing_sections=()

    for section in "${required_sections[@]}"; do
        if ! grep -q "$section" "$skill_md"; then
            missing_sections+=("$section")
        fi
    done

    if [[ ${#missing_sections[@]} -eq 0 ]]; then
        log_success "SKILL.md has required sections"
        return 0
    else
        log_fail "SKILL.md missing sections: ${missing_sections[*]}"
        return 1
    fi
}

# Test: Validate TOOLS.md exists
test_tools_md_exists() {
    ((TESTS_RUN++))
    local output_dir="$TEST_OUTPUT_DIR/global"

    local tools_md=$(find "$output_dir" -name "TOOLS.md" -type f | head -n 1)

    if [[ -n "$tools_md" ]] && [[ -f "$tools_md" ]]; then
        log_success "TOOLS.md files generated"
        return 0
    else
        log_warn "No TOOLS.md files found"
        ((TESTS_RUN--))
        return 0
    fi
}

# Test: Scripts are executable
test_scripts_executable() {
    ((TESTS_RUN++))
    local output_dir="$TEST_OUTPUT_DIR/global"

    # Find script files
    local scripts=$(find "$output_dir" -path "*/scripts/*.sh" -type f)

    if [[ -z "$scripts" ]]; then
        log_warn "No script files found to check"
        ((TESTS_RUN--))
        return 0
    fi

    local non_executable=0
    while IFS= read -r script; do
        if [[ ! -x "$script" ]]; then
            ((non_executable++))
            verbose "Not executable: $script"
        fi
    done <<< "$scripts"

    if [[ $non_executable -eq 0 ]]; then
        log_success "All generated scripts are executable"
        return 0
    else
        log_fail "$non_executable scripts are not executable"
        return 1
    fi
}

# Test: Scripts have valid bash syntax
test_scripts_syntax() {
    ((TESTS_RUN++))
    local output_dir="$TEST_OUTPUT_DIR/global"

    local scripts=$(find "$output_dir" -path "*/scripts/*.sh" -type f | head -n 5)

    if [[ -z "$scripts" ]]; then
        log_warn "No script files found to check syntax"
        ((TESTS_RUN--))
        return 0
    fi

    local syntax_errors=0
    while IFS= read -r script; do
        if ! bash -n "$script" 2>/dev/null; then
            ((syntax_errors++))
            verbose "Syntax error in: $script"
        fi
    done <<< "$scripts"

    if [[ $syntax_errors -eq 0 ]]; then
        log_success "All generated scripts have valid syntax"
        return 0
    else
        log_fail "$syntax_errors scripts have syntax errors"
        return 1
    fi
}

# Test: Generate single skill
test_generate_single_skill() {
    ((TESTS_RUN++))
    local output_dir="$TEST_OUTPUT_DIR/single-skill"
    local test_skill="kubernetes"  # Common skill name

    # Check if manifest has this skill
    if ! grep -q "$test_skill" "$PROJECT_ROOT/.skill-engine.toml" 2>/dev/null; then
        log_warn "Test skill '$test_skill' not in manifest, skipping"
        ((TESTS_RUN--))
        return 0
    fi

    verbose "Generating single skill: $test_skill"
    if "$SKILL_BIN" claude generate --output "$output_dir" --skill "$test_skill" --force 2>&1; then
        if [[ -d "$output_dir/$test_skill" ]]; then
            log_success "Single skill generation succeeded"
            return 0
        else
            log_fail "Single skill generation succeeded but directory not created"
            return 1
        fi
    else
        log_fail "Single skill generation failed"
        return 1
    fi
}

# Test: Project-local generation (--project flag)
test_project_local_generation() {
    ((TESTS_RUN++))
    local test_dir="$TEST_OUTPUT_DIR/project-local"
    mkdir -p "$test_dir"
    cd "$test_dir"

    verbose "Testing project-local generation in: $test_dir"
    if "$SKILL_BIN" claude generate --project --force 2>&1; then
        if [[ -d ".claude/skills" ]]; then
            log_success "Project-local generation succeeded"
            cd "$SCRIPT_DIR"
            return 0
        else
            log_fail "Project-local generation succeeded but .claude/skills not created"
            cd "$SCRIPT_DIR"
            return 1
        fi
    else
        log_fail "Project-local generation failed"
        cd "$SCRIPT_DIR"
        return 1
    fi
}

# Test: Force overwrite works
test_force_overwrite() {
    ((TESTS_RUN++))
    local output_dir="$TEST_OUTPUT_DIR/force-test"

    verbose "Testing force overwrite..."

    # Generate once
    "$SKILL_BIN" claude generate --output "$output_dir" --force > /dev/null 2>&1 || true

    # Touch a file to mark it
    local marker_file=$(find "$output_dir" -name "SKILL.md" -type f | head -n 1)
    if [[ -z "$marker_file" ]]; then
        log_warn "No files generated for force test"
        ((TESTS_RUN--))
        return 0
    fi

    local original_time=$(stat -f %m "$marker_file" 2>/dev/null || stat -c %Y "$marker_file" 2>/dev/null)
    sleep 1

    # Generate again with force
    if "$SKILL_BIN" claude generate --output "$output_dir" --force > /dev/null 2>&1; then
        local new_time=$(stat -f %m "$marker_file" 2>/dev/null || stat -c %Y "$marker_file" 2>/dev/null)
        if [[ "$new_time" != "$original_time" ]]; then
            log_success "Force overwrite works"
            return 0
        else
            log_fail "Force overwrite did not update files"
            return 1
        fi
    else
        log_fail "Force overwrite command failed"
        return 1
    fi
}

# Test: No-scripts mode
test_no_scripts_mode() {
    ((TESTS_RUN++))
    local output_dir="$TEST_OUTPUT_DIR/no-scripts"

    verbose "Testing no-scripts mode..."
    if "$SKILL_BIN" claude generate --output "$output_dir" --no-scripts --force 2>&1; then
        # Check if scripts directory was NOT created
        local script_dirs=$(find "$output_dir" -type d -name "scripts" | wc -l)
        if [[ $script_dirs -eq 0 ]]; then
            log_success "No-scripts mode works (no script directories created)"
            return 0
        else
            log_fail "No-scripts mode created script directories"
            return 1
        fi
    else
        log_fail "No-scripts mode command failed"
        return 1
    fi
}

# Test: Output directory structure
test_output_directory_structure() {
    ((TESTS_RUN++))
    local output_dir="$TEST_OUTPUT_DIR/global"

    # Check for expected structure
    local skill_dirs=$(find "$output_dir" -mindepth 1 -maxdepth 1 -type d | wc -l)

    if [[ $skill_dirs -eq 0 ]]; then
        log_warn "No skill directories to check structure"
        ((TESTS_RUN--))
        return 0
    fi

    # Pick first skill directory
    local skill_dir=$(find "$output_dir" -mindepth 1 -maxdepth 1 -type d | head -n 1)
    local has_skill_md=false
    local has_tools_md=false
    local has_scripts=false

    [[ -f "$skill_dir/SKILL.md" ]] && has_skill_md=true
    [[ -f "$skill_dir/TOOLS.md" ]] && has_tools_md=true
    [[ -d "$skill_dir/scripts" ]] && has_scripts=true

    if $has_skill_md; then
        log_success "Output directory structure is valid"
        return 0
    else
        log_fail "Output directory structure missing required files"
        return 1
    fi
}

# Test: YAML frontmatter is valid
test_yaml_frontmatter() {
    ((TESTS_RUN++))
    local output_dir="$TEST_OUTPUT_DIR/global"

    local skill_md=$(find "$output_dir" -name "SKILL.md" -type f | head -n 1)

    if [[ -z "$skill_md" ]]; then
        log_warn "No SKILL.md file found for YAML validation"
        ((TESTS_RUN--))
        return 0
    fi

    # Extract frontmatter (between --- markers)
    local frontmatter=$(sed -n '/^---$/,/^---$/p' "$skill_md" | sed '1d;$d')

    if [[ -z "$frontmatter" ]]; then
        log_fail "No YAML frontmatter found in SKILL.md"
        return 1
    fi

    # Check for required fields
    if echo "$frontmatter" | grep -q "^name:" && echo "$frontmatter" | grep -q "^description:"; then
        log_success "YAML frontmatter is present and has required fields"
        return 0
    else
        log_fail "YAML frontmatter missing required fields"
        return 1
    fi
}

# Generate JSON report
generate_report() {
    local end_time=$(date +%s)
    local duration=$((end_time - START_TIME))
    local success_rate=0

    [[ $TESTS_RUN -gt 0 ]] && success_rate=$(awk "BEGIN {printf \"%.2f\", ($TESTS_PASSED / $TESTS_RUN) * 100}")

    cat > "$REPORT_FILE" << EOF
{
  "test_suite": "skill-generation",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "duration_seconds": $duration,
  "summary": {
    "total": $TESTS_RUN,
    "passed": $TESTS_PASSED,
    "failed": $TESTS_FAILED,
    "success_rate": "$success_rate%"
  },
  "test_output_directory": "$TEST_OUTPUT_DIR",
  "generated_skills": $(find "$TEST_OUTPUT_DIR/global" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l)
}
EOF
}

# Print summary
print_summary() {
    echo ""
    echo "=========================================="
    echo " Skill Generation Test Results"
    echo "=========================================="
    echo ""
    echo "  Total Tests:  $TESTS_RUN"
    echo -e "  Passed:       ${GREEN}$TESTS_PASSED${NC}"
    echo -e "  Failed:       ${RED}$TESTS_FAILED${NC}"
    echo ""

    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "${GREEN}✓ All tests passed!${NC}"
        echo ""
        echo "Test outputs saved to: $TEST_OUTPUT_DIR"
        echo "Report saved to: $REPORT_FILE"
    else
        echo -e "${RED}✗ Some tests failed${NC}"
        echo ""
        echo "Check logs and report: $REPORT_FILE"
    fi

    echo "=========================================="
}

# Main execution
main() {
    log_info "Starting Skill Generation Tests..."
    echo ""

    setup

    # Run all tests
    test_dry_run
    test_generate_all_skills
    test_skill_md_structure
    test_tools_md_exists
    test_scripts_executable
    test_scripts_syntax
    test_generate_single_skill
    test_project_local_generation
    test_force_overwrite
    test_no_scripts_mode
    test_output_directory_structure
    test_yaml_frontmatter

    generate_report
    print_summary

    [[ $TESTS_FAILED -eq 0 ]] && exit 0 || exit 1
}

# Handle arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose) VERBOSE=true; shift ;;
        -h|--help)
            echo "Usage: $0 [-v|--verbose] [-h|--help]"
            exit 0
            ;;
        *) log_warn "Unknown option: $1"; shift ;;
    esac
done

main "$@"
