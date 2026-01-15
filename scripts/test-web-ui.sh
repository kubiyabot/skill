#!/bin/bash
# Test runner script for Skill Engine Web Interface
# Runs all automated tests: backend integration, frontend unit, and E2E browser tests

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Track test results
BACKEND_TESTS_PASSED=0
FRONTEND_TESTS_PASSED=0
E2E_TESTS_PASSED=0

# Parse command line arguments
RUN_BACKEND=1
RUN_FRONTEND=1
RUN_E2E=0  # E2E tests optional by default
VERBOSE=0
BROWSER="chrome"

show_help() {
    cat << EOF
Usage: ./scripts/test-web-ui.sh [OPTIONS]

Run automated tests for Skill Engine Web Interface

OPTIONS:
    -b, --backend-only      Run only backend integration tests
    -f, --frontend-only     Run only frontend WASM tests
    -e, --e2e               Include E2E browser tests (Chrome required)
    -a, --all               Run all tests including E2E
    --browser BROWSER       Browser for WASM tests (chrome, firefox, safari)
    -v, --verbose           Verbose output
    -h, --help              Show this help message

EXAMPLES:
    ./scripts/test-web-ui.sh                    # Run backend + frontend tests
    ./scripts/test-web-ui.sh --all              # Run all tests including E2E
    ./scripts/test-web-ui.sh --backend-only     # Only backend tests
    ./scripts/test-web-ui.sh --e2e --browser firefox  # E2E tests with Firefox

EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -b|--backend-only)
            RUN_FRONTEND=0
            RUN_E2E=0
            shift
            ;;
        -f|--frontend-only)
            RUN_BACKEND=0
            RUN_E2E=0
            shift
            ;;
        -e|--e2e)
            RUN_E2E=1
            shift
            ;;
        -a|--all)
            RUN_BACKEND=1
            RUN_FRONTEND=1
            RUN_E2E=1
            shift
            ;;
        --browser)
            BROWSER="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=1
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Check if we're in the project root
if [ ! -f "Cargo.toml" ]; then
    print_error "This script must be run from the project root directory"
    exit 1
fi

# Check for required tools
print_header "Checking Prerequisites"

# Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
    print_error "cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi
print_success "Rust/Cargo installed"

# Check for wasm-pack if running frontend tests
if [ $RUN_FRONTEND -eq 1 ] || [ $RUN_E2E -eq 1 ]; then
    if ! command -v wasm-pack &> /dev/null; then
        print_warning "wasm-pack not found. Installing..."
        curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
        if [ $? -eq 0 ]; then
            print_success "wasm-pack installed successfully"
        else
            print_error "Failed to install wasm-pack"
            exit 1
        fi
    else
        print_success "wasm-pack installed"
    fi
fi

# Check for browser if running E2E tests
if [ $RUN_E2E -eq 1 ]; then
    case $BROWSER in
        chrome|chromium)
            if ! command -v google-chrome &> /dev/null && ! command -v chromium &> /dev/null; then
                print_warning "Chrome/Chromium not found. E2E tests may fail."
            else
                print_success "Chrome/Chromium available"
            fi
            ;;
        firefox)
            if ! command -v firefox &> /dev/null; then
                print_warning "Firefox not found. E2E tests may fail."
            else
                print_success "Firefox available"
            fi
            ;;
        safari)
            if [[ "$OSTYPE" != "darwin"* ]]; then
                print_error "Safari tests only available on macOS"
                exit 1
            fi
            print_success "Safari available (macOS)"
            ;;
    esac
fi

echo ""

# ============================================================================
# BACKEND INTEGRATION TESTS
# ============================================================================

if [ $RUN_BACKEND -eq 1 ]; then
    print_header "Running Backend Integration Tests"
    echo "Package: skill-http"
    echo "Location: crates/skill-http/tests/"
    echo ""

    if [ $VERBOSE -eq 1 ]; then
        cargo test -p skill-http --tests -- --nocapture
    else
        cargo test -p skill-http --tests
    fi

    if [ $? -eq 0 ]; then
        BACKEND_TESTS_PASSED=1
        print_success "Backend integration tests passed"
    else
        print_error "Backend integration tests failed"
        BACKEND_TESTS_PASSED=0
    fi
    echo ""
fi

# ============================================================================
# FRONTEND WASM TESTS
# ============================================================================

