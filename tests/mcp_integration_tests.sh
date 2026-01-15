#!/bin/bash
# Comprehensive MCP Server Integration Tests
# Run with: ./tests/mcp_integration_tests.sh

set -e

SKILL_BIN="/Users/shaked/projects/skill-engine/target/release/skill"
PASSED=0
FAILED=0
TOTAL=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test helper function
run_mcp_test() {
    local test_name="$1"
    local request="$2"
    local expected_pattern="$3"
    local timeout_sec="${4:-15}"
    local wait_time="${5:-3}"

    TOTAL=$((TOTAL + 1))
    echo -e "${CYAN}[$TOTAL] Testing: $test_name${NC}"

    # Run test with proper timing
    local result=$(
        (
            echo '{"jsonrpc": "2.0", "method": "initialize", "id": 1, "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}'
            sleep 0.3
            echo '{"jsonrpc": "2.0", "method": "notifications/initialized"}'
            sleep 0.3
            echo "$request"
            sleep $wait_time
        ) | timeout $timeout_sec "$SKILL_BIN" serve 2>/dev/null
    )

    # Check result
    if echo "$result" | grep -qE "$expected_pattern"; then
        echo -e "${GREEN}  PASSED${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}  FAILED${NC}"
        echo -e "${YELLOW}  Expected pattern: $expected_pattern${NC}"
        echo -e "${YELLOW}  Got: $(echo "$result" | grep -v "^$" | tail -1 | head -c 300)...${NC}"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

echo ""
echo "========================================"
echo "  MCP Server Integration Tests"
echo "========================================"
echo ""

# ============================================
# SECTION 1: Basic MCP Protocol Tests
# ============================================
echo -e "${YELLOW}=== Section 1: Basic MCP Protocol ===${NC}"

# Test 1.1: Initialize handshake
run_mcp_test "Initialize handshake" \
    '{"jsonrpc": "2.0", "method": "tools/list", "id": 2, "params": {}}' \
    '"protocolVersion":"2024-11-05"' 10 2

# Test 1.2: Tools list returns execute tool
run_mcp_test "Tools list has execute" \
    '{"jsonrpc": "2.0", "method": "tools/list", "id": 2, "params": {}}' \
    '"name":"execute"' 10 2

# Test 1.3: Tools list returns list_skills tool
run_mcp_test "Tools list has list_skills" \
    '{"jsonrpc": "2.0", "method": "tools/list", "id": 2, "params": {}}' \
    '"name":"list_skills"' 10 2

# Test 1.4: Tools list returns search_skills tool
run_mcp_test "Tools list has search_skills" \
    '{"jsonrpc": "2.0", "method": "tools/list", "id": 2, "params": {}}' \
    '"name":"search_skills"' 10 2

# ============================================
# SECTION 2: list_skills Tool Tests
# ============================================
echo ""
echo -e "${YELLOW}=== Section 2: list_skills Tool ===${NC}"

# Test 2.1: List all skills
run_mcp_test "list_skills returns kubernetes" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 10, "params": {"name": "list_skills", "arguments": {}}}' \
    '"id":10.*kubernetes' 10 2

# Test 2.2: List skills shows get tool
run_mcp_test "list_skills shows get tool" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 11, "params": {"name": "list_skills", "arguments": {}}}' \
    'get.*Get Kubernetes resources' 10 2

# Test 2.3: List skills shows describe tool
run_mcp_test "list_skills shows describe tool" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 12, "params": {"name": "list_skills", "arguments": {}}}' \
    'describe.*Show detailed' 10 2

# Test 2.4: List skills shows logs tool
run_mcp_test "list_skills shows logs tool" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 13, "params": {"name": "list_skills", "arguments": {}}}' \
    'logs.*Get logs' 10 2

# Test 2.5: List skills with kubernetes filter
run_mcp_test "list_skills filtered by kubernetes" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 14, "params": {"name": "list_skills", "arguments": {"skill": "kubernetes"}}}' \
    '"id":14.*kubernetes.*get' 10 2

