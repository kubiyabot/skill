#!/usr/bin/env bash
#
# Master Test Runner
# Runs all test suites for Skill Engine
#

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $*"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $*"
}

# Track results
SUITES_RUN=0
SUITES_PASSED=0
SUITES_FAILED=0

run_suite() {
    local suite_name="$1"
    local suite_script="$2"

    SUITES_RUN=$((SUITES_RUN + 1))

    log_info "Running test suite: $suite_name"
    log_info "Script: $suite_script"

    if [[ ! -x "$suite_script" ]]; then
        log_error "$suite_name - Script not executable"
        SUITES_FAILED=$((SUITES_FAILED + 1))
        return 1
    fi

    set +e
    "$suite_script"
    local exit_code=$?
    set -e

    if [[ $exit_code -eq 0 ]]; then
        log_success "$suite_name passed"
        SUITES_PASSED=$((SUITES_PASSED + 1))
    else
        log_error "$suite_name failed (exit code: $exit_code)"
        SUITES_FAILED=$((SUITES_FAILED + 1))
    fi

    echo ""
    return $exit_code
}

main() {
    log_info "=== Skill Engine Test Suite Runner ==="
    log_info "Project: $PROJECT_ROOT"
    echo ""

    # Run unit tests
    if [[ -f "$SCRIPT_DIR/unit/test-cli-commands.sh" ]]; then
        run_suite "CLI Commands (Unit)" "$SCRIPT_DIR/unit/test-cli-commands.sh"
    fi

    # Run integration tests
    if [[ -f "$SCRIPT_DIR/integration/test-wasm-runtime.sh" ]]; then
        run_suite "WASM Runtime (Integration)" "$SCRIPT_DIR/integration/test-wasm-runtime.sh"
    fi

    if [[ -f "$SCRIPT_DIR/integration/test-docker-runtime.sh" ]]; then
        run_suite "Docker Runtime (Integration)" "$SCRIPT_DIR/integration/test-docker-runtime.sh"
    fi

    if [[ -f "$SCRIPT_DIR/integration/test-native-runtime.sh" ]]; then
        run_suite "Native Runtime (Integration)" "$SCRIPT_DIR/integration/test-native-runtime.sh"
    fi

    # Run e2e tests
    if [[ -f "$SCRIPT_DIR/e2e/test-claude-bridge.sh" ]]; then
        run_suite "Claude Bridge (E2E)" "$SCRIPT_DIR/e2e/test-claude-bridge.sh"
    fi

    if [[ -f "$SCRIPT_DIR/e2e/test-mcp-integration.sh" ]]; then
        run_suite "MCP Integration (E2E)" "$SCRIPT_DIR/e2e/test-mcp-integration.sh"
    fi

    # Run documentation verification
    if [[ -f "$SCRIPT_DIR/verify-code-examples.sh" ]]; then
        run_suite "Documentation Verification" "$SCRIPT_DIR/verify-code-examples.sh"
    fi

    # Run security tests
    if [[ -f "$SCRIPT_DIR/security/test-injection-prevention.sh" ]]; then
        run_suite "Security - Injection Prevention" "$SCRIPT_DIR/security/test-injection-prevention.sh"
    fi

    if [[ -f "$SCRIPT_DIR/security/test-path-traversal.sh" ]]; then
        run_suite "Security - Path Traversal" "$SCRIPT_DIR/security/test-path-traversal.sh"
    fi

    if [[ -f "$SCRIPT_DIR/security/test-capabilities.sh" ]]; then
        run_suite "Security - Capabilities" "$SCRIPT_DIR/security/test-capabilities.sh"
    fi

    if [[ -f "$SCRIPT_DIR/security/test-credentials.sh" ]]; then
        run_suite "Security - Credentials" "$SCRIPT_DIR/security/test-credentials.sh"
    fi

    if [[ -f "$SCRIPT_DIR/security/test-resource-limits.sh" ]]; then
        run_suite "Security - Resource Limits" "$SCRIPT_DIR/security/test-resource-limits.sh"
    fi

    # Summary
    echo ""
    log_info "=== Test Suite Summary ==="
    log_info "Suites Run:    $SUITES_RUN"
    log_success "Passed:        $SUITES_PASSED"
    log_error "Failed:        $SUITES_FAILED"

    if [[ $SUITES_FAILED -gt 0 ]]; then
        log_error "Some test suites failed!"
        exit 1
    else
        log_success "All test suites passed!"
        exit 0
    fi
}

main "$@"