if [ $RUN_FRONTEND -eq 1 ]; then
    print_header "Running Frontend WASM Tests"
    echo "Package: skill-web"
    echo "Location: crates/skill-web/tests/"
    echo "Browser: $BROWSER"
    echo ""

    cd crates/skill-web

    case $BROWSER in
        chrome|chromium)
            if [ $VERBOSE -eq 1 ]; then
                wasm-pack test --headless --chrome -- --nocapture
            else
                wasm-pack test --headless --chrome
            fi
            ;;
        firefox)
            if [ $VERBOSE -eq 1 ]; then
                wasm-pack test --headless --firefox -- --nocapture
            else
                wasm-pack test --headless --firefox
            fi
            ;;
        safari)
            print_warning "Safari tests require manual interaction"
            if [ $VERBOSE -eq 1 ]; then
                wasm-pack test --safari -- --nocapture
            else
                wasm-pack test --safari
            fi
            ;;
    esac

    if [ $? -eq 0 ]; then
        FRONTEND_TESTS_PASSED=1
        print_success "Frontend WASM tests passed"
    else
        print_error "Frontend WASM tests failed"
        FRONTEND_TESTS_PASSED=0
    fi

    cd ../..
    echo ""
fi

# ============================================================================
# E2E BROWSER TESTS
# ============================================================================

if [ $RUN_E2E -eq 1 ]; then
    print_header "Running E2E Browser Tests"
    echo "Package: skill-web"
    echo "Tests: e2e_*.rs"
    echo "Browser: $BROWSER"
    echo ""

    cd crates/skill-web

    case $BROWSER in
        chrome|chromium)
            if [ $VERBOSE -eq 1 ]; then
                wasm-pack test --headless --chrome -- --nocapture --test 'e2e_*'
            else
                wasm-pack test --headless --chrome -- --test 'e2e_*'
            fi
            ;;
        firefox)
            if [ $VERBOSE -eq 1 ]; then
                wasm-pack test --headless --firefox -- --nocapture --test 'e2e_*'
            else
                wasm-pack test --headless --firefox -- --test 'e2e_*'
            fi
            ;;
        safari)
            print_warning "Safari E2E tests require manual interaction"
            if [ $VERBOSE -eq 1 ]; then
                wasm-pack test --safari -- --nocapture --test 'e2e_*'
            else
                wasm-pack test --safari -- --test 'e2e_*'
            fi
            ;;
    esac

    if [ $? -eq 0 ]; then
        E2E_TESTS_PASSED=1
        print_success "E2E browser tests passed"
    else
        print_error "E2E browser tests failed"
        E2E_TESTS_PASSED=0
    fi

    cd ../..
    echo ""
fi

# ============================================================================
# SUMMARY
# ============================================================================

print_header "Test Results Summary"

if [ $RUN_BACKEND -eq 1 ]; then
    if [ $BACKEND_TESTS_PASSED -eq 1 ]; then
        print_success "Backend Integration Tests: PASSED"
    else
        print_error "Backend Integration Tests: FAILED"
    fi
fi

if [ $RUN_FRONTEND -eq 1 ]; then
    if [ $FRONTEND_TESTS_PASSED -eq 1 ]; then
        print_success "Frontend WASM Tests: PASSED"
    else
        print_error "Frontend WASM Tests: FAILED"
    fi
fi

if [ $RUN_E2E -eq 1 ]; then
    if [ $E2E_TESTS_PASSED -eq 1 ]; then
        print_success "E2E Browser Tests: PASSED"
    else
        print_error "E2E Browser Tests: FAILED"
    fi
fi

echo ""

# Calculate overall result
TOTAL_SUITES=0
PASSED_SUITES=0

if [ $RUN_BACKEND -eq 1 ]; then
    TOTAL_SUITES=$((TOTAL_SUITES + 1))
    PASSED_SUITES=$((PASSED_SUITES + BACKEND_TESTS_PASSED))
fi

if [ $RUN_FRONTEND -eq 1 ]; then
    TOTAL_SUITES=$((TOTAL_SUITES + 1))
    PASSED_SUITES=$((PASSED_SUITES + FRONTEND_TESTS_PASSED))
fi

if [ $RUN_E2E -eq 1 ]; then
    TOTAL_SUITES=$((TOTAL_SUITES + 1))
    PASSED_SUITES=$((PASSED_SUITES + E2E_TESTS_PASSED))
fi

if [ $PASSED_SUITES -eq $TOTAL_SUITES ]; then
    print_success "All test suites passed! ($PASSED_SUITES/$TOTAL_SUITES)"
    echo ""
    exit 0
else
    print_error "Some test suites failed ($PASSED_SUITES/$TOTAL_SUITES passed)"
    echo ""
    exit 1
fi