# ============================================
# SECTION 3: execute Tool Tests - Kubernetes
# ============================================
echo ""
echo -e "${YELLOW}=== Section 3: execute Tool - Kubernetes ===${NC}"

# Test 3.1: kubectl get namespaces
run_mcp_test "kubectl get namespaces" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 20, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "namespaces"}}}}' \
    '"id":20.*default.*kube-system' 15 4

# Test 3.2: kubectl get pods -A
run_mcp_test "kubectl get pods all namespaces" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 21, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "pods", "all-namespaces": "true"}}}}' \
    '"id":21.*"isError":false' 15 4

# Test 3.3: kubectl cluster-info
run_mcp_test "kubectl cluster-info" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 22, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "cluster-info"}}}' \
    '"id":22.*control plane' 15 4

# Test 3.4: kubectl config current-context
run_mcp_test "kubectl config current-context" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 23, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "config", "args": {"action": "current-context"}}}}' \
    '"id":23.*"isError":false' 15 4

# Test 3.5: kubectl get namespaces -o json
run_mcp_test "kubectl get namespaces JSON output" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 24, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "namespaces", "output": "json"}}}}' \
    '"id":24.*apiVersion.*items' 15 4

# Test 3.6: kubectl describe namespace default
run_mcp_test "kubectl describe namespace" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 25, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "describe", "args": {"resource": "namespace", "name": "default"}}}}' \
    '"id":25.*"isError":false' 15 4

# Test 3.7: kubectl get nodes
run_mcp_test "kubectl get nodes" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 26, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "nodes"}}}}' \
    '"id":26.*"isError":false' 15 4

# Test 3.8: kubectl config get-contexts
run_mcp_test "kubectl config get-contexts" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 27, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "config", "args": {"action": "get-contexts"}}}}' \
    '"id":27.*"isError":false' 15 4

# Test 3.9: kubectl get services
run_mcp_test "kubectl get services" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 28, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "services"}}}}' \
    '"id":28.*"isError":false' 15 4

# Test 3.10: kubectl get deployments
run_mcp_test "kubectl get deployments" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 29, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "deployments", "all-namespaces": "true"}}}}' \
    '"id":29.*"isError":false' 15 4

# ============================================
# SECTION 4: search_skills Tool Tests
# ============================================
echo ""
echo -e "${YELLOW}=== Section 4: search_skills Tool ===${NC}"

# Test 4.1: Search for pods
run_mcp_test "search: list pods" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 30, "params": {"name": "search_skills", "arguments": {"query": "list running pods"}}}' \
    '"id":30.*Search Results' 20 8

# Test 4.2: Search for scaling
run_mcp_test "search: scale deployment" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 31, "params": {"name": "search_skills", "arguments": {"query": "scale deployment replicas"}}}' \
    '"id":31.*Search Results' 20 8

# Test 4.3: Search with top_k limit
run_mcp_test "search: top_k=3" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 32, "params": {"name": "search_skills", "arguments": {"query": "kubernetes", "top_k": 3}}}' \
    '"id":32.*Search Results' 20 8

# Test 4.4: Search for logs
run_mcp_test "search: container logs" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 33, "params": {"name": "search_skills", "arguments": {"query": "view container logs"}}}' \
    '"id":33.*Search Results' 20 8

# Test 4.5: Search for node management
run_mcp_test "search: drain node" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 34, "params": {"name": "search_skills", "arguments": {"query": "drain node maintenance"}}}' \
    '"id":34.*Search Results' 20 8

# ============================================
# SECTION 5: Error Handling Tests
# ============================================
echo ""
echo -e "${YELLOW}=== Section 5: Error Handling ===${NC}"

# Test 5.1: Missing required parameter (skill)
run_mcp_test "Error: missing skill parameter" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 40, "params": {"name": "execute", "arguments": {"tool": "get"}}}' \
    '"id":40.*error' 10 2

# Test 5.2: Missing required parameter (tool)
run_mcp_test "Error: missing tool parameter" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 41, "params": {"name": "execute", "arguments": {"skill": "kubernetes"}}}' \
    '"id":41.*error' 10 2

