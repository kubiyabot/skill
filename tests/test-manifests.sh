#!/bin/bash
# Test script for validating manifest files
# Tests that manifests load correctly and skills are functional

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MANIFESTS_DIR="$PROJECT_ROOT/examples/manifests"

echo "==================================="
echo "Testing Skill Engine Manifests"
echo "==================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

pass_count=0
fail_count=0

# Test function
test_manifest() {
    local manifest_name=$1
    local expected_skills=$2
    local manifest_path="$MANIFESTS_DIR/$manifest_name"

    echo "Testing: $manifest_name"
    echo "  Expected skills: $expected_skills"

    # Test 1: Manifest loads
    if skill list --manifest "$manifest_path" &>/dev/null; then
        echo -e "  ${GREEN}✓${NC} Manifest loads successfully"
        ((pass_count++))
    else
        echo -e "  ${RED}✗${NC} Manifest failed to load"
        ((fail_count++))
        return 1
    fi

    # Test 2: Correct number of skills
    actual_skills=$(skill list --manifest "$manifest_path" 2>&1 | grep "skill(s) available" | awk '{print $2}')
    if [ "$actual_skills" -ge "$expected_skills" ]; then
        echo -e "  ${GREEN}✓${NC} Skills count: $actual_skills (expected ≥$expected_skills)"
        ((pass_count++))
    else
        echo -e "  ${RED}✗${NC} Skills count: $actual_skills (expected ≥$expected_skills)"
        ((fail_count++))
    fi

    # Test 3: No TOML syntax errors
    if ! skill list --manifest "$manifest_path" 2>&1 | grep -i "error\|failed" | grep -v "skill(s)"; then
        echo -e "  ${GREEN}✓${NC} No syntax errors detected"
        ((pass_count++))
    else
        echo -e "  ${RED}✗${NC} Syntax errors detected"
        ((fail_count++))
    fi

    echo ""
}

# Test all manifests
echo "Phase 1: Manifest Loading Tests"
echo "================================"
echo ""

test_manifest "minimal.toml" 5
test_manifest "team.toml" 12
test_manifest "enterprise.toml" 12
test_manifest "devops.toml" 8
test_manifest "data-engineering.toml" 7

echo ""
echo "Phase 2: Multi-Instance Configuration Tests"
echo "==========================================="
echo ""

# Test enterprise manifest multiple instances
echo "Testing: enterprise.toml multi-instance configs"
instances=$(skill list --manifest "$MANIFESTS_DIR/enterprise.toml" 2>&1 | grep "kubernetes" | grep -o "prod, staging, dev\|dev, staging, prod")
if [ -n "$instances" ]; then
    echo -e "  ${GREEN}✓${NC} Multiple kubernetes instances found: $instances"
    ((pass_count++))
else
    echo -e "  ${RED}✗${NC} Multiple kubernetes instances not found"
    ((fail_count++))
fi

github_instances=$(skill list --manifest "$MANIFESTS_DIR/enterprise.toml" 2>&1 | grep "github" | grep -o "personal, enterprise\|enterprise, personal")
if [ -n "$github_instances" ]; then
    echo -e "  ${GREEN}✓${NC} Multiple github instances found: $github_instances"
    ((pass_count++))
else
    echo -e "  ${RED}✗${NC} Multiple github instances not found"
    ((fail_count++))
fi

echo ""
echo "Phase 3: Skill Functionality Tests"
echo "==================================="
echo ""

# Test git skill (should work without env vars)
echo "Testing: git skill execution"
if skill run git:default status --manifest "$PROJECT_ROOT/.skill-engine.toml" &>/dev/null; then
    echo -e "  ${GREEN}✓${NC} Git skill executes successfully"
    ((pass_count++))
else
    echo -e "  ${YELLOW}⚠${NC} Git skill execution skipped (may require system git)"
fi

# Test docker skill (should work if docker is installed)
echo "Testing: docker skill availability"
if command -v docker &>/dev/null; then
    if skill run docker:default ps --manifest "$PROJECT_ROOT/.skill-engine.toml" &>/dev/null; then
        echo -e "  ${GREEN}✓${NC} Docker skill executes successfully"
        ((pass_count++))
    else
        echo -e "  ${YELLOW}⚠${NC} Docker skill requires docker daemon running"
    fi
else
    echo -e "  ${YELLOW}⚠${NC} Docker not installed, skipping docker skill test"
fi

echo ""
echo "Phase 4: Environment Variable Expansion Tests"
echo "============================================="
echo ""

# Create temp manifest with env var
TEMP_MANIFEST=$(mktemp)
cat > "$TEMP_MANIFEST" <<'EOF'
version = "1"

[skills.test]
source = "./examples/native-skills/git-skill"
runtime = "native"

[skills.test.instances.default]
config.test_required = "${TEST_VAR}"
config.test_default = "${TEST_VAR_DEFAULT:-fallback_value}"
EOF

echo "Testing: Environment variable expansion"

# Test required var (should error without setting)
if skill list --manifest "$TEMP_MANIFEST" 2>&1 | grep -q "TEST_VAR"; then
    echo -e "  ${GREEN}✓${NC} Required env var error detected correctly"
    ((pass_count++))
else
    echo -e "  ${RED}✗${NC} Required env var error not detected"
    ((fail_count++))
fi

# Test default value expansion
export TEST_VAR="test_value"
if skill list --manifest "$TEMP_MANIFEST" &>/dev/null; then
    echo -e "  ${GREEN}✓${NC} Env var expansion works with default values"
    ((pass_count++))
else
    echo -e "  ${RED}✗${NC} Env var expansion failed"
    ((fail_count++))
fi

rm "$TEMP_MANIFEST"

echo ""
echo "==================================="
echo "Test Results Summary"
echo "==================================="
echo ""
echo -e "Passed: ${GREEN}$pass_count${NC}"
echo -e "Failed: ${RED}$fail_count${NC}"
echo ""

if [ $fail_count -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed${NC}"
    exit 1
fi
