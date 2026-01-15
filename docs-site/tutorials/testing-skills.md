# Testing Skills

Learn how to test your skills to ensure they work correctly. This tutorial covers testing strategies for both WASM and native skills.

**Time:** 15 minutes

## Why Test Skills?

- **Catch bugs early** before users encounter them
- **Document behavior** through test cases
- **Enable refactoring** with confidence
- **Improve reliability** for production use

## Testing Strategies

### 1. Manual Testing

The simplest approach - run your skill and check output:

```bash
# Install the skill
skill install ./my-skill

# Test each tool
skill run my-skill:tool1 param="value"
skill run my-skill:tool2 other="test"

# Check output matches expectations
```

### 2. Shell Script Testing

Create automated tests in a shell script:

```bash
#!/bin/bash
# test.sh - Test suite for my-skill

set -e  # Exit on first failure

echo "Testing my-skill..."

# Test 1: Basic functionality
OUTPUT=$(skill run my-skill:hello name="Test")
if [[ "$OUTPUT" != *"Hello, Test"* ]]; then
    echo "FAIL: hello tool output incorrect"
    exit 1
fi
echo "✓ Test 1: hello tool works"

# Test 2: Error handling
OUTPUT=$(skill run my-skill:hello 2>&1) || true
if [[ "$OUTPUT" != *"required"* ]]; then
    echo "FAIL: Should require name parameter"
    exit 1
fi
echo "✓ Test 2: error handling works"

# Test 3: Optional parameters
OUTPUT=$(skill run my-skill:goodbye name="Test" formal=true)
if [[ "$OUTPUT" != *"Farewell"* ]]; then
    echo "FAIL: formal goodbye incorrect"
    exit 1
fi
echo "✓ Test 3: optional parameters work"

echo ""
echo "All tests passed!"
```

Run with:
```bash
chmod +x test.sh
./test.sh
```

### 3. JSON Output Testing

Use JSON output for easier parsing:

```bash
# If your skill supports JSON output
OUTPUT=$(skill run my-skill:data --format json)

# Parse with jq
VALUE=$(echo "$OUTPUT" | jq -r '.result')
if [[ "$VALUE" != "expected" ]]; then
    echo "FAIL: unexpected value: $VALUE"
    exit 1
fi
```

### 4. Unit Testing (WASM Skills)

For JavaScript skills, use Node.js testing:

```javascript
// test/skill.test.js
import { getTools, executeTool, getMetadata } from '../skill.js';

describe('Greeting Skill', () => {
  describe('getMetadata', () => {
    it('returns skill metadata', () => {
      const meta = getMetadata();
      expect(meta.name).toBe('greeting');
      expect(meta.version).toMatch(/^\d+\.\d+\.\d+$/);
    });
  });

  describe('getTools', () => {
    it('returns array of tools', () => {
      const tools = getTools();
      expect(Array.isArray(tools)).toBe(true);
      expect(tools.length).toBeGreaterThan(0);
    });

    it('hello tool has required parameter', () => {
      const tools = getTools();
      const hello = tools.find(t => t.name === 'hello');
      expect(hello).toBeDefined();

      const nameParam = hello.parameters.find(p => p.name === 'name');
      expect(nameParam.required).toBe(true);
    });
  });

  describe('executeTool', () => {
    it('hello returns greeting', () => {
      const result = executeTool('hello', JSON.stringify({ name: 'World' }));
      expect(result.success).toBe(true);
      expect(result.output).toContain('Hello, World');
    });

    it('handles unknown tool', () => {
      const result = executeTool('unknown', '{}');
      expect(result.success).toBe(false);
      expect(result.errorMessage).toContain('Unknown tool');
    });

    it('handles missing required parameter', () => {
      const result = executeTool('hello', '{}');
      // Skill should handle missing params gracefully
      expect(result.success).toBe(false);
    });
  });
});
```

Run with Jest:
```bash
npm install --save-dev jest
npx jest
```

### 5. Python Testing (Python Skills)

```python
# test_skill.py
import pytest
from skill import get_tools, execute_tool, get_metadata

def test_metadata():
    meta = get_metadata()
    assert meta['name'] == 'my-skill'
    assert 'version' in meta

def test_tools_defined():
    tools = get_tools()
    assert len(tools) > 0
    assert all('name' in t for t in tools)

def test_hello_tool():
    result = execute_tool('hello', '{"name": "World"}')
    assert result['success'] is True
    assert 'Hello, World' in result['output']

def test_unknown_tool():
    result = execute_tool('nonexistent', '{}')
    assert result['success'] is False

def test_missing_parameter():
    result = execute_tool('hello', '{}')
    assert result['success'] is False
```

Run with pytest:
```bash
pip install pytest
pytest test_skill.py -v
```

## Testing Native Skills

### Test Command Generation

For SKILL.md skills, test that correct commands are generated:

```bash
#!/bin/bash
# test-native.sh

# Test that skill is installed
skill info kubernetes > /dev/null || {
    echo "FAIL: kubernetes skill not installed"
    exit 1
}

# Test basic command
OUTPUT=$(skill run kubernetes:get resource=pods namespace=default 2>&1)
if [[ $? -ne 0 && "$OUTPUT" != *"connection refused"* ]]; then
    # Expected: either success or kubectl not connected
    echo "FAIL: unexpected error: $OUTPUT"
    exit 1
fi
echo "✓ Command generation works"

# Test parameter handling
OUTPUT=$(skill run kubernetes:get resource=deployments output=json 2>&1)
# Verify the output format parameter is passed correctly
echo "✓ Parameters passed correctly"
```

## Integration Testing

Test the full flow with real services:

```bash
#!/bin/bash
# integration-test.sh

# Start test environment
docker-compose up -d

# Wait for services
sleep 5

# Run tests against real services
skill run database:query sql="SELECT 1"
skill run api:health

# Cleanup
docker-compose down
```

## Test Organization

Recommended structure:

```
my-skill/
├── skill.js
├── SKILL.md
├── tests/
│   ├── unit/
│   │   └── skill.test.js
│   ├── integration/
│   │   └── api.test.js
│   └── e2e/
│       └── test.sh
├── package.json
└── README.md
```

## CI/CD Integration

Add tests to your CI pipeline:

```yaml
# .github/workflows/test.yml
name: Test Skill

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Skill CLI
        run: |
          curl -fsSL https://skill-ai.dev/install.sh | sh
          echo "$HOME/.skill-engine/bin" >> $GITHUB_PATH

      - name: Install skill
        run: skill install .

      - name: Run tests
        run: ./tests/e2e/test.sh

      - name: Run unit tests
        run: npm test
```

## Best Practices

### 1. Test Edge Cases

```javascript
// Test empty input
executeTool('search', '{"query": ""}');

// Test special characters
executeTool('hello', '{"name": "O\'Brien"}');

// Test long input
executeTool('process', '{"data": "' + 'x'.repeat(10000) + '"}');
```

### 2. Test Error Paths

```javascript
// Missing required params
// Invalid param types
// API failures
// Timeout handling
```

### 3. Use Meaningful Assertions

```javascript
// Bad: just check success
expect(result.success).toBe(true);

// Good: check actual output
expect(result.success).toBe(true);
expect(result.output).toContain('expected content');
expect(result.errorMessage).toBeNull();
```

### 4. Keep Tests Fast

- Mock external APIs in unit tests
- Use integration tests sparingly
- Run slow tests separately

## Next Steps

- Add tests to your existing skills
- Set up CI/CD for automatic testing
- Explore [example skills](/examples/) for test patterns
- Learn about [debugging](/guides/troubleshooting)
