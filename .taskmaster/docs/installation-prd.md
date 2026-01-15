# Skill Engine - Installation & Distribution PRD

## Executive Summary

Create a frictionless installation experience for Skill Engine that works everywhere: CI pipelines, Python scripts, shell sessions, and containers. The goal is a single curl command that gets users from zero to running skills in under 30 seconds.

## Design Principles

1. **Non-interactive by default** - Must work in CI/CD, scripts, containers
2. **Fast** - Download and run in seconds, not minutes
3. **Universal** - Works on macOS, Linux, Windows (WSL)
4. **No dependencies** - Single static binary, no runtime required
5. **Programmatic-first** - Easy to call from Python, Node, Go, etc.

## Installation Methods

### 1. Curl One-Liner (Primary)

```bash
curl -fsSL https://skill.sh/install | sh
```

Or with explicit version:
```bash
curl -fsSL https://skill.sh/install | sh -s -- --version 0.1.0
```

**Behavior**:
- Detects OS (darwin/linux) and architecture (amd64/arm64)
- Downloads pre-built binary from GitHub releases
- Installs to `~/.skill-engine/bin/skill`
- Adds to PATH (appends to ~/.bashrc, ~/.zshrc, or prints instructions)
- Verifies installation with `skill --version`
- Silent by default, verbose with `--verbose`

**Environment Variables**:
```bash
SKILL_INSTALL_DIR=/custom/path  # Override install location
SKILL_NO_MODIFY_PATH=1          # Don't modify shell rc files
SKILL_VERSION=0.1.0             # Specific version
```

### 2. Cargo Install (Rust Developers)

```bash
cargo install skill-cli
```

Or from source:
```bash
cargo install --git https://github.com/kubiyabot/skill --bin skill
```

### 3. Package Managers (Future)

```bash
# Homebrew (macOS/Linux)
brew install kubiyabot/tap/skill

# apt (Debian/Ubuntu)
curl -fsSL https://skill.sh/gpg | sudo gpg --dearmor -o /usr/share/keyrings/skill.gpg
echo "deb [signed-by=/usr/share/keyrings/skill.gpg] https://apt.skill.sh stable main" | sudo tee /etc/apt/sources.list.d/skill.list
sudo apt update && sudo apt install skill

# Nix
nix-env -iA nixpkgs.skill
```

### 4. Container Image

```dockerfile
FROM ghcr.io/kubiyabot/skill:latest

# Or multi-stage
COPY --from=ghcr.io/kubiyabot/skill:latest /usr/local/bin/skill /usr/local/bin/skill
```

### 5. GitHub Action

```yaml
- uses: kubiyabot/skill-action@v1
  with:
    version: latest
```

## Install Script Specification

### `install.sh`

```bash
#!/bin/sh
# Skill Engine Installer
# Usage: curl -fsSL https://skill.sh/install | sh
#
# Environment:
#   SKILL_INSTALL_DIR  - Installation directory (default: ~/.skill-engine/bin)
#   SKILL_NO_MODIFY_PATH - Don't modify PATH in shell rc files
#   SKILL_VERSION      - Specific version to install (default: latest)

set -e

# Configuration
GITHUB_REPO="kubiyabot/skill"
INSTALL_DIR="${SKILL_INSTALL_DIR:-$HOME/.skill-engine/bin}"
BINARY_NAME="skill"

# Detect platform
detect_platform() {
    OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
    ARCH="$(uname -m)"

    case "$ARCH" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *) error "Unsupported architecture: $ARCH" ;;
    esac

    case "$OS" in
        darwin) OS="apple-darwin" ;;
        linux) OS="unknown-linux-gnu" ;;
        *) error "Unsupported OS: $OS" ;;
    esac

    PLATFORM="${ARCH}-${OS}"
}

# Get latest version from GitHub
get_latest_version() {
    if [ -n "$SKILL_VERSION" ]; then
        echo "$SKILL_VERSION"
        return
    fi

    curl -fsSL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" | \
        grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^v//'
}

# Download and install
install() {
    VERSION=$(get_latest_version)
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/skill-${PLATFORM}.tar.gz"

    echo "Installing Skill Engine v${VERSION}..."

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download and extract
    curl -fsSL "$DOWNLOAD_URL" | tar -xz -C "$INSTALL_DIR"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    # Verify
    "${INSTALL_DIR}/${BINARY_NAME}" --version

    echo "Installed to: ${INSTALL_DIR}/${BINARY_NAME}"
}

# Add to PATH
setup_path() {
    [ -n "$SKILL_NO_MODIFY_PATH" ] && return

    EXPORT_LINE="export PATH=\"${INSTALL_DIR}:\$PATH\""

    for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
        if [ -f "$rc" ] && ! grep -q "skill-engine" "$rc"; then
            echo "" >> "$rc"
            echo "# Skill Engine" >> "$rc"
            echo "$EXPORT_LINE" >> "$rc"
        fi
    done

    echo ""
    echo "Add to your PATH: $EXPORT_LINE"
    echo "Or restart your shell"
}

# Main
main() {
    detect_platform
    install
    setup_path

    echo ""
    echo "✓ Skill Engine installed successfully!"
    echo ""
    echo "Get started:"
    echo "  skill --help"
    echo "  skill install ./examples/kubernetes-skill"
    echo "  skill serve  # Start MCP server"
}

main "$@"
```

## Binary Release Process

