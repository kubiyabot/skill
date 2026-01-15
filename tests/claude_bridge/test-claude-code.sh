#!/bin/bash
# Automated End-to-End Test Suite for Claude Code Integration
# Tests complete workflow from skill generation to execution

set -e  # Exit on first error

# Configuration
SKILL_BIN="${SKILL_BIN:-skill}"
TEST_OUTPUT_DIR="/tmp/claude-code-test-$$"
PASSED=0
FAILED=0
TOTAL=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Cleanup on exit
cleanup() {
    echo ""
    echo "Cleaning up..."
    rm -rf "$TEST_OUTPUT_DIR"
    # Kill any running MCP servers
    pkill -f "skill serve" 2>/dev/null || true
}
trap cleanup EXIT

# Test helper function
run_test() {
    local test_name="$1"
    local test_func="$2"

    TOTAL=$((TOTAL + 1))
    echo ""
    echo -e "${CYAN}[$TOTAL] Testing: $test_name${NC}"

    if $test_func; then
        echo -e "${GREEN}  PASSED${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}  FAILED${NC}"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# ============================================================================
# Test Functions
# ============================================================================

test_prerequisites() {
    # Check skill binary exists
    if ! command -v "$SKILL_BIN" &> /dev/null; then
        echo "  ERROR: skill binary not found in PATH"
        return 1
    fi

    # Check skill version
    if ! "$SKILL_BIN" --version &> /dev/null; then
        echo "  ERROR: skill --version failed"
        return 1
    fi

    echo "  skill binary: $(which "$SKILL_BIN")"
    echo "  skill version: $("$SKILL_BIN" --version)"
    return 0
}

test_skill_generation() {
    mkdir -p "$TEST_OUTPUT_DIR"

    # Generate skills
    "$SKILL_BIN" claude generate --output "$TEST_OUTPUT_DIR" --force &> /dev/null

    # Verify output directory exists
    if [ ! -d "$TEST_OUTPUT_DIR" ]; then
        echo "  ERROR: Output directory not created"
        return 1
    fi

    # Count generated skills
    local skill_count=$(find "$TEST_OUTPUT_DIR" -mindepth 1 -maxdepth 1 -type d | wc -l)
    echo "  Generated $skill_count skills"

    if [ "$skill_count" -eq 0 ]; then
        echo "  ERROR: No skills generated"
        return 1
    fi

    return 0
}

test_skill_structure() {
    # Find first skill directory
    local skill_dir=$(find "$TEST_OUTPUT_DIR" -mindepth 1 -maxdepth 1 -type d | head -1)

    if [ -z "$skill_dir" ]; then
        echo "  ERROR: No skill directories found"
        return 1
    fi

    local skill_name=$(basename "$skill_dir")
    echo "  Validating skill: $skill_name"

    # Check for SKILL.md
    if [ ! -f "$skill_dir/SKILL.md" ]; then
        echo "  ERROR: SKILL.md not found"
        return 1
    fi

    # Check for TOOLS.md
    if [ ! -f "$skill_dir/TOOLS.md" ]; then
        echo "  ERROR: TOOLS.md not found"
        return 1
    fi

    # Check for scripts directory
    if [ ! -d "$skill_dir/scripts" ]; then
        echo "  ERROR: scripts/ directory not found"
        return 1
    fi

    echo "  ✓ SKILL.md exists"
    echo "  ✓ TOOLS.md exists"
    echo "  ✓ scripts/ directory exists"

    return 0
}

test_yaml_frontmatter() {
    local skill_dir=$(find "$TEST_OUTPUT_DIR" -mindepth 1 -maxdepth 1 -type d | head -1)
    local skill_md="$skill_dir/SKILL.md"

    # Check for YAML frontmatter
    if ! grep -q "^---$" "$skill_md"; then
        echo "  ERROR: No YAML frontmatter found"
        return 1
    fi

    # Extract YAML and validate with Python (if available)
    if command -v python3 &> /dev/null; then
        python3 -c "
import sys
import yaml

with open('$skill_md') as f:
    content = f.read()

# Extract YAML between --- delimiters
start = content.find('---\\n')
if start == -1:
    print('ERROR: No YAML frontmatter start')
    sys.exit(1)

end = content.find('\\n---', start + 4)
if end == -1:
    print('ERROR: No YAML frontmatter end')
    sys.exit(1)

yaml_content = content[start + 4:end]

try:
    data = yaml.safe_load(yaml_content)
    assert 'name' in data, 'name field required'
    assert 'description' in data, 'description field required'
    print(f'  ✓ Valid YAML with name: {data[\"name\"]}')
except Exception as e:
    print(f'ERROR: {e}')
    sys.exit(1)
" || return 1
    else
        echo "  ⚠ Python3 not available, skipping YAML validation"
    fi

    return 0
}

test_script_permissions() {
    local skill_dir=$(find "$TEST_OUTPUT_DIR" -mindepth 1 -maxdepth 1 -type d | head -1)
    local scripts_dir="$skill_dir/scripts"

    # Check that scripts are executable
    local script_count=0
    local executable_count=0

    for script in "$scripts_dir"/*.sh; do
        if [ -f "$script" ]; then
            script_count=$((script_count + 1))
            if [ -x "$script" ]; then
                executable_count=$((executable_count + 1))
            fi
        fi
    done

    echo "  Found $script_count scripts, $executable_count executable"

    if [ "$script_count" -eq 0 ]; then
        echo "  ERROR: No scripts found"
        return 1
    fi

    if [ "$executable_count" -ne "$script_count" ]; then
        echo "  ERROR: Not all scripts are executable"
        return 1
    fi

    return 0
}

test_mcp_server_health() {
    # Start MCP server in background
    "$SKILL_BIN" serve > /tmp/mcp-server-$$.log 2>&1 &
    local mcp_pid=$!

    # Wait for server to start
    sleep 2

    # Check if process is still running
    if ! kill -0 $mcp_pid 2>/dev/null; then
        echo "  ERROR: MCP server failed to start"
        cat /tmp/mcp-server-$$.log
        rm -f /tmp/mcp-server-$$.log
        return 1
    fi

    # Try to send a request
    local response=$(
        (
            echo '{"jsonrpc": "2.0", "method": "initialize", "id": 1, "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}'
            sleep 0.5
        ) | timeout 5s "$SKILL_BIN" serve 2>/dev/null
    )

    # Kill background server
    kill $mcp_pid 2>/dev/null || true
    rm -f /tmp/mcp-server-$$.log

    # Validate response
    if ! echo "$response" | grep -q "protocolVersion"; then
        echo "  ERROR: Invalid MCP response"
        echo "  Response: $response"
        return 1
    fi

    echo "  ✓ MCP server started successfully"
    echo "  ✓ MCP protocol handshake succeeded"

    return 0
}

test_mcp_tools_list() {
    local response=$(
        (
            echo '{"jsonrpc": "2.0", "method": "initialize", "id": 1, "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}'
            sleep 0.3
            echo '{"jsonrpc": "2.0", "method": "notifications/initialized"}'
            sleep 0.3
            echo '{"jsonrpc": "2.0", "method": "tools/list", "id": 2, "params": {}}'
            sleep 1
        ) | timeout 10s "$SKILL_BIN" serve 2>/dev/null
    )

    # Check for expected tools
    if ! echo "$response" | grep -q "execute"; then
        echo "  ERROR: 'execute' tool not found"
        return 1
    fi

    if ! echo "$response" | grep -q "list_skills"; then
        echo "  ERROR: 'list_skills' tool not found"
        return 1
    fi

    if ! echo "$response" | grep -q "search_skills"; then
        echo "  ERROR: 'search_skills' tool not found"
        return 1
    fi

    echo "  ✓ execute tool available"
    echo "  ✓ list_skills tool available"
    echo "  ✓ search_skills tool available"

    return 0
}

test_end_to_end_workflow() {
    echo "  Testing complete workflow: generate -> serve -> execute"

    # 1. Generate skills
    "$SKILL_BIN" claude generate --output "$TEST_OUTPUT_DIR-e2e" --force &> /dev/null

    # 2. Verify generation
    if [ ! -d "$TEST_OUTPUT_DIR-e2e" ]; then
        echo "  ERROR: Skill generation failed"
        return 1
    fi
    echo "  ✓ Skills generated"

    # 3. Test MCP server responds
    local response=$(
        (
            echo '{"jsonrpc": "2.0", "method": "initialize", "id": 1, "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}'
            sleep 0.3
            echo '{"jsonrpc": "2.0", "method": "notifications/initialized"}'
            sleep 0.3
            echo '{"jsonrpc": "2.0", "method": "tools/call", "id": 3, "params": {"name": "list_skills", "arguments": {}}}'
            sleep 2
        ) | timeout 15s "$SKILL_BIN" serve 2>/dev/null
    )

    if ! echo "$response" | grep -q '"id":3'; then
        echo "  ERROR: MCP execution failed"
        return 1
    fi
    echo "  ✓ MCP execution successful"

    # Cleanup
    rm -rf "$TEST_OUTPUT_DIR-e2e"

    return 0
}

# ============================================================================
# Main Test Execution
# ============================================================================

echo ""
echo "========================================"
echo "  Claude Code E2E Test Suite"
echo "========================================"
echo ""
echo "Test output directory: $TEST_OUTPUT_DIR"
echo ""

# Run all tests
run_test "Prerequisites check" test_prerequisites
run_test "Skill generation" test_skill_generation
run_test "Skill directory structure" test_skill_structure
run_test "YAML frontmatter validation" test_yaml_frontmatter
run_test "Script permissions" test_script_permissions
run_test "MCP server health" test_mcp_server_health
run_test "MCP tools list" test_mcp_tools_list
run_test "End-to-end workflow" test_end_to_end_workflow

# ============================================================================
# Print Summary
# ============================================================================
echo ""
echo "========================================"
echo "  Test Summary"
echo "========================================"
echo -e "  Total:  $TOTAL"
echo -e "  ${GREEN}Passed: $PASSED${NC}"
echo -e "  ${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed.${NC}"
    exit 1
fi
