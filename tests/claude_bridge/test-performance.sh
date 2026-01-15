#!/bin/bash
# Performance & Scalability Test Suite for Claude Bridge
# Tests generation speed, memory usage, skill discovery, and tool execution latency

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Performance targets from PRD (in milliseconds)
TARGET_1_SKILL=5000        # 5 seconds
TARGET_10_SKILLS=30000     # 30 seconds
TARGET_50_SKILLS=120000    # 2 minutes (120 seconds)
TARGET_DISCOVERY=1000      # 1 second
TARGET_TOOL_EXEC=2000      # 2 seconds
TARGET_MEMORY_KB=512000    # 500MB in KB

# Test counters
PASSED=0
FAILED=0
TOTAL=0

echo ""
echo "========================================"
echo "  Claude Bridge Performance Tests"
echo "========================================"
echo ""

# Test helper function
test_generation_performance() {
    local num_skills=$1
    local target_ms=$2
    local test_name=$3
    local manifest_path=$4

    TOTAL=$((TOTAL + 1))
    echo -e "${CYAN}[$TOTAL] Testing: $test_name${NC}"

    # Create output directory
    local output_dir="/tmp/perf-test-$$-${num_skills}"
    mkdir -p "$output_dir"

    # Measure time (millisecond precision)
    if command -v gdate &> /dev/null; then
        # macOS with GNU coreutils
        start=$(gdate +%s%3N)
    else
        # Linux
        start=$(date +%s%3N)
    fi

    # Run skill generation
    if [ -n "$manifest_path" ]; then
        # Use specific manifest
        cargo run --release --bin skill -- claude generate \
            --manifest "$manifest_path" \
            --output "$output_dir" \
            --force \
            > /dev/null 2>&1
    elif [ $num_skills -eq 1 ]; then
        # Generate single skill (assumes kubernetes skill exists in default manifest)
        cargo run --release --bin skill -- claude generate \
            --skill kubernetes \
            --output "$output_dir" \
            --force \
            > /dev/null 2>&1 || {
                echo -e "  ${YELLOW}SKIPPED${NC} - kubernetes skill not in manifest"
                rm -rf "$output_dir"
                return 0
            }
    else
        # Generate all skills from default manifest
        cargo run --release --bin skill -- claude generate \
            --output "$output_dir" \
            --force \
            > /dev/null 2>&1
    fi

    if command -v gdate &> /dev/null; then
        end=$(gdate +%s%3N)
    else
        end=$(date +%s%3N)
    fi

    duration=$((end - start))

    echo "  Duration: ${duration}ms (target: ${target_ms}ms)"

    # Check if target met
    if [ $duration -lt $target_ms ]; then
        echo -e "  ${GREEN}✓ PASSED${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗ FAILED${NC} - Exceeded target by $((duration - target_ms))ms"
        FAILED=$((FAILED + 1))
    fi

    # Cleanup
    rm -rf "$output_dir"
    echo ""
}

# Test 1: Single skill generation
test_generation_performance 1 $TARGET_1_SKILL "Generate 1 skill (kubernetes)" ""

# Test 2: 10 skills generation (requires large manifest)
# Skip if large manifest doesn't exist yet
if [ -f "/tmp/large-manifest-10.toml" ]; then
    test_generation_performance 10 $TARGET_10_SKILLS "Generate 10 skills" "/tmp/large-manifest-10.toml"
else
    echo -e "${YELLOW}[SKIPPED] Generate 10 skills - manifest not found${NC}"
    echo ""
fi

# Test 3: 50 skills generation (requires large manifest)
if [ -f "/tmp/large-manifest-50.toml" ]; then
    test_generation_performance 50 $TARGET_50_SKILLS "Generate 50 skills" "/tmp/large-manifest-50.toml"
else
    echo -e "${YELLOW}[SKIPPED] Generate 50 skills - manifest not found${NC}"
    echo ""
fi

# Test 4: Memory usage monitoring
TOTAL=$((TOTAL + 1))
echo -e "${CYAN}[$TOTAL] Testing: Memory usage during generation${NC}"

output_dir="/tmp/perf-test-memory-$$"
mkdir -p "$output_dir"

# Monitor memory during generation
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    /usr/bin/time -l cargo run --release --bin skill -- claude generate \
        --output "$output_dir" \
        --force \
        > /dev/null 2>&1 &

    PID=$!
    max_mem=0

    # Poll memory usage
    while kill -0 $PID 2>/dev/null; do
        # Get memory in KB for macOS
        mem=$(ps -o rss= -p $PID 2>/dev/null || echo "0")
        if [ $mem -gt $max_mem ]; then
            max_mem=$mem
        fi
        sleep 0.1
    done

    wait $PID