# Test 5.3: Non-existent skill
run_mcp_test "Error: non-existent skill" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 42, "params": {"name": "execute", "arguments": {"skill": "nonexistent_skill_xyz", "tool": "get"}}}' \
    '"id":42.*error' 10 2

# Test 5.4: Missing search query
run_mcp_test "Error: search without query" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 45, "params": {"name": "search_skills", "arguments": {}}}' \
    '"id":45.*error' 10 2

# Test 5.5: Invalid tool name
run_mcp_test "Error: invalid MCP tool" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 46, "params": {"name": "invalid_tool_xyz", "arguments": {}}}' \
    '"id":46.*error' 10 2

# ============================================
# SECTION 6: Edge Cases
# ============================================
echo ""
echo -e "${YELLOW}=== Section 6: Edge Cases ===${NC}"

# Test 6.1: Empty arguments object
run_mcp_test "Edge: list_skills empty args" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 50, "params": {"name": "list_skills", "arguments": {}}}' \
    '"id":50.*kubernetes' 10 2

# Test 6.2: Instance parameter (default)
run_mcp_test "Edge: explicit default instance" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 51, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "cluster-info", "instance": "default"}}}' \
    '"id":51.*"isError":false' 15 4

# Test 6.3: Args with special characters (label selector)
run_mcp_test "Edge: label selector" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 52, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "pods", "selector": "app=test", "all-namespaces": "true"}}}}' \
    '"id":52.*"isError"' 15 4

# Test 6.4: Very long search query
run_mcp_test "Edge: long search query" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 53, "params": {"name": "search_skills", "arguments": {"query": "I need to find a tool that can help me list all the running pods in my kubernetes cluster"}}}' \
    '"id":53.*Search Results' 20 8

# Test 6.5: Numeric top_k
run_mcp_test "Edge: numeric top_k=2" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 55, "params": {"name": "search_skills", "arguments": {"query": "scale", "top_k": 2}}}' \
    '"id":55.*Search Results' 20 8

# Test 6.6: Get with namespace filter
run_mcp_test "Edge: get pods in kube-system" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 56, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "pods", "namespace": "kube-system"}}}}' \
    '"id":56.*"isError":false' 15 4

# Test 6.7: Get with wide output
run_mcp_test "Edge: get pods wide output" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 57, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "pods", "all-namespaces": "true", "output": "wide"}}}}' \
    '"id":57.*"isError":false' 15 4

# ============================================
# SECTION 7: Concurrent Requests
# ============================================
echo ""
echo -e "${YELLOW}=== Section 7: Concurrent Requests ===${NC}"

