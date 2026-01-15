# Installation Guide

Comprehensive installation instructions for Skill Engine on all supported platforms.

## System Requirements

### Minimum Requirements
- **OS**: Linux, macOS 11+, or Windows 10+ (with WSL2)
- **RAM**: 512MB available memory
- **Disk**: 50MB for binary + storage for skills

**No Rust compiler needed!** Skill Engine is distributed as a pre-compiled binary.

### Optional Requirements
- **Docker**: For running Docker-based skills (containerized tools)
- **Rust**: Only needed if building from source (development)

## Installation Methods

### Method 1: One-Liner Install (Recommended)

The easiest way to install Skill Engine:

```bash
curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | sh
```

This script:
- Detects your OS and architecture automatically
- Downloads the pre-compiled binary (no Rust needed!)
- Installs to `~/.skill-engine/bin/skill`
- Adds to your PATH (with permission)

**Verify installation:**

```bash
skill --version
# Output: skill 0.3.0
```

**Install specific version:**

```bash
curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | SKILL_VERSION=0.2.0 sh
```

**Non-interactive install (for CI/CD):**

```bash
# Won't modify PATH, won't ask for confirmation
SKILL_NO_MODIFY_PATH=1 curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | sh
```

**Custom install directory:**

```bash
SKILL_INSTALL_DIR=/opt/skill curl -fsSL https://dqkbk9o7ynwhxfjx.public.blob.vercel-storage.com/install.sh | sh
```

### Method 2: Build from Source (Optional)

For developers who want to build from source or contribute:

**Requirements**: Rust 1.75.0 or later ([rustup.rs](https://rustup.rs))

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/kubiyabot/skill.git
cd skill

# Build and install
cargo install --path crates/skill-cli
```

## Platform-Specific Notes

### Linux

The one-liner install works on all Linux distributions. No additional setup needed!

**Optional: Install Docker** (for Docker-based skills):

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install docker.io
sudo systemctl start docker
sudo usermod -aG docker $USER
```

#### Fedora/RHEL
```bash
sudo dnf install docker
sudo systemctl start docker
sudo usermod -aG docker $USER
```

#### Arch Linux
```bash
sudo pacman -S docker
sudo systemctl start docker
sudo usermod -aG docker $USER
```

### macOS

The one-liner install works on all macOS versions 11+.

**Optional: Install Docker** (for Docker-based skills):

```bash
# Download Docker Desktop
# Visit: https://www.docker.com/products/docker-desktop

# Or via Homebrew
brew install --cask docker
```

### Windows (WSL2)

The one-liner install works in WSL2 Ubuntu/Debian.

**Optional: Install Docker**:
1. Download Docker Desktop for Windows with WSL2 backend
2. Visit: https://www.docker.com/products/docker-desktop

## Claude Code Setup

If you're using Claude Code, set up MCP integration automatically:

```bash
skill claude setup
```

This command:
- Finds your `skill` binary automatically
- Creates/updates `.mcp.json` in your current directory
- Configures the MCP server for Claude Code

**For global setup** (use across all projects):

```bash
skill claude setup --global
```

**Verify integration**:

```bash
skill claude status
```

Output:

```
✓ Claude Code integration configured
  Location: /path/to/project/.mcp.json
  Server name: skill-engine
  Binary: /Users/you/.skill-engine/bin/skill
```

**Restart Claude Code** and you're ready to use skills through Claude!

See the [Claude Code Integration Guide](../guides/claude-code.md) for advanced configuration and tips.

## Shell Completion

Enable command-line completion for your shell:

### Bash

```bash
# Generate completion script
skill completions bash > ~/.local/share/bash-completion/completions/skill

# Or add to ~/.bashrc
echo 'eval "$(skill completions bash)"' >> ~/.bashrc
source ~/.bashrc
```

### Zsh

```bash
# Generate completion script
skill completions zsh > ~/.zfunc/_skill

# Add to ~/.zshrc
echo 'fpath=(~/.zfunc $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc
source ~/.zshrc
```

### Fish

```bash
# Generate completion script
skill completions fish > ~/.config/fish/completions/skill.fish
```

## Configuration

### Default Configuration

Create `~/.config/skill-engine/config.toml`:

```toml
# Default manifest file location
manifest_path = ".skill-engine.toml"

# Enable execution history
history_enabled = true
history_limit = 1000

# HTTP server settings
[server]
host = "127.0.0.1"
port = 3000
ui_enabled = true

# Docker runtime settings
[docker]
default_memory = "2g"
default_cpus = "4"
network = "bridge"

# WASM runtime settings
[wasm]
max_memory_pages = 256  # 16MB
max_execution_time_ms = 30000
```

### Environment Variables

Skill Engine respects these environment variables:

```bash
# Configuration file location
export SKILL_ENGINE_CONFIG=~/.config/skill-engine/config.toml

# Manifest file name
export SKILL_ENGINE_MANIFEST=.skill-engine.toml

# Enable debug logging
export RUST_LOG=skill=debug

# Docker host
export DOCKER_HOST=unix:///var/run/docker.sock

# HTTP server port
export SKILL_ENGINE_PORT=3000
```

Add to `~/.bashrc` or `~/.zshrc` to make permanent.

## MCP Server Configuration

For Claude Code and other MCP-compatible agents:

### Claude Code

Create or edit `~/.config/claude/mcp.json`:

```json
{
  "mcpServers": {
    "skill-engine": {
      "command": "skill",
      "args": ["mcp"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Cursor

Create or edit `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "skill-engine": {
      "command": "skill",
      "args": ["mcp"]
    }
  }
}
```

## Verification

Verify your installation is working correctly:

```bash
# Check version
skill --version