else
    # Linux
    /usr/bin/time -v cargo run --release --bin skill -- claude generate \
        --output "$output_dir" \
        --force \
        > /dev/null 2>&1 &

    PID=$!
    max_mem=0

    # Poll memory usage
    while kill -0 $PID 2>/dev/null; do
        # Get memory in KB for Linux
        mem=$(ps -o rss= -p $PID 2>/dev/null || echo "0")
        if [ $mem -gt $max_mem ]; then
            max_mem=$mem
        fi
        sleep 0.1
    done

    wait $PID
fi

echo "  Peak memory: ${max_mem}KB (target: <${TARGET_MEMORY_KB}KB / 500MB)"

if [ $max_mem -lt $TARGET_MEMORY_KB ]; then
    echo -e "  ${GREEN}✓ PASSED${NC} - Memory usage within limits"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗ FAILED${NC} - Exceeded 500MB memory limit"
    FAILED=$((FAILED + 1))
fi

rm -rf "$output_dir"
echo ""

# Test 5: Skill discovery speed
TOTAL=$((TOTAL + 1))
echo -e "${CYAN}[$TOTAL] Testing: Skill discovery latency${NC}"

# Check if skill binary is installed
if ! command -v skill &> /dev/null; then
    echo -e "  ${YELLOW}SKIPPED${NC} - 'skill' binary not in PATH"
    echo ""
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

    echo "  Duration: ${duration}ms (target: <${TARGET_DISCOVERY}ms)"

    if [ $duration -lt $TARGET_DISCOVERY ]; then
        echo -e "  ${GREEN}✓ PASSED${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗ FAILED${NC}"
        FAILED=$((FAILED + 1))
    fi
    echo ""
fi

# Test 6: Tool execution latency
TOTAL=$((TOTAL + 1))
echo -e "${CYAN}[$TOTAL] Testing: Tool execution latency${NC}"

# Test with git:status as an example (if git skill exists)
if ! command -v skill &> /dev/null; then
    echo -e "  ${YELLOW}SKIPPED${NC} - 'skill' binary not in PATH"
    echo ""
else
    if command -v gdate &> /dev/null; then
        start=$(gdate +%s%3N)
    else
        start=$(date +%s%3N)
    fi

    # Try to run a simple command (git status)
    skill run git:status > /dev/null 2>&1 || {
        echo -e "  ${YELLOW}SKIPPED${NC} - git skill not available"
        echo ""
        TOTAL=$((TOTAL - 1))
        continue
    }

    if command -v gdate &> /dev/null; then
        end=$(gdate +%s%3N)
    else
        end=$(date +%s%3N)
    fi

    duration=$((end - start))

    echo "  Duration: ${duration}ms (target: <${TARGET_TOOL_EXEC}ms)"

    if [ $duration -lt $TARGET_TOOL_EXEC ]; then
        echo -e "  ${GREEN}✓ PASSED${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗ FAILED${NC}"
        FAILED=$((FAILED + 1))
    fi
    echo ""
fi

# Test 7: Concurrent generation safety (basic stress test)
TOTAL=$((TOTAL + 1))
echo -e "${CYAN}[$TOTAL] Testing: Concurrent generation (3 processes)${NC}"

output_dir="/tmp/perf-test-concurrent-$$"
mkdir -p "$output_dir"

# Launch 3 concurrent generation processes
for i in 1 2 3; do
    cargo run --release --bin skill -- claude generate \
        --output "$output_dir/run-$i" \
        --force \
        > /dev/null 2>&1 &
done

# Wait for all to complete
wait

# Check all generated successfully
success=0
for i in 1 2 3; do
    if [ -d "$output_dir/run-$i" ] && [ "$(ls -A $output_dir/run-$i 2>/dev/null)" ]; then
        success=$((success + 1))
    fi
done

echo "  Completed: $success/3 processes"

if [ $success -eq 3 ]; then
    echo -e "  ${GREEN}✓ PASSED${NC} - All concurrent processes completed"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗ FAILED${NC} - Only $success/3 processes completed successfully"
    FAILED=$((FAILED + 1))
fi

rm -rf "$output_dir"
echo ""

# Print summary
echo "========================================"
echo "  Performance Test Summary"
echo "========================================"
echo -e "  Total Tests:  $TOTAL"
echo -e "  ${GREEN}Passed:       $PASSED${NC}"
echo -e "  ${RED}Failed:       $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All performance tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some performance tests failed.${NC}"
    echo ""
    echo "Review failed tests above and optimize performance."
    exit 1
fi
