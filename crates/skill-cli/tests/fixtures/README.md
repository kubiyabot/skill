# Test Fixtures for Claude Bridge

This directory contains test fixtures for validating the Claude Bridge skill generation functionality.

## Directory Structure

```
fixtures/
├── manifests/          # Test manifest files (.skill-engine.toml)
│   ├── minimal.toml    # 1 skill, 1 tool (baseline)
│   ├── small.toml      # 3 skills, 5 tools each (typical)
│   ├── medium.toml     # 10 skills (production scale)
│   ├── invalid-syntax.toml      # Malformed TOML (negative test)
│   ├── missing-required.toml    # Missing required fields
│   └── invalid-types.toml       # Wrong parameter types
├── skills/             # Reference SKILL.md files
│   └── reference-minimal.md     # Expected output for minimal.toml
└── README.md           # This file
```

## Manifest Fixtures

### minimal.toml

**Purpose**: Baseline testing and quick validation

**Contents**:
- 1 skill: "hello"
- 1 tool: "greet"
- 1 parameter: "name" (string, required)

**Use Cases**:
- Smoke testing skill generation
- Validating minimal requirements
- Quick iteration during development
- CI/CD sanity checks

**Expected Generation Time**: < 1 second

### small.toml

**Purpose**: Testing typical skill collections

**Contents**:
- 3 skills: user-management, file-operations, network-tools
- 15 tools total (5 per skill)
- Mix of required and optional parameters
- Various parameter types (string, number, boolean)

**Use Cases**:
- Standard integration testing
- Parameter handling validation
- Multi-skill generation testing
- Tool categorization verification

**Expected Generation Time**: < 5 seconds

### medium.toml

**Purpose**: Production-scale testing

**Contents**:
- 10 skills: kubernetes, docker, git, aws, terraform, database, monitoring, nginx, systemd, logging
- ~50+ tools total
- Complex parameter structures
- Real-world DevOps tooling examples

**Use Cases**:
- Performance testing
- Large-scale generation validation
- Complex parameter handling
- Production simulation

**Expected Generation Time**: < 30 seconds

### large.toml (Not Implemented Yet)

**Purpose**: Stress testing and scalability validation

**Planned Contents**:
- 50 skills
- 200+ tools
- Edge cases (special characters, long descriptions)
- Maximum complexity scenarios

**Use Cases**:
- Performance benchmarking
- Scalability testing
- Memory usage profiling
- Edge case discovery

**Expected Generation Time**: < 2 minutes

## Invalid Fixtures (Negative Testing)

### invalid-syntax.toml

**Purpose**: Test TOML parsing error handling

**Contains**:
- Malformed TOML syntax
- Missing closing quotes
- Incomplete array definitions
- Invalid field declarations

**Expected Behavior**: Should fail with clear parsing error message

### missing-required.toml

**Purpose**: Test validation of required fields

**Contains**:
- Skills without names
- Tools without descriptions
- Parameters without types

**Expected Behavior**: Should fail validation with helpful error messages indicating missing fields

### invalid-types.toml

**Purpose**: Test type validation

**Contains**:
- String fields with numeric values
- Boolean fields with string values
- Invalid type identifiers

**Expected Behavior**: Should fail with type mismatch errors

## Reference Files

### reference-minimal.md

**Purpose**: Golden file for comparison testing

**Contents**:
- Expected SKILL.md output for minimal.toml
- Proper YAML frontmatter
- All required sections
- Correct markdown formatting
- Example tool usage

**Use Cases**:
- Regression testing
- Output format validation
- Documentation generation verification
- Template validation

## Using Fixtures in Tests

### In Bash Tests

```bash
# Load minimal fixture
skill claude generate --manifest tests/fixtures/manifests/minimal.toml \
  --output /tmp/test-output

# Validate output
test -f /tmp/test-output/hello/SKILL.md
test -f /tmp/test-output/hello/TOOLS.md
test -x /tmp/test-output/hello/scripts/greet.sh
```

### In Rust Tests

```rust
use std::path::Path;

#[test]
fn test_minimal_fixture() {
    let manifest = Path::new("tests/fixtures/manifests/minimal.toml");
    assert!(manifest.exists());

    // Load and parse manifest
    let skills = load_manifest(manifest).unwrap();
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "hello");
}
```

### In Integration Tests

```bash
# Test all valid fixtures
for fixture in tests/fixtures/manifests/{minimal,small,medium}.toml; do
    echo "Testing $fixture..."
    skill claude generate --manifest "$fixture" --output /tmp/test-$$ --force
done

# Test invalid fixtures (should fail)
for fixture in tests/fixtures/manifests/invalid-*.toml; do
    echo "Testing $fixture (should fail)..."
    if skill claude generate --manifest "$fixture" 2>&1 | grep -q "error"; then
        echo "✓ Correctly failed"
    else
        echo "✗ Should have failed but didn't"
        exit 1
    fi
done
```

## Fixture Maintenance

### Adding New Fixtures

1. Create new `.toml` file in `manifests/`
2. Follow naming convention: `{purpose}.toml` or `invalid-{reason}.toml`
3. Add documentation to this README
4. Update relevant test scripts
5. Add expected output to `skills/` if applicable

### Updating Existing Fixtures

1. Maintain backward compatibility when possible
2. Update corresponding reference files in `skills/`
3. Re-run tests to verify no regressions
4. Update documentation if fixture purpose changes

### Validating Fixtures

```bash
# Validate TOML syntax
for file in tests/fixtures/manifests/*.toml; do
    toml check "$file" || echo "Invalid: $file"
done

# Generate and validate all fixtures
./tests/claude_bridge/scripts/validate-fixtures.sh
```

## Fixture Design Principles

1. **Realism**: Fixtures should represent real-world use cases
2. **Completeness**: Cover common and edge cases
3. **Maintainability**: Keep fixtures simple and well-documented
4. **Performance**: Balance thoroughness with test execution speed
5. **Isolation**: Each fixture should be self-contained

## Performance Expectations

| Fixture | Skills | Tools | Generation Time | Output Size |
|---------|--------|-------|-----------------|-------------|
| minimal | 1 | 1 | < 1s | ~2 KB |
| small | 3 | 15 | < 5s | ~15 KB |
| medium | 10 | 50+ | < 30s | ~100 KB |
| large | 50 | 200+ | < 2m | ~500 KB |

## Known Limitations

1. **large.toml**: Not yet implemented (planned for performance testing phase)
2. **Complex Parameters**: Nested objects and arrays need more comprehensive fixtures
3. **Streaming Tools**: Not yet represented in fixtures
4. **Authentication**: No fixtures with authentication/credential handling

## Future Enhancements

- [ ] Add large.toml for stress testing
- [ ] Create fixtures with nested parameter structures
- [ ] Add fixtures for streaming tool support
- [ ] Include examples with authentication
- [ ] Add fixtures for different skill runtime types (WASM, Native, Docker)
- [ ] Create fixtures for edge cases (Unicode, very long descriptions, special characters)

## Related Documentation

- [Claude Bridge Tests README](../README.md)
- [Test Automation Scripts](../test-skill-generation.sh)
- [PRD: Claude Bridge Testing](.taskmaster/docs/claude-bridge-testing-prd.txt)