### GitHub Actions Workflow

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package
        run: |
          cd target/${{ matrix.target }}/release
          tar -czf skill-${{ matrix.target }}.tar.gz skill

      - name: Upload
        uses: actions/upload-artifact@v4
        with:
          name: skill-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/skill-${{ matrix.target }}.tar.gz

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            skill-*/skill-*.tar.gz
```

## Programmatic Usage

### Python

```python
import subprocess
import shutil

def ensure_skill_installed():
    """Ensure skill CLI is installed, install if missing."""
    if shutil.which("skill"):
        return True

    # Install via curl
    subprocess.run(
        "curl -fsSL https://skill.sh/install | sh",
        shell=True,
        check=True,
        env={"SKILL_NO_MODIFY_PATH": "1", **os.environ}
    )
    return True

def run_skill(skill_name: str, tool: str, **args) -> dict:
    """Run a skill tool and return JSON output."""
    cmd = ["skill", "run", f"{skill_name}:{tool}", "--format", "json"]
    for k, v in args.items():
        cmd.extend([f"--{k}", str(v)])

    result = subprocess.run(cmd, capture_output=True, text=True)
    return json.loads(result.stdout)

# Usage
ensure_skill_installed()
pods = run_skill("kubernetes", "get", resource="pods", namespace="default")
```

### Node.js

```javascript
const { execSync, spawn } = require('child_process');
const which = require('which');

async function ensureSkillInstalled() {
  try {
    which.sync('skill');
    return true;
  } catch {
    execSync('curl -fsSL https://skill.sh/install | sh', {
      stdio: 'inherit',
      env: { ...process.env, SKILL_NO_MODIFY_PATH: '1' }
    });
    return true;
  }
}

async function runSkill(skill, tool, args = {}) {
  const cmd = ['skill', 'run', `${skill}:${tool}`, '--format', 'json'];
  for (const [k, v] of Object.entries(args)) {
    cmd.push(`--${k}`, String(v));
  }

  const result = execSync(cmd.join(' '), { encoding: 'utf8' });
  return JSON.parse(result);
}

// Usage
await ensureSkillInstalled();
const pods = await runSkill('kubernetes', 'get', { resource: 'pods' });
```

### Go

```go
package skill

import (
    "encoding/json"
    "os"
    "os/exec"
)

func EnsureInstalled() error {
    if _, err := exec.LookPath("skill"); err == nil {
        return nil
    }

    cmd := exec.Command("sh", "-c", "curl -fsSL https://skill.sh/install | sh")
    cmd.Env = append(os.Environ(), "SKILL_NO_MODIFY_PATH=1")
    return cmd.Run()
}

func Run(skill, tool string, args map[string]string) (map[string]interface{}, error) {
    cmdArgs := []string{"run", skill + ":" + tool, "--format", "json"}
    for k, v := range args {
        cmdArgs = append(cmdArgs, "--"+k, v)
    }

    out, err := exec.Command("skill", cmdArgs...).Output()
    if err != nil {
        return nil, err
    }

    var result map[string]interface{}
    json.Unmarshal(out, &result)
    return result, nil
}
```

## CI/CD Examples

### GitHub Actions

```yaml
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Install Skill
        run: curl -fsSL https://skill.sh/install | sh

      - name: Install Kubernetes skill
        run: |
          export PATH="$HOME/.skill-engine/bin:$PATH"
          skill install https://github.com/kubiyabot/kubernetes-skill

      - name: Deploy
        run: |
          skill run kubernetes:apply --file ./k8s/deployment.yaml
```

### GitLab CI

```yaml
deploy:
  image: ubuntu:latest
  before_script:
    - apt-get update && apt-get install -y curl
    - curl -fsSL https://skill.sh/install | sh
    - export PATH="$HOME/.skill-engine/bin:$PATH"
  script:
    - skill run kubernetes:apply --file ./k8s/deployment.yaml
```

### Jenkins

```groovy
pipeline {
    agent any
    stages {
        stage('Install Skill') {
            steps {
                sh 'curl -fsSL https://skill.sh/install | sh'
            }
        }
        stage('Deploy') {
            steps {
                sh '''
                    export PATH="$HOME/.skill-engine/bin:$PATH"
                    skill run kubernetes:apply --file ./k8s/deployment.yaml
                '''
            }
        }
    }
}
```

## CLI Flags for Non-Interactive Use

The CLI should support these flags for CI/programmatic use:

```bash
# Silent mode (no output except errors)
skill --quiet run kubernetes:get --resource pods

# JSON output for parsing
skill run kubernetes:get --resource pods --format json

# Exit codes
# 0 = success
# 1 = error
# 2 = skill not found

# Version check
skill --version

# Health check (useful in containers)
skill health
```

## Success Metrics

1. **Installation time** < 10 seconds on typical connection
2. **Binary size** < 50MB (ideally < 30MB)
3. **Zero dependencies** - static binary, no runtime needed
4. **Works in CI** - GitHub Actions, GitLab, Jenkins, CircleCI
5. **Programmatic** - Easy to call from any language

## Implementation Tasks

### Phase 1: Install Script & Releases
1. Create `install.sh` script
2. Set up GitHub Actions release workflow
3. Build binaries for all platforms (x86_64/aarch64, linux/darwin)
4. Host install script at skill.sh domain (or GitHub raw)
5. Test on all platforms

### Phase 2: Package Managers
1. Create Homebrew formula
2. Set up apt repository
3. Create Nix package

### Phase 3: Container & Actions
1. Publish Docker image
2. Create GitHub Action
3. Document CI/CD examples

---

**Document Status**: New PRD
**Created**: 2025-12-21
**Author**: Shaked (with Claude assistance)
