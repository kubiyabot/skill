# Testing Skills

Complete guide to testing Skill Engine skills and running the project's test suite.

## Quick Reference

| Command | Description |
|---------|-------------|
| `cargo test --workspace` | Run all Rust tests |
| `cargo test -p skill-cli` | Test specific crate |
| `./tests/mcp_integration_tests.sh` | MCP protocol tests (45 tests) |
| `cargo bench` | Performance benchmarks |
| `cargo tarpaulin --workspace --out Html` | Generate coverage report |

## Overview

Testing skills ensures they work correctly across different:
- **Runtimes**: WASM, Docker, Native
- **Environments**: Development, staging, production
- **Inputs**: Valid, invalid, edge cases
- **Integrations**: External APIs and services

## Testing Strategies

### 1. Manual Testing
Quick validation during development

### 2. Unit Testing
Test individual tool functions

### 3. Integration Testing
Test full skill execution

### 4. End-to-End Testing
Test with real AI agents (Claude Code)

## Manual Testing

### Basic Execution

```bash
# Test skill installation
skill install ./my-skill

# List available tools
skill list my-skill

# Test tool execution
skill run my-skill my-tool --param value

# Check output
echo $?  # Exit code (0 = success)
```

### Dry Run Mode

```bash
# Preview without executing
skill run --dry-run my-skill my-tool --param value

# Shows what would be executed
```

### Debug Mode

```bash
# Enable debug logging
export SKILL_LOG_LEVEL=debug
skill run my-skill my-tool --param value

# Rust-level debugging
export RUST_LOG=skill_runtime=trace
skill run my-skill my-tool
```

### Test Different Instances

```bash
# Test development instance
skill run my-skill:dev my-tool

# Test staging instance
skill run my-skill:staging my-tool

# Test production instance
skill run my-skill:prod my-tool
```

## Unit Testing

### WASM Skills (JavaScript/TypeScript)

```typescript
// src/skill.test.ts
import { describe, it, expect } from 'vitest';
import { execute } from './skill';

describe('MySkill', () => {
  it('should process valid input', async () => {
    const result = await execute({
      tool: 'process',
      parameters: {
        input: 'test data'
      }
    });

    expect(result.status).toBe('success');
    expect(result.output).toContain('processed');
  });

  it('should reject invalid input', async () => {
    await expect(
      execute({
        tool: 'process',
        parameters: {
          input: ''  // Empty input
        }
      })
    ).rejects.toThrow('Input required');
  });

  it('should handle edge cases', async () => {
    const result = await execute({
      tool: 'process',
      parameters: {
        input: 'a'.repeat(10000)  // Large input
      }
    });

    expect(result.status).toBe('success');
  });
});
```

**Run tests**:
```bash
npm test
```

### Native Skills (Node.js)

```javascript
// skill.test.js
const { execute } = require('./skill');

describe('Kubernetes Skill', () => {
  beforeEach(() => {
    // Mock kubectl command
    process.env.KUBECTL = 'echo';
  });

  test('get pods should return valid JSON', async () => {
    const result = await execute({
      tool: 'get',
      parameters: {
        resource: 'pods',
        namespace: 'default'
      }
    });

    expect(result).toBeInstanceOf(Object);
    expect(result.items).toBeInstanceOf(Array);
  });

  test('should validate resource names', async () => {
    await expect(
      execute({
        tool: 'get',
        parameters: {
          resource: 'invalid-resource',
          namespace: 'default'
        }
      })
    ).rejects.toThrow();
  });
});
```

**Run tests**:
```bash
npm test
# or
node --test skill.test.js
```

### Rust Skills

```rust
// src/lib.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_valid_input() {
        let params = ToolParameters {
            param1: "value1".to_string(),
            param2: Some("value2".to_string()),
        };

        let result = execute_tool("my-tool", params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "success");
    }

    #[test]
    fn test_execute_invalid_input() {
        let params = ToolParameters {
            param1: "".to_string(),  // Invalid
            param2: None,
        };

        let result = execute_tool("my-tool", params);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_async_execution() {
        let result = execute_async_tool("api-call").await;
        assert!(result.is_ok());
    }
}
```

**Run tests**:
```bash
cargo test
```

## Integration Testing

### Test Script

