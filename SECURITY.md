# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.3.x   | :white_check_mark: |
| 0.2.x   | :warning: Security fixes only |
| < 0.2   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

To report a security vulnerability, please email [security@kubiya.ai](mailto:security@kubiya.ai).

Include the following information:
- Description of the vulnerability
- Steps to reproduce the issue
- Affected versions
- Potential impact assessment
- Any suggested mitigations (if known)

### Response Timeline

| Stage | Timeline |
|-------|----------|
| Initial Response | 3 business days |
| Status Update | 7 business days |
| Fix Development | Based on severity |

### Severity Levels

| Severity | Description | Target Fix Time |
|----------|-------------|-----------------|
| **Critical** | Remote code execution, sandbox escape, data breach | 24-48 hours |
| **High** | Privilege escalation, authentication bypass | 7 days |
| **Medium** | Information disclosure, denial of service | 30 days |
| **Low** | Minor issues, hardening improvements | Next release |

### Responsible Disclosure

We follow a 90-day responsible disclosure policy:

1. **Acknowledgment**: We will acknowledge receipt within 3 business days
2. **Validation**: We will validate and assess the vulnerability
3. **Development**: We will develop and test a fix
4. **Release**: We will release a security update
5. **Credit**: We will credit you in the security advisory (unless you prefer anonymity)

We ask that you:
- Give us reasonable time to address the issue before public disclosure
- Make a good faith effort to avoid privacy violations, data destruction, or service disruption
- Do not access or modify data that does not belong to you

## Security Best Practices for Contributors

When contributing to Skill, please follow these security guidelines:

- **Never commit secrets**: API keys, tokens, passwords, or credentials must never be committed
- **Use keyring**: For credential storage in skills, use the system keyring via `keyring` crate
- **Respect sandbox boundaries**: WASM skills must not attempt to escape sandbox isolation
- **Validate inputs**: All user inputs in skill implementations must be validated
- **Review security implications**: Consider security impact of new features during code review
- **Report concerns**: If you notice a security issue during development, report it immediately

## Security Features

Skill Engine implements multiple layers of security:

### WASM Sandbox Isolation
All WASM skills run in an isolated WebAssembly environment powered by Wasmtime. Skills cannot:
- Access the host filesystem (unless explicitly granted)
- Make network requests (unless explicitly granted)
- Execute arbitrary system commands
- Access other processes or memory

### Capability-Based Security
Skills must declare required permissions in their manifest:
```toml
[skills.example.capabilities]
network = ["api.example.com"]
filesystem = { read = ["./config"] }
```

### Credential Isolation
- Each skill instance has separate credential storage
- Credentials are stored in the system keyring
- Secrets are never logged or exposed in output

### Native Skill Command Allowlisting
For SKILL.md-based native skills, only explicitly allowed commands can be executed:
```yaml
allowed-tools: Bash, kubectl, helm
```

### Audit Logging
Execution events are logged for security review and debugging.

## Security Scanning

Our CI/CD pipeline includes automated security scanning:

- **cargo-audit**: Checks for known vulnerabilities in dependencies
- **cargo-deny**: Enforces license and security policies
- **gitleaks**: Scans for accidentally committed secrets
- **CodeQL**: Static analysis for security vulnerabilities

## Contact

For security concerns: [security@kubiya.ai](mailto:security@kubiya.ai)

For general questions: [GitHub Discussions](https://github.com/kubiyabot/skill/discussions)
