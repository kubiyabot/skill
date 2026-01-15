# Edge Case Test Scenarios

This document catalogs edge cases and error conditions for Claude Bridge skill generation.
Each scenario includes test status, expected behavior, and validation steps.

## Test Status Legend

- ✅ **Automated**: Covered by integration/unit tests
- 🔧 **Manual**: Requires manual testing
- 📝 **Documented**: Known issue or limitation

---

## 1. Filesystem Edge Cases

### 1.1 Symlinks

#### Output Directory is a Symlink
- **Status**: 🔧 Manual
- **Test**: Create symlink to actual directory, use as `--output`
- **Expected**: Should follow symlink and generate skills in target directory
- **Command**:
  ```bash
  ln -s /tmp/real-skills /tmp/symlink-skills
  skill claude generate --output /tmp/symlink-skills
  ```
- **Validation**: Skills appear in /tmp/real-skills

#### Manifest Path is a Symlink
- **Status**: 🔧 Manual
- **Test**: Symlink `.skill-engine.toml` to another location
- **Expected**: Should read manifest from symlink target
- **Command**:
  ```bash
  ln -s /path/to/real-manifest.toml .skill-engine.toml
  skill claude generate
  ```

#### Generated Skills Contain Symlinks
- **Status**: 📝 Documented
- **Expected**: Symlinks in source skills should be preserved or handled gracefully
- **Note**: Behavior depends on file copy implementation

### 1.2 Special Characters in Paths

#### Spaces in Paths
- **Status**: ✅ Automated (`test_special_characters_in_paths`)
- **Test**: Output directory with spaces
- **Expected**: Should handle correctly with proper quoting
- **Example**: `--output "/tmp/skills with spaces"`

#### Unicode in Paths
- **Status**: 🔧 Manual
- **Test**: Output directory with unicode characters
- **Expected**: Should handle UTF-8 paths correctly
- **Example**: `--output "/tmp/测试/skills"`
- **Platform Note**: Works on Linux/macOS, may fail on Windows with non-ASCII