# Test 7.1: Multiple requests in sequence (simulating concurrent usage)
run_concurrent_test() {
    local test_name="$1"
    local timeout_sec="${2:-20}"

    TOTAL=$((TOTAL + 1))
    echo -e "${CYAN}[$TOTAL] Testing: $test_name${NC}"

    # Send multiple requests in rapid succession
    local result=$(
        (
            echo '{"jsonrpc": "2.0", "method": "initialize", "id": 1, "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}'
            sleep 0.2
            echo '{"jsonrpc": "2.0", "method": "notifications/initialized"}'
            sleep 0.2
            # Rapid fire 3 different requests
            echo '{"jsonrpc": "2.0", "method": "tools/call", "id": 60, "params": {"name": "list_skills", "arguments": {}}}'
            echo '{"jsonrpc": "2.0", "method": "tools/call", "id": 61, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "namespaces"}}}}'
            echo '{"jsonrpc": "2.0", "method": "tools/call", "id": 62, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "cluster-info"}}}'
            sleep 5
        ) | timeout $timeout_sec "$SKILL_BIN" serve 2>/dev/null
    )

    # Check all three responses came back with correct IDs
    local found_60=$(echo "$result" | grep -c '"id":60')
    local found_61=$(echo "$result" | grep -c '"id":61')
    local found_62=$(echo "$result" | grep -c '"id":62')

    if [ "$found_60" -ge 1 ] && [ "$found_61" -ge 1 ] && [ "$found_62" -ge 1 ]; then
        echo -e "${GREEN}  PASSED (all 3 responses received)${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}  FAILED (missing responses: 60=$found_60, 61=$found_61, 62=$found_62)${NC}"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

run_concurrent_test "Multiple rapid requests" 25

# Test 7.2: Mixed tool types concurrently
run_mixed_concurrent_test() {
    local test_name="$1"
    local timeout_sec="${2:-25}"

    TOTAL=$((TOTAL + 1))
    echo -e "${CYAN}[$TOTAL] Testing: $test_name${NC}"

    local result=$(
        (
            echo '{"jsonrpc": "2.0", "method": "initialize", "id": 1, "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}'
            sleep 0.2
            echo '{"jsonrpc": "2.0", "method": "notifications/initialized"}'
            sleep 0.2
            # Mixed requests: list, execute, search
            echo '{"jsonrpc": "2.0", "method": "tools/list", "id": 70, "params": {}}'
            echo '{"jsonrpc": "2.0", "method": "tools/call", "id": 71, "params": {"name": "list_skills", "arguments": {}}}'
            echo '{"jsonrpc": "2.0", "method": "tools/call", "id": 72, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "nodes"}}}}'
            echo '{"jsonrpc": "2.0", "method": "tools/call", "id": 73, "params": {"name": "search_skills", "arguments": {"query": "kubernetes pods", "top_k": 2}}}'
            sleep 10
        ) | timeout $timeout_sec "$SKILL_BIN" serve 2>/dev/null
    )

    # Check all responses
    local found_70=$(echo "$result" | grep -c '"id":70')
    local found_71=$(echo "$result" | grep -c '"id":71')
    local found_72=$(echo "$result" | grep -c '"id":72')
    local found_73=$(echo "$result" | grep -c '"id":73')

    if [ "$found_70" -ge 1 ] && [ "$found_71" -ge 1 ] && [ "$found_72" -ge 1 ] && [ "$found_73" -ge 1 ]; then
        echo -e "${GREEN}  PASSED (all 4 responses received)${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}  FAILED (missing responses: 70=$found_70, 71=$found_71, 72=$found_72, 73=$found_73)${NC}"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

run_mixed_concurrent_test "Mixed tool types" 30

# ============================================
# SECTION 8: Pagination Tests
# ============================================
echo ""
echo -e "${YELLOW}=== Section 8: Pagination ===${NC}"

# Test 8.1: list_skills with limit
run_mcp_test "Pagination: limit=5" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 80, "params": {"name": "list_skills", "arguments": {"limit": 5}}}' \
    '"id":80.*Showing 5 of' 10 2

# Test 8.2: list_skills with offset and limit
run_mcp_test "Pagination: offset=5, limit=3" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 81, "params": {"name": "list_skills", "arguments": {"offset": 5, "limit": 3}}}' \
    '"id":81.*offset: 5.*limit: 3' 10 2

# Test 8.3: Verify "has more" indicator
run_mcp_test "Pagination: has_more indicator" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 82, "params": {"name": "list_skills", "arguments": {"limit": 2}}}' \
    '"id":82.*Next page.*offset=' 10 2

# Test 8.4: Pagination with skill filter
run_mcp_test "Pagination: filter + limit" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 83, "params": {"name": "list_skills", "arguments": {"skill": "kubernetes", "limit": 5}}}' \
    '"id":83.*Showing 5 of.*kubernetes' 10 2

# Test 8.5: Large offset (beyond results)
run_mcp_test "Pagination: large offset" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 84, "params": {"name": "list_skills", "arguments": {"offset": 1000, "limit": 10}}}' \
    '"id":84.*Showing 0 of' 10 2

# Test 8.6: Offset=0 explicit
run_mcp_test "Pagination: offset=0 explicit" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 85, "params": {"name": "list_skills", "arguments": {"offset": 0, "limit": 10}}}' \
    '"id":85.*Showing.*of' 10 2

