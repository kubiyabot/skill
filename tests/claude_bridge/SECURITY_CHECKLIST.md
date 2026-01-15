# Claude Bridge Security Checklist

**Version**: 1.0
**Last Updated**: 2026-01-04
**Purpose**: Security validation checklist for Claude Bridge skill generation

---

## Overview

This checklist provides a comprehensive security validation framework for the Claude Bridge feature. Use it during development, code review, and before each release to ensure all security controls are in place.

## Table of Contents

1. [Credential Security](#1-credential-security)
2. [Command Injection Prevention](#2-command-injection-prevention)
3. [Path Traversal Prevention](#3-path-traversal-prevention)
4. [XSS and Content Injection](#4-xss-and-content-injection)
5. [Script Security](#5-script-security)
6. [Filesystem Security](#6-filesystem-security)
7. [Dependency Security](#7-dependency-security)
8. [CI/CD Security](#8-cicd-security)
9. [Pre-Release Checklist](#9-pre-release-checklist)

---

## 1. Credential Security

### 1.1 API Key Protection

**Requirement**: API keys and secrets must never appear in logs, outputs, or generated files.

**Validation Steps**:

- [ ] API keys are not logged to stdout/stderr
- [ ] API keys are not written to generated SKILL.md files
- [ ] API keys are not included in TOOLS.md files
- [ ] API keys are not embedded in generated scripts
- [ ] Environment variables containing secrets are not echoed
- [ ] Error messages do not include secret values

**Tests**:
```bash
# Run API key leak tests
cargo test test_security_no_api_keys_in_logs -- --ignored --nocapture
```

**Code Locations**:
- `crates/skill-cli/tests/security_tests.rs:48-80` - API key leak test
- `crates/skill-cli/tests/security_tests.rs:82-145` - Hardcoded credential scanner

---

### 1.2 Hardcoded Credentials Detection

**Requirement**: No hardcoded credentials in source code or generated files.

**Patterns to Detect**:
- `api_key = "sk-..."`
- `password = "..."`
- `secret = "..."`
- `token = "..."`
- Anthropic API keys: `sk-ant-...`
- AWS keys: `AKIA...`

**Validation Steps**:

- [ ] Run credential scanner on generated files
- [ ] Check for base64-encoded credentials
- [ ] Verify no credentials in git history
- [ ] Scan for credential patterns in comments

**Tests**:
```bash
# Run hardcoded credential detection
cargo test test_security_no_hardcoded_credentials -- --ignored --nocapture

# Run gitleaks scan
gitleaks detect --source . --no-git
```

---

## 2. Command Injection Prevention

### 2.1 Shell Metacharacter Sanitization

**Requirement**: User input must not allow command injection via shell metacharacters.

**Dangerous Characters**:
- `;` - Command separator
- `|` - Pipe operator
- `&` - Background execution
- `$()` - Command substitution
- `` ` `` - Backtick substitution
- `>`, `<` - Redirection
- `\n` - Newline injection

**Validation Steps**:

- [ ] Skill names with semicolons are rejected or sanitized
- [ ] Skill names with backticks are rejected or sanitized
- [ ] Skill names with `$()` are rejected or sanitized
- [ ] Generated scripts do not execute user-controlled commands
- [ ] All user input is properly quoted in shell scripts

**Tests**:
```bash
# Run command injection tests
cargo test test_security_script_no_command_injection -- --ignored --nocapture
```

**Code Locations**:
- `crates/skill-cli/tests/security_tests.rs:147-235` - Command injection tests

---

### 2.2 Script Generation Safety

**Requirement**: Generated scripts must not contain dangerous commands.

**Validation Steps**:

- [ ] No `rm -rf` in generated scripts
- [ ] No `eval` of user input
- [ ] No direct execution of environment variables
- [ ] All paths are validated before use
- [ ] Commands use absolute paths where possible

**Code Review Checklist**:
```rust
// BAD - Dangerous command injection
let cmd = format!("rm -rf {}", user_input);

// GOOD - Validated and quoted
let path = PathBuf::from(user_input);
if !path.starts_with(&safe_root) {
    return Err("Invalid path");
}
let cmd = format!("rm -rf {:?}", path);
```

---

## 3. Path Traversal Prevention

### 3.1 Directory Traversal Protection

**Requirement**: Users must not be able to write files outside intended directories.

**Attack Vectors**:
- `../../../etc/passwd` - Relative path traversal
- `/etc/passwd` - Absolute path injection
- Symlink attacks to redirect writes

**Validation Steps**:

- [ ] `../` patterns are rejected or sanitized
- [ ] Absolute paths are validated against allowed roots
- [ ] Symlinks are resolved and validated
- [ ] Output directory is canonicalized before use
- [ ] All file writes check final resolved path

**Tests**:
```bash
# Run path traversal tests
cargo test test_security_path_traversal -- --ignored --nocapture
cargo test test_security_symlink_attack -- --ignored --nocapture
```

**Code Locations**:
- `crates/skill-cli/tests/security_tests.rs:237-331` - Path traversal tests
- `crates/skill-cli/tests/error_tests.rs:388-426` - Path security tests

---

### 3.2 Symlink Attack Prevention

**Requirement**: Symlinks must not allow writing to unintended locations.

**Validation Steps**:

- [ ] Output directory symlinks are resolved before use
- [ ] Generated file paths are validated after resolution
- [ ] Race conditions (TOCTOU) are prevented
- [ ] Atomic operations are used where possible

**Code Pattern**:
```rust
// Resolve symlinks and validate
let output_dir = output_dir.canonicalize()?;
let file_path = output_dir.join(skill_name).join("SKILL.md");
let canonical_path = file_path.canonicalize()?;

// Ensure still within output_dir after resolution
if !canonical_path.starts_with(&output_dir) {
    return Err("Path traversal detected");
}
```

---

## 4. XSS and Content Injection

### 4.1 HTML/Script Tag Sanitization

**Requirement**: Generated markdown must not contain unescaped HTML/JavaScript.

**Attack Vectors**:
- `<script>alert('XSS')</script>`
- `<img src=x onerror=alert('XSS')>`
- `<iframe src="javascript:alert('XSS')">`
- `<a href="javascript:alert('XSS')">Click</a>`

**Validation Steps**:

- [ ] `<script>` tags are escaped or rejected
- [ ] Event handlers (`onclick`, `onerror`) are escaped
- [ ] JavaScript URLs are rejected
- [ ] HTML entities are properly encoded
- [ ] Markdown is rendered safely without HTML interpretation

**Tests**:
```bash
# Run XSS protection tests
cargo test test_security_xss -- --ignored --nocapture
```

**Code Locations**:
- `crates/skill-cli/tests/security_tests.rs:333-402` - XSS protection tests

---

### 4.2 Template Injection Prevention

**Requirement**: Template variables must not execute arbitrary code.

**Validation Steps**:

- [ ] Template variables are escaped before rendering
- [ ] User input is not used in template expressions
- [ ] Template engine is configured for safe mode
- [ ] No `eval()` or similar in templates

---

## 5. Script Security

### 5.1 Script Permissions

**Requirement**: Generated scripts must have appropriate Unix permissions.

**Expected Permissions**:
- Shell scripts (`.sh`): `755` or `750` (rwxr-xr-x)
- Configuration files: `644` or `640` (rw-r--r--)
- Private keys/secrets: `600` (rw-------)
- **Never**: `777` (world-writable)

**Validation Steps**:

- [ ] Scripts are not world-writable (`o+w` not set)
- [ ] Scripts are executable by owner (`u+x` set)
- [ ] Scripts are readable by group if needed
- [ ] No setuid/setgid bits unless explicitly required

**Tests**:
```bash
# Run script permission tests (Unix only)
cargo test test_security_script_permissions -- --ignored --nocapture

# Manual check
find .claude/skills -name "*.sh" -ls | awk '{print $3, $11}'
```

**Code Locations**:
- `crates/skill-cli/tests/security_tests.rs:404-453` - Script permission tests

---

### 5.2 Privilege Escalation Prevention

**Requirement**: Scripts must not contain privilege escalation commands.

**Prohibited Commands**:
- `sudo`
- `su`
- `pkexec`
- `doas`
- `setuid`
- `setcap`

**Validation Steps**:

- [ ] No `sudo` in generated scripts
- [ ] No `su` for privilege escalation
- [ ] No setuid binaries created
- [ ] Scripts run with least privilege

**Tests**:
```bash
# Run privilege escalation tests
cargo test test_security_no_privilege_escalation -- --ignored --nocapture
```

**Code Locations**:
- `crates/skill-cli/tests/security_tests.rs:455-507` - Privilege escalation tests

---

## 6. Filesystem Security

### 6.1 Input Validation

**Requirement**: All filesystem operations must validate input.

**Validation Steps**:

- [ ] File paths are validated before use
- [ ] File names match expected patterns
- [ ] Extensions are whitelisted
- [ ] Maximum path lengths are enforced
- [ ] Null bytes in paths are rejected

**Tests**:
```bash
# Run filesystem security tests
cargo test test_security_null_byte_injection -- --ignored --nocapture
```

**Code Locations**:
- `crates/skill-cli/tests/security_tests.rs:696-727` - Null byte injection test

---

### 6.2 File Operation Safety

**Requirement**: File operations must be safe from race conditions.

**Validation Steps**:

- [ ] Use atomic operations where possible
- [ ] Check file existence before overwrite (unless `--force`)
- [ ] Lock files during critical operations
- [ ] Handle partial writes gracefully
- [ ] Clean up temporary files on failure

---

## 7. Dependency Security

### 7.1 Dependency Auditing

**Requirement**: All dependencies must be regularly audited for vulnerabilities.

**Validation Steps**:

- [ ] Run `cargo audit` weekly
- [ ] Review high/critical severity advisories
- [ ] Update vulnerable dependencies promptly
- [ ] Use `cargo deny` for policy enforcement
- [ ] Pin versions for reproducible builds

**Commands**:
```bash
# Audit dependencies for vulnerabilities
cargo audit

# Check security policies
cargo deny check advisories
cargo deny check licenses
```

---

### 7.2 License Compliance

**Requirement**: Dependencies must not have incompatible licenses.

**Prohibited Licenses**:
- GPL-3.0 (unless project is GPL)
- AGPL-3.0

**Validation Steps**:

- [ ] Review all dependency licenses
- [ ] Use `cargo deny` to enforce license policy
- [ ] Document license decisions

**Commands**:
```bash
# Check licenses
cargo deny check licenses
```

---

## 8. CI/CD Security

### 8.1 Automated Security Testing

**Requirement**: Security tests must run on every commit/PR.

**CI/CD Jobs**:

- [ ] Security test suite (`cargo test --test security_tests`)
- [ ] Error handling tests (`cargo test --test error_tests`)
- [ ] Cargo audit for vulnerabilities
- [ ] Cargo deny for policy enforcement
- [ ] Gitleaks for secret scanning
- [ ] CodeQL/SAST analysis
- [ ] Dependency review on PRs
- [ ] File permissions check

**GitHub Actions Workflow**: `.github/workflows/security-tests.yml`

---

### 8.2 Secret Management in CI

**Requirement**: Secrets must never be exposed in CI logs.

**Validation Steps**:

- [ ] Secrets are stored in GitHub Secrets
- [ ] Secrets are not echoed in scripts
- [ ] Masked in GitHub Actions automatically
- [ ] Test runs use dummy secrets only
- [ ] Production secrets never in repo

---

## 9. Pre-Release Checklist

### 9.1 Security Testing

Before each release, verify:

- [ ] All security tests pass
- [ ] No high/critical vulnerabilities in dependencies
- [ ] Gitleaks scan shows no secrets
- [ ] CodeQL analysis passes
- [ ] Manual security review completed
- [ ] Threat model reviewed and updated

**Command**:
```bash
# Run full security test suite
./tests/claude_bridge/security-check.sh
```

---

### 9.2 Code Review Requirements

Security-sensitive changes require:

- [ ] Review by security-aware team member
- [ ] Threat modeling for new attack surfaces
- [ ] Additional tests for new features
- [ ] Documentation updates

---

## 10. Security Testing Commands

### Quick Reference

```bash
# Build skill binary for testing
cargo build --release

# Run all security tests
cargo test --test security_tests -- --ignored --nocapture

# Run specific security test category
cargo test test_security_no_api_keys -- --ignored --nocapture
cargo test test_security_script_no_command_injection -- --ignored --nocapture
cargo test test_security_path_traversal -- --ignored --nocapture
cargo test test_security_xss -- --ignored --nocapture
cargo test test_security_script_permissions -- --ignored --nocapture

# Run error handling tests
cargo test --test error_tests -- --ignored --nocapture

# Dependency security audits
cargo audit
cargo deny check advisories
cargo deny check licenses

# Secret scanning
gitleaks detect --source . --no-git

# File permission checks
find . -type f -perm -002 ! -path "./.git/*"  # World-writable files
find . -type f -name "*.sh" -perm -002        # World-writable scripts
```

---

## 11. Security Incident Response

### If a Security Issue is Found

1. **Do Not Publicly Disclose**: Report privately to security team
2. **Assess Impact**: Determine scope and severity
3. **Develop Fix**: Create patch in private branch
4. **Test Thoroughly**: Verify fix and add regression test
5. **Coordinate Disclosure**: Prepare advisory and release notes
6. **Release**: Push fix and notify users
7. **Post-Mortem**: Review root cause and improve processes

### Reporting Security Issues

**Contact**: security@kubiya.com (or open a private GitHub Security Advisory)

**Include**:
- Description of vulnerability
- Steps to reproduce
- Proof of concept (if applicable)
- Suggested fix (if known)

---

## 12. Security Training Resources

### Recommended Reading

- **OWASP Top 10**: https://owasp.org/www-project-top-ten/
- **Rust Security**: https://anssi-fr.github.io/rust-guide/
- **Cargo Security**: https://doc.rust-lang.org/cargo/reference/security.html
- **GitHub Security Best Practices**: https://docs.github.com/en/code-security

### Tools

- `cargo-audit`: Vulnerability scanning
- `cargo-deny`: Policy enforcement
- `gitleaks`: Secret scanning
- `semgrep`: SAST analysis
- `CodeQL`: Static analysis

---

## Appendix A: Threat Model

### Attack Surfaces

1. **Manifest Parsing**: TOML injection, path traversal
2. **Skill Generation**: Template injection, XSS
3. **Script Generation**: Command injection, privilege escalation
4. **File Operations**: Path traversal, symlink attacks
5. **MCP Protocol**: JSON injection, protocol abuse
6. **Dependencies**: Supply chain attacks

### Mitigations

- Input validation at all boundaries
- Principle of least privilege
- Fail secure defaults
- Defense in depth
- Regular security audits

---

## Appendix B: Security Metrics

Track these metrics over time:

- Number of security vulnerabilities found (in-house vs external)
- Time to fix security issues
- Test coverage for security-sensitive code
- Dependency update frequency
- CI/CD test pass rate

---

**Document Maintainer**: Claude Bridge Security Team
**Review Frequency**: Quarterly or after security incidents
**Next Review**: 2026-04-04
