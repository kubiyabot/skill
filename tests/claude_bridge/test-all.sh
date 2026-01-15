#!/bin/bash
# test-all.sh - Master test runner for Claude Bridge
# Runs all test suites in sequence and generates combined report

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERBOSE="${VERBOSE:-false}"

# Test suites
declare -a TEST_SUITES=(
    "test-fresh-install.sh:Fresh Installation"
    "test-skill-generation.sh:Skill Generation"
)

# Counters
SUITES_RUN=0
SUITES_PASSED=0
SUITES_FAILED=0
START_TIME=$(date +%s)

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[PASS]${NC} $1"; }
log_fail() { echo -e "${RED}[FAIL]${NC} $1"; }

# Run a test suite
run_suite() {
    local script="$1"
    local name="$2"

    ((SUITES_RUN++))

    echo ""
    echo "=========================================="
    echo " Running: $name"
    echo "=========================================="

    local flags=""
    [[ "$VERBOSE" == "true" ]] && flags="-v"

    if "$SCRIPT_DIR/$script" $flags; then
        log_success "$name suite passed"
        ((SUITES_PASSED++))
        return 0
    else
        log_fail "$name suite failed"
        ((SUITES_FAILED++))
        return 1
    fi
}

# Print final summary
print_summary() {
    local end_time=$(date +%s)
    local duration=$((end_time - START_TIME))

    echo ""
    echo "=========================================="
    echo " TEST RUN COMPLETE"
    echo "=========================================="
    echo ""
    echo "  Total Suites:  $SUITES_RUN"
    echo -e "  Passed:        ${GREEN}$SUITES_PASSED${NC}"
    echo -e "  Failed:        ${RED}$SUITES_FAILED${NC}"
    echo "  Duration:      ${duration}s"
    echo ""

    if [[ $SUITES_FAILED -eq 0 ]]; then
        echo -e "${GREEN}✓ ALL TEST SUITES PASSED!${NC}"
        echo ""
        echo "Next steps:"
        echo "  1. Review reports in: $SCRIPT_DIR/reports/"
        echo "  2. Mark subtask complete: task-master set-status --id=1.2 --status=done"
        echo "  3. Continue to subtask 1.3: task-master show 1.3"
    else
        echo -e "${RED}✗ SOME TEST SUITES FAILED${NC}"
        echo ""
        echo "Check individual test reports for details"
    fi

    echo "=========================================="
}

# Main
main() {
    log_info "Claude Bridge Test Runner"
    log_info "Running ${#TEST_SUITES[@]} test suites..."

    # Run each suite
    for suite in "${TEST_SUITES[@]}"; do
        IFS=':' read -r script name <<< "$suite"
        if [[ -f "$SCRIPT_DIR/$script" ]]; then
            run_suite "$script" "$name"
        else
            log_fail "Test suite not found: $script"
            ((SUITES_RUN++))
            ((SUITES_FAILED++))
        fi
    done

    print_summary

    [[ $SUITES_FAILED -eq 0 ]] && exit 0 || exit 1
}

# Handle arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose) VERBOSE=true; shift ;;
        -h|--help)
            echo "Usage: $0 [-v|--verbose] [-h|--help]"
            echo ""
            echo "Runs all Claude Bridge test suites"
            exit 0
            ;;
        *) shift ;;
    esac
done

main "$@"
