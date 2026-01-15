#!/bin/bash
# YAML Linter for Claude Agent Skills SKILL.md Frontmatter
#
# This script extracts YAML frontmatter from generated SKILL.md files
# and validates them using yamllint (if available).

set -e

SKILLS_DIR="${1:-.claude/skills}"
PASSED=0
FAILED=0
TOTAL=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo ""
echo "========================================"
echo "  YAML Frontmatter Validator"
echo "========================================"
echo ""

# Check if yamllint is installed
if ! command -v yamllint &> /dev/null; then
    echo -e "${YELLOW}yamllint not installed. Install with: pip install yamllint${NC}"
    echo -e "${YELLOW}Continuing with basic validation only...${NC}"
    echo ""
fi

# Find all SKILL.md files
SKILL_FILES=$(find "$SKILLS_DIR" -name "SKILL.md" 2>/dev/null || echo "")

if [ -z "$SKILL_FILES" ]; then
    echo -e "${RED}No SKILL.md files found in $SKILLS_DIR${NC}"
    exit 1
fi

# Process each SKILL.md file
for skill_md in $SKILL_FILES; do
    TOTAL=$((TOTAL + 1))
    skill_name=$(basename "$(dirname "$skill_md")")

    echo -e "${CYAN}[$TOTAL] Validating: $skill_name${NC}"

    # Extract YAML frontmatter (between first two --- lines)
    yaml_content=$(awk '/^---$/{flag++; next} flag==1{print}' "$skill_md")

    if [ -z "$yaml_content" ]; then
        echo -e "${RED}  FAILED: No YAML frontmatter found${NC}"
        FAILED=$((FAILED + 1))
        continue
    fi

    # Create temporary YAML file
    temp_yaml=$(mktemp)
    echo "$yaml_content" > "$temp_yaml"

    # Validate with yamllint if available
    if command -v yamllint &> /dev/null; then
        if yamllint -d "{extends: default, rules: {line-length: disable, document-start: disable}}" "$temp_yaml" 2>&1 | grep -q "error"; then
            echo -e "${RED}  FAILED: yamllint errors${NC}"
            yamllint -d "{extends: default, rules: {line-length: disable, document-start: disable}}" "$temp_yaml" 2>&1 | sed 's/^/    /'
            FAILED=$((FAILED + 1))
            rm "$temp_yaml"
            continue
        fi
    fi

    # Basic validation: check for required fields
    if ! echo "$yaml_content" | grep -q "^name:"; then
        echo -e "${RED}  FAILED: Missing 'name' field${NC}"
        FAILED=$((FAILED + 1))
        rm "$temp_yaml"
        continue
    fi

    if ! echo "$yaml_content" | grep -q "^description:"; then
        echo -e "${RED}  FAILED: Missing 'description' field${NC}"
        FAILED=$((FAILED + 1))
        rm "$temp_yaml"
        continue
    fi

    # Extract name and validate format (lowercase alphanumeric + hyphens)
    name=$(echo "$yaml_content" | grep "^name:" | sed 's/name: *//' | tr -d '"' | tr -d "'")
    if ! echo "$name" | grep -qE "^[a-z0-9-]{1,64}$"; then
        echo -e "${RED}  FAILED: Invalid name format '$name' (must be lowercase alphanumeric + hyphens, max 64 chars)${NC}"
        FAILED=$((FAILED + 1))
        rm "$temp_yaml"
        continue
    fi

    # Extract description and validate length
    description=$(echo "$yaml_content" | sed -n '/^description:/,/^[a-z]/p' | sed '1s/description: *//' | sed '$d' | tr -d '\n')
    desc_len=${#description}
    if [ "$desc_len" -gt 1024 ]; then
        echo -e "${RED}  FAILED: Description too long ($desc_len chars, max 1024)${NC}"
        FAILED=$((FAILED + 1))
        rm "$temp_yaml"
        continue
    fi

    # Check for HTML/XML tags in description
    if echo "$description" | grep -q "[<>]"; then
        echo -e "${RED}  FAILED: Description contains HTML/XML tags${NC}"
        FAILED=$((FAILED + 1))
        rm "$temp_yaml"
        continue
    fi

    echo -e "${GREEN}  PASSED${NC}"
    echo "    name: $name"
    echo "    description: $(echo "$description" | cut -c1-60)..."
    PASSED=$((PASSED + 1))

    rm "$temp_yaml"
done

# Print summary
echo ""
echo "========================================"
echo "  Validation Summary"
echo "========================================"
echo -e "  Total:  $TOTAL"
echo -e "  ${GREEN}Passed: $PASSED${NC}"
echo -e "  ${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All YAML frontmatter is valid!${NC}"
    exit 0
else
    echo -e "${RED}Some validations failed.${NC}"
    exit 1
fi