#### Special Shell Characters
- **Status**: 🔧 Manual
- **Test**: Paths with `$`, `` ` ``, `&`, `;`, etc.
- **Expected**: Should escape or sanitize properly
- **Example**: `--output "/tmp/skill$test"`
- **Security**: Must not allow command injection

### 1.3 Path Traversal

#### Attempt Directory Traversal
- **Status**: ✅ Automated (`test_path_traversal_prevention`)
- **Test**: Use `--output ../../../etc/skills`
- **Expected**: Should reject or sanitize path
- **Security**: Critical - must not write outside intended directory

#### Absolute vs Relative Paths
- **Status**: 🔧 Manual
- **Test**: Mix of absolute and relative paths
- **Expected**: Both should work correctly
- **Examples**:
  - `--output ./skills` (relative)
  - `--output /tmp/skills` (absolute)
  - `--output ~/skills` (home directory)

### 1.4 File Permissions

#### Read-Only Output Directory
- **Status**: ✅ Automated (`test_error_output_dir_not_writable`)
- **Test**: Set output directory to read-only (chmod 444)
- **Expected**: Clear permission denied error
- **Platform**: Unix only

#### No Write Permission on Parent Directory
- **Status**: 🔧 Manual
- **Test**: Output directory doesn't exist, parent is read-only
- **Expected**: Error creating directory

#### Executable Bit on SKILL.md
- **Status**: 📝 Documented
- **Expected**: SKILL.md should not be executable (should be 644 or 444)
- **Validation**: `ls -l skills/*/SKILL.md | grep -v 'rw-r--r--'` should be empty

---

## 2. Concurrency Edge Cases

### 2.1 Multiple Processes

#### Simultaneous Generation to Same Directory
- **Status**: ✅ Automated (`test_concurrent_generation_safety`)
- **Test**: Launch 3+ processes generating to same output
- **Expected**: No file corruption, all processes complete
- **Validation**: SKILL.md files remain valid after concurrent writes

#### One Process Reading While Another Writes
- **Status**: 🔧 Manual
- **Test**: Start generation, simultaneously read SKILL.md
- **Expected**: Either old content or new content, never partial/corrupt
- **Tools**: Use `inotifywait` (Linux) or `fswatch` (macOS)

#### File Locking Behavior
- **Status**: 📝 Documented
- **Current**: No explicit file locking
- **Risk**: Race conditions in concurrent writes
- **Mitigation**: Use --force carefully, avoid concurrent generation

### 2.2 Interrupted Generation

#### Kill Process Mid-Generation (Ctrl+C)
- **Status**: 🔧 Manual
- **Test**: Start generation, press Ctrl+C after 1-2 seconds
- **Expected**: Partial files left behind
- **Cleanup**: Run with `--force` to overwrite partial files
- **Command**:
  ```bash
  skill claude generate --output /tmp/test &
  PID=$!
  sleep 1
  kill $PID
  # Check /tmp/test for partial files
  ```

#### Retry After Interruption
- **Status**: 🔧 Manual
- **Test**: Interrupt generation, then retry
- **Expected**: Should complete successfully with --force
- **Without --force**: Should skip existing files

#### SIGTERM vs SIGKILL
- **Status**: 🔧 Manual
- **SIGTERM**: Graceful shutdown possible
- **SIGKILL**: Immediate termination, may leave corrupt files
- **Test**: Compare behavior with `kill -TERM` vs `kill -KILL`

---

## 3. Resource Exhaustion

### 3.1 Disk Space

#### Generate When Disk Nearly Full
- **Status**: 🔧 Manual
- **Test**: Fill disk to 95%, attempt generation
- **Expected**: Clear "no space left" error before corruption
- **Setup**:
  ```bash
  # Create 95% full filesystem
  dd if=/dev/zero of=/tmp/fill bs=1M count=$(df /tmp | awk 'NR==2{print int($3*0.95/1024)}')
  ```
- **Cleanup**: `rm /tmp/fill`

#### Partial Write on Disk Full
- **Status**: 🔧 Manual
- **Test**: Start generation with enough space, fill disk mid-generation
- **Expected**: Error message, no corrupt files left
- **Validation**: Verify SKILL.md either absent or complete

### 3.2 Memory

#### Generate 1000+ Skills
- **Status**: 🔧 Manual (requires large manifest)
- **Test**: Manifest with 1000 skills
- **Expected**: Memory usage should not grow unbounded
- **Monitor**:
  ```bash
  skill claude generate --output /tmp/many &
  PID=$!
  while kill -0 $PID 2>/dev/null; do
    ps -o rss= -p $PID
    sleep 1
  done
  ```
- **Target**: Memory usage should stabilize (streaming generation)

#### Very Large Skill Descriptions
- **Status**: ✅ Automated (`test_extreme_values`)
- **Test**: 1024-char description (max length)
- **Expected**: Should handle without memory issues

### 3.3 File Descriptors

#### Generate with Low ulimit -n
- **Status**: 🔧 Manual
- **Test**: Set `ulimit -n 100`, generate many skills
- **Expected**: Should not hit "too many open files" error
- **Setup**:
  ```bash
  ulimit -n 100
  skill claude generate --output /tmp/test
  ```
- **Validation**: Process opens and closes files properly

---

## 4. Data Edge Cases

### 4.1 Extreme Values

#### Skill Name Exactly 64 Characters
- **Status**: ✅ Automated (`test_extreme_values`)
- **Test**: Skill name with 64 characters (max)
- **Expected**: Should generate successfully
- **Example**: `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`

#### Skill Name 65 Characters (Over Limit)
- **Status**: 🔧 Manual
- **Test**: Skill name exceeding 64 chars
- **Expected**: Validation error with clear message
- **Example**: `aaaaa...` (65 chars)

#### Description Exactly 1024 Characters
- **Status**: ✅ Automated (`test_extreme_values`)
- **Test**: Description at max length
- **Expected**: Should generate successfully

#### Description Over 1024 Characters
- **Status**: 🔧 Manual
- **Test**: Description exceeding 1024 chars
- **Expected**: Validation error or truncation with warning

#### Tool with 100+ Parameters
- **Status**: 🔧 Manual
- **Test**: Skill tool with 100 parameters
- **Expected**: Should generate correctly, may impact Claude context window
- **Consideration**: Recommend tool redesign if >20 parameters

#### Empty Skill (No Tools)
- **Status**: 🔧 Manual
- **Test**: Skill with no tools defined
- **Expected**: Should generate SKILL.md with empty tools section
- **Validation**: TOOLS.md should indicate "No tools available"

### 4.2 Unicode and Encoding

#### Unicode in Skill Descriptions
- **Status**: ✅ Automated (`test_unicode_in_descriptions`)
- **Test**: Description with Chinese, Arabic, emoji
- **Expected**: Unicode preserved correctly
- **Example**: `"Kubernetes 集群管理 مجموعة 🚀"`

#### Unicode in Skill Names
- **Status**: 🔧 Manual
- **Test**: Skill name with unicode (e.g., `kube-集群`)
- **Expected**: Should reject (only lowercase ASCII + hyphens allowed)
- **Validation**: Spec validator should catch this

#### Emoji in Tool Names
- **Status**: 🔧 Manual
- **Test**: Tool name with emoji (e.g., `deploy-🚀`)
- **Expected**: Should reject per naming convention

#### RTL (Right-to-Left) Text
- **Status**: 🔧 Manual
- **Test**: Arabic or Hebrew in descriptions
- **Expected**: Should preserve correctly
- **Example**: `"أداة Kubernetes للإدارة"`

### 4.3 Invalid UTF-8

#### Manifest with Invalid UTF-8
- **Status**: 🔧 Manual
- **Test**: Binary data or invalid UTF-8 in TOML
- **Expected**: Parse error with clear message
- **Setup**:
  ```bash
  printf '\xff\xfe[skills.test]\n' > .skill-engine.toml
  ```

#### Skill Name with Null Bytes
- **Status**: 🔧 Manual
- **Test**: Skill name containing `\0`
- **Expected**: Should reject or sanitize
- **Security**: Must not allow null byte injection

---

## 5. Manifest Edge Cases

### 5.1 TOML Format Issues

#### Mixed Inline and Block Tables
- **Status**: 🔧 Manual
- **Test**: Combination of `[skills.x]` and inline tables
- **Expected**: Should parse correctly per TOML spec

#### Duplicate Skill Names
- **Status**: 🔧 Manual
- **Test**: Same skill defined twice in manifest
- **Expected**: TOML parser error or last definition wins

#### Missing Sections
- **Status**: ✅ Automated (`test_error_no_skills_in_manifest`)
- **Test**: Manifest with no `[skills]` section
- **Expected**: Clear error about no skills found

### 5.2 Skill Configuration Issues

#### Missing Required Fields
- **Status**: ✅ Automated (`test_error_missing_required_manifest_fields`)
- **Test**: Skill without `description`
- **Expected**: Validation error listing missing fields

#### Invalid Runtime Values
- **Status**: 🔧 Manual
- **Test**: `runtime = "invalid"`
- **Expected**: Error listing valid runtimes (wasm, native)

#### Conflicting Skill Configurations
- **Status**: 🔧 Manual
- **Test**: Two skills with same name but different sources
- **Expected**: Error or warning about conflict

---

## 6. Output Edge Cases

### 6.1 Existing Files

#### Generate Over Existing Files (Without --force)
- **Status**: ✅ Automated (integration tests)
- **Test**: Generate twice without --force
- **Expected**: Second run skips existing files

#### Generate Over Existing Files (With --force)
- **Status**: ✅ Automated (`test_generate_force_overwrite`)
- **Test**: Modify SKILL.md, regenerate with --force
- **Expected**: Files overwritten with new content

#### Partial Generation State
- **Status**: 🔧 Manual
- **Test**: Some skills exist, some don't
- **Expected**: Generate only missing skills (unless --force)

### 6.2 File Name Collisions

#### Case-Insensitive Filesystems (macOS/Windows)
- **Status**: 🔧 Manual
- **Test**: Skills named "Test" and "test"
- **Expected**: Error on case-insensitive systems
- **Platform**: macOS, Windows (case-insensitive by default)

#### Reserved File Names (Windows)
- **Status**: 🔧 Manual
- **Test**: Skill named "CON", "PRN", "AUX" (Windows reserved)
- **Expected**: Error or sanitization on Windows

---

## 7. CLI Edge Cases

### 7.1 Argument Parsing

#### Empty --skill Argument
- **Status**: 🔧 Manual
- **Test**: `skill claude generate --skill ""`
- **Expected**: Error about invalid skill name

#### Duplicate Flags
- **Status**: 🔧 Manual
- **Test**: `skill claude generate --force --force`
- **Expected**: Last flag wins or error

#### Missing --output Value
- **Status**: 🔧 Manual
- **Test**: `skill claude generate --output`
- **Expected**: Argument parser error

### 7.2 Environment Variables

#### Manifest Path from Environment
- **Status**: 🔧 Manual
- **Test**: Set `SKILL_MANIFEST=/path/to/manifest.toml`
- **Expected**: Should use environment variable

#### Conflicting Environment and CLI Args
- **Status**: 🔧 Manual
- **Test**: Both env var and `--manifest` flag
- **Expected**: CLI flag takes precedence

---

## 8. Error Recovery

### 8.1 Cleanup on Failure

#### Partial Files After Error
- **Status**: 🔧 Manual
- **Test**: Cause error mid-generation (disk full)
- **Expected**: No corrupt or partial files left
- **Cleanup**: Either complete files or nothing

#### Transaction-Like Behavior
- **Status**: 📝 Documented
- **Current**: No atomic transaction support
- **Future**: Consider atomic directory swap pattern

### 8.2 Retry Behavior

#### Retry After Transient Error
- **Status**: 🔧 Manual
- **Test**: Fix issue (e.g., add disk space), retry
- **Expected**: Should complete successfully

#### Idempotent Generation
- **Status**: ✅ Automated
- **Test**: Generate same skills multiple times
- **Expected**: Same output each time (deterministic)

---

## Testing Procedures

### Automated Test Execution

```bash
# Run all error tests
cargo test --test error_tests -- --ignored

# Run specific test category
cargo test test_error_missing_manifest -- --ignored
cargo test test_concurrent_generation -- --ignored
```

### Manual Test Template

For each manual test:

1. **Setup**: Describe environment preparation
2. **Execute**: Run command with specific inputs
3. **Observe**: Note actual behavior
4. **Validate**: Check expected outcome
5. **Cleanup**: Restore environment
6. **Document**: Record results and any issues

### Stress Testing

```bash
# Concurrent generation stress test
for i in {1..10}; do
  skill claude generate --output /tmp/stress-$i --force &
done
wait
# Verify all outputs are valid
```

### Memory Profiling

```bash
# Use valgrind or heaptrack
valgrind --leak-check=full skill claude generate --output /tmp/test

# macOS: use leaks
leaks --atExit -- skill claude generate --output /tmp/test
```

---

## Checklist for Pre-Release Testing

- [ ] All automated tests pass
- [ ] Manual filesystem edge cases tested
- [ ] Concurrency stress test completed (10+ iterations)
- [ ] Memory profiling shows no leaks
- [ ] Error messages reviewed for quality
- [ ] Cross-platform testing (Linux, macOS, Windows)
- [ ] Unicode/emoji handling verified
- [ ] Permission errors tested on Unix
- [ ] Path traversal security verified
- [ ] Disk full scenario tested
- [ ] Extreme value tests completed
- [ ] Documentation updated with findings

---

## Known Limitations

1. **No File Locking**: Concurrent writes may cause corruption
2. **No Atomic Writes**: Process interruption may leave partial files
3. **Platform-Specific**: Some tests only work on Unix/Linux
4. **No Transaction Support**: Failures may leave inconsistent state

## Future Improvements

1. **File Locking**: Implement per-file or directory-level locking
2. **Atomic Operations**: Write to temp directory, then atomic rename
3. **Progress Indication**: Show progress for long-running operations
4. **Dry-Run Validation**: Validate manifest without generation
5. **Checksum Verification**: Detect partial/corrupt files

---

**Document Version**: 1.0
**Last Updated**: 2026-01-04
**Maintainer**: Claude Bridge Testing Team