```bash
#!/bin/bash
# test-skill.sh
set -e

echo "Testing skill installation..."
skill install ./my-skill

echo "Testing tool execution..."
result=$(skill run my-skill my-tool --input "test")

if [[ "$result" == *"success"* ]]; then
  echo "✓ Test passed"
  exit 0
else
  echo "✗ Test failed"
  exit 1
fi
```

### Test Manifest

```toml
# test-manifest.toml
version = "1"

[skills.test-skill]
source = "./my-skill"
runtime = "wasm"

[skills.test-skill.instances.test]
config.base_url = "http://localhost:8000"
env.LOG_LEVEL = "debug"
capabilities.network_access = true
```

**Run integration test**:
```bash
skill --manifest test-manifest.toml run test-skill:test my-tool
```

### Automated Integration Tests

```bash
#!/bin/bash
# integration-tests.sh
set -e

# Setup
export MANIFEST=test-manifest.toml
skill --manifest $MANIFEST validate

# Test each tool
echo "Testing tool1..."
skill --manifest $MANIFEST run test-skill tool1 --param1 value1

echo "Testing tool2..."
skill --manifest $MANIFEST run test-skill tool2 --param2 value2

# Test error handling
echo "Testing error cases..."
if skill --manifest $MANIFEST run test-skill invalid-tool 2>/dev/null; then
  echo "✗ Should have failed"
  exit 1
fi

echo "✓ All integration tests passed"
```

## End-to-End Testing

### Claude Code Testing

**Test with actual AI agent**:

```bash
# 1. Generate Claude Agent Skills
skill claude-bridge generate --skill my-skill

# 2. Start Claude Code
claude

# 3. Test with prompts
> Use my-skill to process data
> List available tools in my-skill
> Execute my-skill tool with parameters
```

**Validation checklist**:
- [ ] Skill appears in Claude's context
- [ ] Tools are discovered correctly
- [ ] Parameters are validated
- [ ] Execution succeeds
- [ ] Output is formatted properly
- [ ] Errors are handled gracefully

### MCP Testing

```bash
# Start MCP server
skill serve &
SERVER_PID=$!

# Test MCP protocol
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | \
  skill serve

# Test tool execution via MCP
echo '{
  "jsonrpc":"2.0",
  "id":2,
  "method":"tools/call",
  "params":{
    "name":"skill-engine/execute",
    "arguments":{
      "skill_name":"my-skill",
      "tool_name":"my-tool",
      "parameters":{"input":"test"}
    }
  }
}' | skill serve

# Cleanup
kill $SERVER_PID
```

## Docker Skill Testing

### Test Docker Configuration

```bash
# Test docker configuration
cat > test-docker-skill.toml << 'EOF'
version = "1"

[skills.test-docker]
source = "docker:alpine:latest"
runtime = "docker"

[skills.test-docker.docker]
image = "alpine:latest"
entrypoint = "echo"
EOF

# Test execution
skill --manifest test-docker-skill.toml run test-docker "Hello"
```

### Volume Mounting Test

```bash
# Create test file
echo "test data" > /tmp/test-input.txt

# Test volume mounting
skill run docker-skill process \
  --input /input/test-input.txt \
  --output /output/result.txt

# Verify output
cat /tmp/test-output.txt
```

## Mocking and Stubbing

### Mock External APIs

```javascript
// test-helpers.js
export function mockGitHubAPI() {
  return {
    getIssues: jest.fn().mockResolvedValue([
      { id: 1, title: 'Test issue' }
    ]),
    createIssue: jest.fn().mockResolvedValue({
      id: 2,
      title: 'New issue'
    })
  };
}

// skill.test.js
import { mockGitHubAPI } from './test-helpers';

test('should create issue', async () => {
  const api = mockGitHubAPI();

  const result = await execute({
    tool: 'create-issue',
    parameters: { title: 'Bug' }
  });

  expect(api.createIssue).toHaveBeenCalled();
});
```

### Mock Native Commands

```javascript
// For native skills wrapping CLI tools
jest.mock('child_process', () => ({
  execFile: jest.fn((cmd, args, callback) => {
    // Mock kubectl output
    if (cmd === 'kubectl' && args[0] === 'get') {
      callback(null, JSON.stringify({
        items: [{ name: 'pod-1' }]
      }), '');
    }
  })
}));
```

## Test Data

### Fixtures

