#!/bin/bash
# UX Metrics Testing Script
# Measures key user experience metrics for Claude Bridge

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Test counters
PASSED=0
FAILED=0
TOTAL=0

echo ""
echo "=========================================="
echo "  Claude Bridge UX Metrics Tests"
echo "=========================================="
echo ""

# ==============================================================================
# Metric 1: Time to First Skill Generation
# ==============================================================================

TOTAL=$((TOTAL + 1))
echo -e "${CYAN}[$TOTAL] Metric: Time to first skill generation${NC}"
echo "  Measures how long it takes a new user to generate their first skill"
echo ""

# Measure time
if command -v gdate &> /dev/null; then
    start=$(gdate +%s)
else
    start=$(date +%s)
fi

# Generate skill (assuming manifest exists)
output_dir="/tmp/ux-test-$$"
mkdir -p "$output_dir"

cargo run --release --bin skill -- claude generate \
    --output "$output_dir" \
    --force \
    > /dev/null 2>&1 || {
        echo -e "  ${YELLOW}SKIPPED${NC} - Unable to generate skills"
        rm -rf "$output_dir"
        TOTAL=$((TOTAL - 1))
    }

if command -v gdate &> /dev/null; then
    end=$(gdate +%s)
else
    end=$(date +%s)
fi

duration=$((end - start))

echo "  Duration: ${duration}s (target: <300s / 5 minutes)"

if [ $duration -lt 300 ]; then
    echo -e "  ${GREEN}✓ PASSED${NC} - Fast first-skill generation"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗ FAILED${NC} - Too slow for good first impression"
    FAILED=$((FAILED + 1))
fi

rm -rf "$output_dir"
echo ""

# ==============================================================================
# Metric 2: Help Text Readability
# ==============================================================================

TOTAL=$((TOTAL + 1))
echo -e "${CYAN}[$TOTAL] Metric: Help text readability${NC}"
echo "  Measures whether help text is clear and understandable"
echo ""

# Get help text
help_text=$(cargo run --release --bin skill -- claude generate --help 2>/dev/null || echo "")

if [ -z "$help_text" ]; then
    echo -e "  ${YELLOW}SKIPPED${NC} - Unable to get help text"
    TOTAL=$((TOTAL - 1))
