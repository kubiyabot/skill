# Performance Profiling Guide

**Version**: 1.0
**Last Updated**: 2026-01-04
**Purpose**: Guide for profiling and optimizing Claude Bridge performance

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Criterion Benchmarks](#criterion-benchmarks)
3. [Memory Profiling](#memory-profiling)
4. [CPU Profiling with Flamegraph](#cpu-profiling-with-flamegraph)
5. [Performance Test Suite](#performance-test-suite)
6. [Scalability Testing](#scalability-testing)
7. [Continuous Performance Monitoring](#continuous-performance-monitoring)
8. [Optimization Tips](#optimization-tips)

---

## Quick Start

```bash
# Run all performance tests
./tests/claude_bridge/test-performance.sh

# Run Criterion benchmarks
cargo bench --bench claude_bridge_bench

# Generate large test manifests
./tests/claude_bridge/generate-large-manifest.sh

# Profile with flamegraph (requires root on Linux)
cargo install flamegraph
cargo flamegraph --bin skill -- claude generate --force
```

---

## Criterion Benchmarks

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --bench claude_bridge_bench

# Run specific benchmark group
cargo bench --bench claude_bridge_bench manifest_parsing
cargo bench --bench claude_bridge_bench skill_generation
cargo bench --bench claude_bridge_bench yaml_frontmatter

# View HTML reports
open target/criterion/report/index.html  # macOS
xdg-open target/criterion/report/index.html  # Linux
```

### Benchmark Categories

1. **Manifest Parsing** - TOML parsing performance
2. **Skill Generation** - End-to-end generation (dry-run)
3. **YAML Frontmatter** - YAML serialization speed
4. **Script Generation** - Shell script template rendering
5. **Markdown Rendering** - TOOLS.md generation
6. **File Operations** - I/O performance (read/write)
7. **Validation** - Manifest validation logic
8. **End-to-End** - Full generation pipeline (in-memory)

### Performance Targets

| Operation | Target | Measured By |
|-----------|--------|-------------|
| Parse 10-skill manifest | <100ms | Criterion |
| Generate 1 skill | <5s | Bash suite |
| Generate 10 skills | <30s | Bash suite |
| Generate 50 skills | <2min | Bash suite |
| YAML serialization | <1ms | Criterion |
| Script generation (8 tools) | <10ms | Criterion |

### Interpreting Results

```
manifest_parsing/parse/10skills_20tools
                        time:   [892.45 µs 895.12 µs 898.23 µs]
                        change: [-5.2341% -4.1234% -2.9876%] (p = 0.00 < 0.05)
                        Performance has improved.
```

- **time**: Average execution time with confidence interval
- **change**: Performance delta from previous run
- **p-value**: Statistical significance (< 0.05 = significant)

---

## Memory Profiling

### Basic Memory Monitoring

The performance test suite (`test-performance.sh`) includes memory profiling:

```bash
./tests/claude_bridge/test-performance.sh
```

This monitors:
- Peak memory usage during generation
- Target: < 500MB (512000KB)
- Polls every 100ms for max resident set size

### Platform-Specific Tools

#### macOS

```bash
# Monitor specific process
sudo fs_usage -w -f filesys skill | head -100

# Heap profiling with Instruments
instruments -t "Allocations" -D trace.trace \
    cargo run --release --bin skill -- claude generate --force

# View with Instruments.app
open trace.trace
```

#### Linux

```bash
# Use Valgrind for detailed analysis
valgrind --tool=massif --massif-out-file=massif.out \
    cargo run --release --bin skill -- claude generate --force

# View with ms_print
ms_print massif.out

# Use heaptrack for heap profiling
heaptrack cargo run --release --bin skill -- claude generate --force
heaptrack_gui heaptrack.skill.*.gz
```

### Memory Leak Detection

```bash
# Valgrind memcheck (Linux only)
valgrind --leak-check=full --show-leak-kinds=all \
    cargo run --release --bin skill -- claude generate --force

# macOS leaks
leaks --atExit -- cargo run --release --bin skill -- claude generate --force
```

### Target Memory Budget

- **Single skill generation**: < 100MB
- **10 skills**: < 250MB
- **50 skills**: < 500MB
- **100 skills**: < 1GB

---

## CPU Profiling with Flamegraph

### Installation

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Linux: Install perf (if not already installed)
sudo apt-get install linux-tools-common linux-tools-generic

# macOS: Uses DTrace (built-in, requires root)
```

### Generate Flamegraph

```bash
# Profile skill generation
sudo cargo flamegraph --bin skill -- claude generate \
    --manifest /tmp/large-manifest-50.toml \
    --output /tmp/flamegraph-test \
    --force

# Opens flamegraph.svg in browser
```

### Interpreting Flamegraphs

- **Width**: Percentage of total CPU time
- **Height**: Call stack depth
- **Color**: Random (for differentiation)

**What to look for**:
- Wide plateaus = hotspots (optimize these)
- Deep stacks = excessive nesting
- Repeated patterns = potential for caching

### Common Hotspots

1. **TOML Parsing**: Look for `toml::from_str` calls
2. **File I/O**: `std::fs::write`, `std::fs::read_to_string`
3. **String Allocation**: `String::from`, `format!` macros
4. **Regex Matching**: `regex::Regex::is_match` (if used)

### Profiling MCP Server

```bash
# Start MCP server with profiling
sudo cargo flamegraph --bin skill -- serve &

# Run MCP test suite to generate load
cargo test --test mcp_claude_bridge_tests -- --ignored

# Kill server
kill %1

# View flamegraph.svg
```

---

## Performance Test Suite

### Running Tests

```bash
# Run full performance suite
./tests/claude_bridge/test-performance.sh

# Generate test data first
./tests/claude_bridge/generate-large-manifest.sh
```

### Test Categories

1. **Single Skill Generation** (kubernetes)
   - Target: < 5s
   - Tests basic generation speed

2. **10 Skills Generation**
   - Target: < 30s
   - Tests moderate scale

3. **50 Skills Generation**
   - Target: < 120s
   - Tests large-scale performance

4. **Memory Usage**
   - Target: < 500MB
   - Monitors peak memory during generation

5. **Skill Discovery**
   - Target: < 1s
   - Tests `skill list` latency

6. **Tool Execution**
   - Target: < 2s
   - Tests `skill run` overhead

7. **Concurrent Generation**
   - Tests 3 parallel generations
   - Verifies no corruption or failures

### Expected Output

```
========================================
  Claude Bridge Performance Tests
========================================

[1] Testing: Generate 1 skill (kubernetes)
  Duration: 3247ms (target: 5000ms)
  ✓ PASSED

[2] Testing: Generate 10 skills
  Duration: 24891ms (target: 30000ms)
  ✓ PASSED

[3] Testing: Memory usage during generation
  Peak memory: 387642KB (target: <512000KB / 500MB)
  ✓ PASSED - Memory usage within limits

========================================
  Performance Test Summary
========================================
  Total Tests:  7
  Passed:       7
  Failed:       0

✓ All performance tests passed!
```

---

## Scalability Testing

### Generating Test Data

```bash
# Generate manifests with different scales
./tests/claude_bridge/generate-large-manifest.sh

# Manually test with specific size
skill claude generate \
    --manifest /tmp/large-manifest-100.toml \
    --output /tmp/scale-test \
    --force
```

### Measuring Scalability

```bash
# Time generation at different scales
for size in 10 50 100; do
    echo "Testing $size skills..."
    time skill claude generate \
        --manifest /tmp/large-manifest-$size.toml \
        --output /tmp/scale-$size \
        --force
done
```

### Expected Scaling

| Skills | Time | Memory | Scaling |
|--------|------|--------|---------|
| 1 | ~3s | ~80MB | Baseline |
| 10 | ~25s | ~250MB | ~2.5s per skill |
| 50 | ~110s | ~450MB | ~2.2s per skill |
| 100 | ~200s | ~800MB | ~2.0s per skill |

**Target**: Near-linear scaling (O(n)) with minimal overhead per skill.

---

## Continuous Performance Monitoring

### CI Integration

Performance tests run automatically in CI via `.github/workflows/performance-tests.yml`:

```bash
# Locally simulate CI run
cargo bench --bench claude_bridge_bench --no-fail-fast
./tests/claude_bridge/test-performance.sh
```

### Performance Regression Detection

Criterion automatically detects regressions:
- Warns if performance degrades > 5%
- Fails CI if degradation > 10%

### Viewing CI Reports

1. Go to GitHub Actions for your PR
2. Click "Performance Tests" workflow
3. Download "criterion-reports" artifact
4. Extract and open `report/index.html`

---

## Optimization Tips

### 1. Reduce Allocations

```rust
// SLOW: Creates many allocations
let result = format!("{}:{}", skill, tool);

// FAST: Reuse buffer
let mut result = String::with_capacity(skill.len() + tool.len() + 1);
result.push_str(skill);
result.push(':');
result.push_str(tool);
```

### 2. Parallelize Independent Operations

```rust
use rayon::prelude::*;

// SLOW: Sequential generation
for skill in skills {
    generate_skill(skill)?;
}

// FAST: Parallel generation
skills.par_iter()
    .try_for_each(|skill| generate_skill(skill))?;
```

### 3. Cache Expensive Operations

```rust
use std::sync::OnceLock;

static TEMPLATE: OnceLock<String> = OnceLock::new();

fn get_template() -> &'static str {
    TEMPLATE.get_or_init(|| {
        std::fs::read_to_string("template.md").unwrap()
    })
}
```

### 4. Use Buffered I/O

```rust
use std::io::{BufWriter, Write};

// SLOW: Unbuffered writes
for line in lines {
    file.write_all(line.as_bytes())?;
}

// FAST: Buffered writes
let mut writer = BufWriter::new(file);
for line in lines {
    writer.write_all(line.as_bytes())?;
}
writer.flush()?;
```

### 5. Optimize Manifest Parsing

```rust
// Use streaming parser for large files
use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    #[serde(borrow)]
    skills: Vec<Skill<'static>>,
}
```

### 6. Profile Before Optimizing

**Always profile first!** Don't optimize blindly:

```bash
# Identify hotspots
cargo flamegraph --bin skill -- claude generate --force

# Benchmark before and after
cargo bench --bench claude_bridge_bench manifest_parsing
# ... make changes ...
cargo bench --bench claude_bridge_bench manifest_parsing
```

---

## Performance Checklist

Before releasing:

- [ ] All Criterion benchmarks pass
- [ ] Performance test suite passes (`test-performance.sh`)
- [ ] No performance regression > 5% from previous version
- [ ] Memory usage < 500MB for 50 skills
- [ ] Flamegraph shows no obvious hotspots
- [ ] Linear scaling verified up to 100 skills
- [ ] Concurrent generation stress test passes
- [ ] CI performance tests green

---

## Troubleshooting

### Performance Tests Fail

1. **Check system load**: Close other applications
2. **Run on release build**: `cargo build --release`
3. **Increase targets**: Edit `test-performance.sh` if hardware is slower
4. **Profile bottlenecks**: Use flamegraph to identify issues

### Benchmarks Are Noisy

1. **Reduce background processes**: Close browsers, IDEs
2. **Disable CPU throttling**: Set performance mode
3. **Run multiple times**: Criterion will stabilize over iterations
4. **Use `cargo bench --profile-time=60`**: Longer profiling

### Memory Profiling Fails

1. **Linux**: Install `valgrind` or `heaptrack`
2. **macOS**: Use `leaks` or Instruments.app
3. **Both**: Use performance test suite's built-in monitoring

---

**Maintained By**: Claude Bridge Performance Team
**Review Frequency**: After significant changes
**Next Review**: 2026-04-04