# Test 8.7: Consecutive pages test
run_pagination_sequence_test() {
    local test_name="$1"
    local timeout_sec="${2:-20}"

    TOTAL=$((TOTAL + 1))
    echo -e "${CYAN}[$TOTAL] Testing: $test_name${NC}"

    # Get first page (limit=5)
    local page1=$(
        (
            echo '{"jsonrpc": "2.0", "method": "initialize", "id": 1, "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}'
            sleep 0.2
            echo '{"jsonrpc": "2.0", "method": "notifications/initialized"}'
            sleep 0.2
            echo '{"jsonrpc": "2.0", "method": "tools/call", "id": 90, "params": {"name": "list_skills", "arguments": {"offset": 0, "limit": 5}}}'
            sleep 2
        ) | timeout $timeout_sec "$SKILL_BIN" serve 2>/dev/null
    )

    # Get second page (offset=5, limit=5)
    local page2=$(
        (
            echo '{"jsonrpc": "2.0", "method": "initialize", "id": 1, "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}'
            sleep 0.2
            echo '{"jsonrpc": "2.0", "method": "notifications/initialized"}'
            sleep 0.2
            echo '{"jsonrpc": "2.0", "method": "tools/call", "id": 91, "params": {"name": "list_skills", "arguments": {"offset": 5, "limit": 5}}}'
            sleep 2
        ) | timeout $timeout_sec "$SKILL_BIN" serve 2>/dev/null
    )

    # Check both pages returned data
    local has_page1=$(echo "$page1" | grep -c '"id":90')
    local has_page2=$(echo "$page2" | grep -c '"id":91')

    if [ "$has_page1" -ge 1 ] && [ "$has_page2" -ge 1 ]; then
        echo -e "${GREEN}  PASSED (both pages returned)${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}  FAILED (page1=$has_page1, page2=$has_page2)${NC}"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

run_pagination_sequence_test "Consecutive pages" 25

# ============================================
# SECTION 9: Claude Bridge Context Engineering
# ============================================
echo ""
echo -e "${YELLOW}=== Section 9: Claude Bridge Context Engineering ===${NC}"

# Test 9.1: Execute tool via MCP (baseline)
run_mcp_test "Execute kubernetes:get via MCP" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 100, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "namespaces"}}}}' \
    '"id":100.*"isError":false' 15 4

# Test 9.2: Context engineering - grep filter
run_mcp_test "Context engineering: grep filter" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 101, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "namespaces"}, "grep": "default"}}}' \
    '"id":101.*default' 15 4

# Test 9.3: Context engineering - head limit
run_mcp_test "Context engineering: head limit" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 102, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "namespaces"}, "head": 3}}}' \
    '"id":102.*"isError":false' 15 4

# Test 9.4: Context engineering - jq extraction
run_mcp_test "Context engineering: jq extraction" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 103, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "namespaces", "output": "json"}, "jq": ".items[].metadata.name"}}}' \
    '"id":103.*"isError":false' 20 5

# Test 9.5: Context engineering - max_output truncation
run_mcp_test "Context engineering: max_output" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 104, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get", "args": {"resource": "pods", "all-namespaces": "true"}, "max_output": 500}}}' \
    '"id":104.*"isError":false' 15 4

# Test 9.6: Error handling - invalid skill name
run_mcp_test "Error: invalid skill name" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 105, "params": {"name": "execute", "arguments": {"skill": "nonexistent_skill_xyz", "tool": "get"}}}' \
    '"id":105.*error' 10 2

# Test 9.7: Error handling - invalid tool name
run_mcp_test "Error: invalid tool name" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 106, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "nonexistent_tool_xyz"}}}' \
    '"id":106.*error' 10 2

# Test 9.8: Error handling - missing required parameter
run_mcp_test "Error: missing required param" \
    '{"jsonrpc": "2.0", "method": "tools/call", "id": 107, "params": {"name": "execute", "arguments": {"skill": "kubernetes", "tool": "get"}}}' \
    '"id":107.*error' 10 2

# ============================================
# Print Summary
# ============================================
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