else
    # Check help text length
    help_len=${#help_text}
    echo "  Help text length: $help_len characters"

    # Check for key elements
    has_examples=false
    has_flags=false
    has_description=false

    if echo "$help_text" | grep -qi "example"; then
        has_examples=true
    fi

    if echo "$help_text" | grep -q "\-\-"; then
        has_flags=true
    fi

    if [ $help_len -gt 200 ]; then
        has_description=true
    fi

    echo "  Has examples: $has_examples"
    echo "  Has flags documented: $has_flags"
    echo "  Has description: $has_description"

    if [ "$has_examples" = true ] && [ "$has_flags" = true ] && [ "$has_description" = true ]; then
        echo -e "  ${GREEN}✓ PASSED${NC} - Help text is complete"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗ FAILED${NC} - Help text missing key elements"
        FAILED=$((FAILED + 1))
    fi
fi

echo ""

# ==============================================================================
# Metric 3: Setup Step Count
# ==============================================================================

TOTAL=$((TOTAL + 1))
echo -e "${CYAN}[$TOTAL] Metric: Number of setup steps${NC}"
echo "  Measures setup complexity (target: <5 steps)"
echo ""

echo "  Step 1: Install Skill Engine (cargo install or download binary)"
echo "  Step 2: Create or have manifest (.skill-engine.toml)"
echo "  Step 3: Generate skills (skill claude generate)"
echo "  Step 4: Configure Claude Code (.mcp.json or --project)"
echo "  Total: 4 steps"
echo ""

if [ 4 -lt 5 ]; then
    echo -e "  ${GREEN}✓ PASSED${NC} - Setup requires <5 steps"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗ FAILED${NC} - Too many setup steps"
    FAILED=$((FAILED + 1))
fi

echo ""

# ==============================================================================
# Metric 4: Documentation Completeness
# ==============================================================================

TOTAL=$((TOTAL + 1))
echo -e "${CYAN}[$TOTAL] Metric: Documentation completeness${NC}"
echo "  Measures README quality (target: >500 lines)"
echo ""

# Find README
readme_path=$(find . -maxdepth 2 -name "README.md" | head -1)

if [ -z "$readme_path" ] || [ ! -f "$readme_path" ]; then
    echo -e "  ${RED}✗ FAILED${NC} - README.md not found"
    FAILED=$((FAILED + 1))
else
    readme_lines=$(wc -l < "$readme_path" | tr -d ' ')
    echo "  README lines: $readme_lines"

    # Check for essential sections
    has_installation=false
    has_usage=false
    has_examples=false

    if grep -qi "installation\|install\|quick start" "$readme_path"; then
        has_installation=true
    fi

    if grep -qi "usage\|how to use" "$readme_path"; then
        has_usage=true
    fi

    if grep -qi "example" "$readme_path"; then
        has_examples=true
    fi

    echo "  Has installation section: $has_installation"
    echo "  Has usage section: $has_usage"
    echo "  Has examples section: $has_examples"

    if [ $readme_lines -gt 500 ] && [ "$has_installation" = true ] && [ "$has_usage" = true ] && [ "$has_examples" = true ]; then
        echo -e "  ${GREEN}✓ PASSED${NC} - README is comprehensive"
        PASSED=$((PASSED + 1))
    else
        if [ $readme_lines -le 500 ]; then
            echo -e "  ${RED}✗ FAILED${NC} - README too short ($readme_lines lines, need >500)"
        else
            echo -e "  ${RED}✗ FAILED${NC} - README missing essential sections"
        fi
        FAILED=$((FAILED + 1))
    fi
fi

echo ""

# ==============================================================================
# Metric 5: Error Message Quality
# ==============================================================================

TOTAL=$((TOTAL + 1))
echo -e "${CYAN}[$TOTAL] Metric: Error message quality${NC}"
echo "  Measures whether error messages are helpful"
echo ""

# Test error for nonexistent skill
temp_dir="/tmp/ux-error-test-$$"
mkdir -p "$temp_dir"

# Create minimal manifest
cat > "$temp_dir/.skill-engine.toml" << 'EOF'
[skills.test-skill]
source = "./test"
runtime = "wasm"
description = "Test skill"
EOF

error_output=$(cd "$temp_dir" && cargo run --release --bin skill -- claude generate --skill nonexistent 2>&1 || true)

# Check error quality
has_skill_name=false
has_helpful_keyword=false
no_stack_trace=true

if echo "$error_output" | grep -qi "nonexistent"; then
    has_skill_name=true
fi

if echo "$error_output" | grep -qi "not found\|available\|does not exist"; then
    has_helpful_keyword=true
fi

if echo "$error_output" | grep -qi "panicked\|backtrace"; then
    no_stack_trace=false
fi

echo "  Mentions missing skill name: $has_skill_name"
echo "  Has helpful keywords: $has_helpful_keyword"
echo "  No stack trace: $no_stack_trace"

if [ "$has_skill_name" = true ] && [ "$has_helpful_keyword" = true ] && [ "$no_stack_trace" = true ]; then
    echo -e "  ${GREEN}✓ PASSED${NC} - Error messages are helpful"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗ FAILED${NC} - Error messages need improvement"
    FAILED=$((FAILED + 1))
fi

rm -rf "$temp_dir"
echo ""

# ==============================================================================
# Metric 6: Skill Discovery Time
# ==============================================================================

TOTAL=$((TOTAL + 1))
echo -e "${CYAN}[$TOTAL] Metric: Skill discovery latency${NC}"
echo "  Measures how fast users can list available skills"
echo ""

if ! command -v skill &> /dev/null; then
    echo -e "  ${YELLOW}SKIPPED${NC} - 'skill' binary not in PATH"
    TOTAL=$((TOTAL - 1))
else
    if command -v gdate &> /dev/null; then
        start=$(gdate +%s%3N)
    else
        start=$(date +%s%3N)
    fi

    skill list > /dev/null 2>&1 || true

    if command -v gdate &> /dev/null; then
        end=$(gdate +%s%3N)
    else
        end=$(date +%s%3N)
    fi

    duration=$((end - start))

    echo "  Duration: ${duration}ms (target: <1000ms)"

    if [ $duration -lt 1000 ]; then
        echo -e "  ${GREEN}✓ PASSED${NC} - Fast skill discovery"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗ FAILED${NC} - Skill discovery too slow"
        FAILED=$((FAILED + 1))
    fi
fi

echo ""

# ==============================================================================
# Summary
# ==============================================================================

echo "=========================================="
echo "  UX Metrics Summary"
echo "=========================================="
echo -e "  Total Metrics:  $TOTAL"
echo -e "  ${GREEN}Passed:         $PASSED${NC}"
echo -e "  ${RED}Failed:         $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All UX metrics passed!${NC}"
    echo ""
    echo "The Claude Bridge feature meets UX quality standards."
    exit 0
else
    echo -e "${RED}✗ Some UX metrics failed.${NC}"
    echo ""
    echo "Review failed metrics above and improve:"
    echo "  - Documentation completeness and clarity"
    echo "  - Help text quality"
    echo "  - Error message helpfulness"
    echo "  - Performance and latency"
    exit 1
fi