```
tests/
├── fixtures/
│   ├── valid-input.json
│   ├── invalid-input.json
│   ├── large-input.json
│   └── edge-cases.json
└── skill.test.ts
```

**Load in tests**:
```typescript
import validInput from './fixtures/valid-input.json';

test('should handle valid input', async () => {
  const result = await execute({
    tool: 'process',
    parameters: validInput
  });

  expect(result.status).toBe('success');
});
```

### Test Manifests

```
tests/
├── manifests/
│   ├── minimal.toml
│   ├── full-features.toml
│   └── multi-instance.toml
└── integration.test.sh
```

## Continuous Integration

### GitHub Actions

```yaml
# .github/workflows/test.yml
name: Test Skills
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Skill Engine
        run: |
          curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | sh
          echo "$HOME/.skill/bin" >> $GITHUB_PATH

      - name: Run unit tests
        run: npm test

      - name: Run integration tests
        run: ./tests/integration-tests.sh

      - name: Validate manifest
        run: skill validate

      - name: Test skill execution
        run: |
          skill install ./my-skill
          skill run my-skill my-tool --input test
```

### GitLab CI

```yaml
# .gitlab-ci.yml
test:
  image: ubuntu:22.04
  before_script:
    - curl -fsSL https://install-url.com/install.sh | sh
    - export PATH="$HOME/.skill/bin:$PATH"
  script:
    - npm test
    - ./tests/integration-tests.sh
    - skill validate
```

## Performance Testing

### Measure Execution Time

```bash
#!/bin/bash
# benchmark.sh

echo "Running performance tests..."

for i in {1..100}; do
  start=$(date +%s%N)
  skill run my-skill my-tool --input "test $i"
  end=$(date +%s%N)

  duration=$((($end - $start) / 1000000))  # Convert to ms
  echo "$i,$duration" >> results.csv
done

echo "Average: $(awk -F, '{sum+=$2; count++} END {print sum/count}' results.csv) ms"
```

### Load Testing

```bash
#!/bin/bash
# load-test.sh

# Concurrent executions
for i in {1..10}; do
  skill run my-skill my-tool --input "test $i" &
done

wait
echo "Completed 10 concurrent executions"
```

### Benchmark with hyperfine

```bash
# Install hyperfine
cargo install hyperfine

# Benchmark skill execution
hyperfine 'skill run my-skill my-tool --input test' \
  --warmup 3 \
  --min-runs 10

# Compare runtimes
hyperfine \
  'skill run my-skill:wasm my-tool' \
  'skill run my-skill:docker my-tool' \
  'skill run my-skill:native my-tool'
```

## Coverage

### JavaScript/TypeScript

```bash
# Using vitest
npm test -- --coverage

# Using c8
c8 npm test
```

### Rust

```bash
# Using tarpaulin
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

## Best Practices

### 1. Test All Tool Parameters

```typescript
describe('Tool parameters', () => {
  it('should handle required parameters', async () => {
    const result = await execute({
      tool: 'my-tool',
      parameters: { required_param: 'value' }
    });
    expect(result.status).toBe('success');
  });

  it('should reject missing required parameters', async () => {
    await expect(
      execute({
        tool: 'my-tool',
        parameters: {}
      })
    ).rejects.toThrow('required_param is required');
  });

  it('should use default for optional parameters', async () => {
    const result = await execute({
      tool: 'my-tool',
      parameters: { required_param: 'value' }
    });
    // Optional param should use default
  });
});
```

### 2. Test Error Handling

```typescript
describe('Error handling', () => {
  it('should handle network errors', async () => {
    // Mock network failure
    mockAPI.mockRejectedValue(new Error('Network error'));

    await expect(
      execute({ tool: 'api-call', parameters: {} })
    ).rejects.toThrow('Network error');
  });

  it('should handle timeouts', async () => {
    // Mock slow response
    mockAPI.mockImplementation(() =>
      new Promise(resolve => setTimeout(resolve, 60000))
    );

    await expect(
      execute({ tool: 'slow-operation', parameters: {} })
    ).rejects.toThrow('timeout');
  });
});
```

### 3. Test Edge Cases

```typescript
describe('Edge cases', () => {
  it('should handle empty input', async () => {
    const result = await execute({
      tool: 'process',
      parameters: { input: '' }
    });
    expect(result.output).toBe('');
  });

  it('should handle very large input', async () => {
    const result = await execute({
      tool: 'process',
      parameters: { input: 'a'.repeat(1000000) }
    });
    expect(result.status).toBe('success');
  });

  it('should handle special characters', async () => {
    const result = await execute({
      tool: 'process',
      parameters: { input: '!@#$%^&*()' }
    });
    expect(result.status).toBe('success');
  });
});
```

### 4. Use Test Fixtures

```typescript
import * as fs from 'fs';
import * as path from 'path';