# List available skills
skill list

# Check system info
skill info

# Start HTTP server
skill serve &

# Test API endpoint
curl http://localhost:3000/api/health

# Stop server
pkill skill
```

## Troubleshooting

### Command Not Found

**Issue**: `skill: command not found`

**Solution**: Ensure `~/.cargo/bin` is in your PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.bashrc  # or source ~/.zshrc
```

### Docker Permission Denied

**Issue**: `permission denied while trying to connect to Docker`

**Solution**: Add user to docker group:

```bash
sudo usermod -aG docker $USER
newgrp docker

# Or restart system
```

### Rust Version Too Old

**Issue**: `requires rustc 1.75.0 or newer`

**Solution**: Update Rust:

```bash
rustup update stable
rustc --version
```

### Build Errors

**Issue**: Compilation errors during `cargo install`

**Solution**: Update Rust and dependencies:

```bash
rustup update stable
cargo clean
cargo build --release
```

### macOS Gatekeeper Warning

**Issue**: "skill cannot be opened because it is from an unidentified developer"

**Solution**: Allow the application:

```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/skill

# Or right-click → Open in Finder
```

## Uninstallation

### Remove Binary

```bash
# If installed via cargo
cargo uninstall skill-cli

# If installed manually
sudo rm /usr/local/bin/skill
```

### Remove Configuration

```bash
# Remove config directory
rm -rf ~/.config/skill-engine

# Remove completion scripts
rm ~/.local/share/bash-completion/completions/skill  # Bash
rm ~/.zfunc/_skill  # Zsh
rm ~/.config/fish/completions/skill.fish  # Fish
```

### Remove Docker Images

```bash
# List Skill Engine related images
docker images | grep skill

# Remove specific image
docker rmi <image-name>

# Or remove all unused images
docker image prune -a
```

## Updating

### Update via Cargo

```bash
cargo install skill-cli --force
```

### Update from Source

```bash
cd skill
git pull
cargo install --path crates/skill-cli --force
```

## Next Steps

- **[Quick Start Guide](./quick-start.md)**: Run your first skill
- **[CLI Reference](../reference/cli.md)**: Learn all commands
- **[Configuration Guide](../guides/configuration.md)**: Advanced configuration
- **[Building Skills](../guides/building-skills/)**: Create custom skills

## Support

- **GitHub Issues**: [Report bugs](https://github.com/yourusername/skill/issues)
- **Discussions**: [Ask questions](https://github.com/yourusername/skill/discussions)
- **Documentation**: [docs.skill-engine.dev](https://yourusername.github.io/skill/)