function loadFixture(name: string): any {
  const fixturePath = path.join(__dirname, 'fixtures', `${name}.json`);
  return JSON.parse(fs.readFileSync(fixturePath, 'utf-8'));
}

test('should handle complex input', async () => {
  const input = loadFixture('complex-input');
  const result = await execute({
    tool: 'process',
    parameters: input
  });
  expect(result.status).toBe('success');
});
```

### 5. Clean Up After Tests

```typescript
afterEach(async () => {
  // Clean up test data
  await cleanup();
});

afterAll(async () => {
  // Close connections
  await closeConnections();
});
```

## Troubleshooting Tests

### Tests Fail Intermittently

**Cause**: Race conditions, timing issues

**Solution**:
```typescript
// Add proper async handling
await Promise.all([
  operation1(),
  operation2()
]);

// Add timeouts
await waitFor(() => expect(element).toBeInTheDocument(), {
  timeout: 5000
});
```

### Tests Pass Locally, Fail in CI

**Cause**: Environment differences

**Solution**:
```yaml
# Ensure consistent environment
env:
  NODE_ENV: test
  SKILL_LOG_LEVEL: error
```

### Slow Test Suite

**Cause**: Not using parallelization

**Solution**:
```bash
# Run tests in parallel
npm test -- --parallel

# Rust tests in parallel (default)
cargo test
```

## Project Test Suite

The Skill Engine project has comprehensive tests across all crates.

### Test Directory Structure

```
tests/
├── mcp_integration_tests.sh    # MCP protocol tests (45 tests)
├── run-all-tests.sh            # Full test suite runner
├── security/                   # Security test helpers
├── claude_bridge/              # Claude integration tests
│   ├── test-fresh-install.sh
│   ├── test-skill-generation.sh
│   ├── test-performance.sh
│   └── generate-large-manifest.sh
└── fixtures/                   # Test data

crates/
├── skill-cli/tests/            # CLI integration tests
├── skill-runtime/tests/        # Runtime tests
├── skill-mcp/tests/            # MCP server tests
└── skill-http/tests/           # HTTP server tests (62 tests)
```

### Running the Full Test Suite

```bash
# All workspace tests
cargo test --workspace

# Full test suite with integration tests
./tests/run-all-tests.sh

# MCP protocol integration tests
./tests/mcp_integration_tests.sh
```

### Security Tests

```bash
# Run security test suite (requires --ignored flag)
cargo test --test security_tests --package skill-cli -- --ignored --nocapture

# Error handling tests
cargo test --test error_tests -- --ignored --nocapture
```

### Benchmarks

```bash
# All benchmarks
cargo bench

# Specific benchmark suite
cargo bench --bench claude_bridge_bench

# Output in bencher format for CI
cargo bench -- --output-format bencher
```

### Code Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --workspace --out Html

# Generate XML for CI (Codecov)
cargo tarpaulin --workspace --out Xml
```

View the HTML report at `tarpaulin-report.html`.

### Web UI Tests

```bash
# Backend integration tests (62 tests)
cargo test -p skill-http --tests

# Frontend WASM tests (67 tests, requires Chrome)
cd crates/skill-web
wasm-pack test --headless --chrome
```

See [CI/CD Pipeline](/guides/ci-cd) for details on automated testing workflows.

## Related Documentation

- [CI/CD Pipeline](./ci-cd.md) - Automated testing workflows
- [Skill Development](./developing-skills.md) - Creating skills
- [Environment Variables](./environment.md) - Test configuration
- [Security Model](./advanced/security.md) - Security testing
- [CLI Reference](../api/cli.md) - Test commands

## External Resources

- [Vitest](https://vitest.dev/) - Fast unit testing
- [Jest](https://jestjs.io/) - JavaScript testing framework
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) - Rust coverage
- [hyperfine](https://github.com/sharkdp/hyperfine) - Benchmarking tool
