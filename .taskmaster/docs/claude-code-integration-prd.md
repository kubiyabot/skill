# PRD: Skill Engine + Claude Code Stateless Integration

## Overview

Enable users to configure Skill Engine with Claude Code using a single manifest file (`.skill-engine.toml`) that defines all skills and their configurations. This provides a stateless, reproducible, version-controlled setup that makes team onboarding effortless.

## Problem Statement

Currently, users face several challenges when setting up Skill Engine with Claude Code:

1. **Installation Complexity**: Users must run `skill install` for each skill, creating state in `~/.skill-engine/registry/`
2. **Not Version Controlled**: Installed skills and configs live in home directory, can't be shared via git
3. **Team Onboarding Friction**: Each team member must manually install and configure skills
4. **Environment-Specific Configs**: Hard to maintain dev/staging/prod configurations
5. **Documentation Gap**: No clear guide showing the manifest-based (stateless) approach
6. **Unclear Workflow**: Users don't know that `.skill-engine.toml` is the preferred method for Claude Code

## Goals

### Primary Goals
1. **Zero-Install Setup**: Users should be able to `git clone` a project and immediately use Claude Code with all configured skills
2. **Stateless Configuration**: All skill definitions and configs live in `.skill-engine.toml`, checked into version control
3. **Team Reproducibility**: Entire team uses identical skill configuration from the manifest
4. **Clear Documentation**: Comprehensive guides showing the manifest-first approach as the recommended method

### Secondary Goals
1. Support multiple skill instances (dev, staging, prod) per skill
2. Enable environment variable substitution for secrets
3. Provide example manifests for common use cases
4. Make skill discovery/loading transparent to users

## Solution

### Architecture Overview

Skill Engine already supports manifest-based configuration. The solution involves:

1. **Creating Clear Documentation** showing manifest-first workflow
2. **Example Manifest Templates** for common scenarios (minimal, team, enterprise)
3. **MCP Configuration Guide** showing how to point Claude Code to the manifest
4. **Environment Variable Patterns** for managing secrets
5. **Migration Guide** from installed skills to manifest-based approach

### Technical Implementation

#### 1. Manifest File Structure

```toml
# .skill-engine.toml
version = "1"

# Skills are defined with sources and runtime types
[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"  # Local path
runtime = "native"  # CLI wrapper
description = "Kubernetes cluster management"

# Multiple instances with different configs
[skills.kubernetes.instances.default]
config.cluster = "minikube"

[skills.kubernetes.instances.prod]
config.cluster = "production"
config.kubeconfig = "${KUBECONFIG:-~/.kube/config}"
capabilities.network_access = true

# Docker-based skills
[skills.python-runner]
source = "docker:python:3.12-slim"
runtime = "docker"
description = "Python script execution (sandboxed)"

[skills.python-runner.docker]
image = "python:3.12-slim"
entrypoint = "python3"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
memory = "512m"
network = "none"

# WASM skills with API integrations
[skills.github]
source = "./examples/wasm-skills/github-skill"
runtime = "wasm"
description = "GitHub API integration"

[skills.github.instances.default]
config.token = "${SKILL_GITHUB_TOKEN}"
capabilities.network_access = true
```

#### 2. Claude Code MCP Configuration

```json
// .mcp.json
{
  "mcpServers": {
    "skill-engine": {
      "type": "stdio",
      "command": "skill",
      "args": ["serve"]
    }
  }
}
```

The `skill serve` command automatically:
- Searches for `.skill-engine.toml` in current directory
- Walks up directory tree if not found
- Falls back to `skill-engine.toml` (without leading dot)
- Loads all skills from manifest (no installation needed)
- Expands environment variables
- Exposes tools to Claude Code via MCP

#### 3. Permissions Configuration

```json
// .claude/settings.json or .claude/settings.local.json
{
  "permissions": {
    "allow": [
      "mcp__skill-engine__*"
    ]
  },
  "enabledMcpjsonServers": [
    "skill-engine"
  ]
}
```

### User Workflow

#### For Individual Users

```bash
# 1. Create manifest in project root
cat > .skill-engine.toml <<'EOF'
version = "1"

[skills.git]
source = "./examples/native-skills/git-skill"
runtime = "native"
[skills.git.instances.default]

[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
runtime = "native"
[skills.kubernetes.instances.default]
EOF

# 2. Configure Claude Code MCP
cat > .mcp.json <<'EOF'
{
  "mcpServers": {
    "skill-engine": {
      "command": "skill",
      "args": ["serve"]
    }
  }
}
EOF

# 3. Set permissions (one-time)
mkdir -p .claude
cat > .claude/settings.json <<'EOF'
{
  "permissions": {
    "allow": ["mcp__skill-engine__*"]
  },
  "enabledMcpjsonServers": ["skill-engine"]
}
EOF

# 4. Start Claude Code
claude
```

#### For Teams

```bash
# Team lead commits .skill-engine.toml + .mcp.json to git
git add .skill-engine.toml .mcp.json .claude/
git commit -m "Add skill-engine configuration"
git push

# Team members clone and start
git clone <repo>
cd <repo>
claude  # All skills automatically available!
```

### Example Manifests

#### Minimal Setup (Developer Essentials)

```toml
version = "1"

[skills.git]
source = "./examples/native-skills/git-skill"
runtime = "native"
[skills.git.instances.default]

[skills.docker]
source = "./examples/native-skills/docker-skill"
runtime = "native"
[skills.docker.instances.default]

[skills.http]
source = "./examples/wasm-skills/http-skill"
[skills.http.instances.default]
```

#### Team Setup (Full Stack)

```toml
version = "1"

# Version Control
[skills.git]
source = "./examples/native-skills/git-skill"
runtime = "native"
[skills.git.instances.default]

[skills.github]
source = "./examples/wasm-skills/github-skill"
[skills.github.instances.default]
config.token = "${SKILL_GITHUB_TOKEN}"

# Infrastructure
[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
runtime = "native"

[skills.kubernetes.instances.dev]
config.cluster = "dev-cluster"

[skills.kubernetes.instances.prod]
config.cluster = "prod-cluster"
config.kubeconfig = "/etc/kube/prod-config"

[skills.terraform]
source = "./examples/native-skills/terraform-skill"
runtime = "native"
[skills.terraform.instances.default]

# Databases
[skills.postgres-native]
source = "./examples/native-skills/postgres-skill"
runtime = "native"

[skills.postgres-native.instances.default]
config.host = "${POSTGRES_HOST:-localhost}"
config.port = "${POSTGRES_PORT:-5432}"
config.database = "${POSTGRES_DB:-myapp}"

# CI/CD & Project Management
[skills.circleci]
source = "./examples/wasm-skills/circleci-skill"
[skills.circleci.instances.default]
config.token = "${SKILL_CIRCLECI_TOKEN}"

[skills.jira]
source = "./examples/wasm-skills/jira-skill"
[skills.jira.instances.default]
config.url = "${SKILL_JIRA_URL}"
config.email = "${SKILL_JIRA_EMAIL}"
config.token = "${SKILL_JIRA_TOKEN}"

[skills.slack]
source = "./examples/wasm-skills/slack-skill"
[skills.slack.instances.default]
config.token = "${SKILL_SLACK_TOKEN}"
```

## Documentation Plan

### New Documents to Create

1. **`docs/QUICK_START_CLAUDE_CODE.md`**
   - 3-minute setup guide
   - Copy/paste manifest examples
   - MCP configuration
   - Verification steps

2. **`docs/MANIFEST_GUIDE.md`**
   - Complete manifest syntax reference
   - All source types (local, docker, git, http)
   - Runtime types (wasm, native, docker)
   - Environment variable patterns
   - Instance configurations
   - Capabilities and permissions

3. **`docs/ENVIRONMENT_VARIABLES.md`**
   - Best practices for secret management
   - `.env` file patterns
   - CI/CD integration
   - Team secret sharing strategies

4. **`examples/manifests/`**
   - `minimal.toml` - 5-6 essential skills
   - `team.toml` - Full team setup
   - `enterprise.toml` - Large org with multiple teams
   - `devops.toml` - Infrastructure focused
   - `data-engineering.toml` - Data/analytics focused

5. **`docs/MIGRATION_GUIDE.md`**
   - Moving from installed skills to manifest
   - Script to generate manifest from installed skills
   - Comparison table

### Updates to Existing Docs

1. **`README.md`**
   - Add "Quick Start with Claude Code" section
   - Emphasize manifest-first approach
   - Link to detailed guides

2. **`docs/CLAUDE_CODE_INSTALLATION.md`**
   - Simplify to focus on manifest approach
   - Move complex scenarios to dedicated guides

## Success Metrics

### Quantitative
- Time to first successful Claude Code + skill execution: < 3 minutes
- Number of manual commands required: ≤ 3 (copy manifest, configure MCP, start Claude)
- Team onboarding steps: 1 (git clone)

### Qualitative
- Users understand manifest is the preferred approach
- No confusion about "do I need to install skills?"
- Clear mental model: manifest = source of truth

## Implementation Tasks

### Phase 1: Documentation (Priority: High)
- [ ] Create `docs/QUICK_START_CLAUDE_CODE.md`
- [ ] Create `docs/MANIFEST_GUIDE.md`
- [ ] Create example manifests in `examples/manifests/`
- [ ] Update `README.md` with Claude Code section
- [ ] Create `docs/ENVIRONMENT_VARIABLES.md`

### Phase 2: Example Improvements (Priority: High)
- [ ] Ensure all example skills have SKILL.md documentation
- [ ] Verify all example skills work in manifest mode
- [ ] Add comments to `.skill-engine.toml` explaining each section
- [ ] Create minimal working examples for testing

### Phase 3: Tooling Enhancements (Priority: Medium)
- [ ] Add `skill manifest init` command to generate starter manifest
- [ ] Add `skill manifest validate` to check manifest syntax
- [ ] Add `skill manifest migrate` to convert installed skills to manifest entries
- [ ] Improve error messages when manifest is invalid

### Phase 4: Testing (Priority: High)
- [ ] Test manifest loading in different directory structures
- [ ] Test environment variable expansion edge cases
- [ ] Test with real Claude Code instance
- [ ] Create integration test suite for MCP server + manifest

### Phase 5: Community (Priority: Low)
- [ ] Create video tutorial showing setup
- [ ] Blog post: "Zero-Install Skill Configuration with Claude Code"
- [ ] Example projects showcasing manifest patterns
- [ ] Community manifest registry

## Open Questions

1. **Skill Source URLs**: Should we support remote manifest URLs?
   - Example: `source = "https://skills.example.com/kubernetes-v1.0.0.wasm"`
   - Concerns: Security, verification, caching

2. **Manifest Inheritance**: Should manifests support extends/includes?
   - Example: Base company manifest + project-specific additions
   - Pattern: `.skill-engine.base.toml` + `.skill-engine.toml`

3. **Auto-Discovery**: Should `skill serve` search parent directories?
   - Current: Walks up directory tree to find manifest
   - Alternative: Strict current-directory-only

4. **Default Manifest**: Should skill-engine ship with a default manifest?
   - Location: Built-in or downloaded on first run
   - Content: Common utilities (git, http, docker)

5. **Manifest Validation**: Pre-flight validation before starting server?
   - Pros: Fail fast with clear errors
   - Cons: Slower startup, may block valid configurations

## Risk Mitigation

### Risk: Users still use `skill install` out of habit
**Mitigation**:
- Prominent banner in docs emphasizing manifest approach
- `skill install` shows tip about manifests
- Quick start guide shows only manifest method

### Risk: Environment variable management is confusing
**Mitigation**:
- Clear examples in every guide
- `.env.example` file in project templates
- Error messages show which env vars are missing

### Risk: Manifest syntax errors break setup
**Mitigation**:
- Add `skill manifest validate` command
- Clear error messages with line numbers
- Well-commented example manifests

### Risk: Existing users have installed skills, confusion about migration
**Mitigation**:
- Create migration guide
- `skill manifest migrate` to auto-generate manifest from installed skills
- Show both approaches in docs with clear trade-offs

## Success Criteria

The implementation is successful when:

1. ✅ A new user can set up Claude Code + skill-engine in < 3 minutes
2. ✅ Teams can share skill configurations via git without manual setup
3. ✅ Environment-specific configs (dev/prod) are clearly supported
4. ✅ Documentation clearly communicates manifest-first approach
5. ✅ Error messages guide users to fix common issues
6. ✅ All example skills work in manifest mode
7. ✅ Test suite validates manifest loading and resolution

## Timeline

- **Week 1**: Documentation (Quick Start, Manifest Guide, Examples)
- **Week 2**: Example manifest creation and testing
- **Week 3**: Tooling enhancements (`manifest init`, `validate`, `migrate`)
- **Week 4**: Integration testing with Claude Code, bug fixes
- **Week 5**: Community materials (video, blog post)

## Appendix

### Current Codebase References

- **Manifest Loading**: `crates/skill-runtime/src/manifest.rs:296-482`
- **MCP Server**: `crates/skill-cli/src/commands/serve.rs:76-96`
- **Environment Expansion**: `crates/skill-runtime/src/manifest.rs:531-582`
- **HTTP Server**: `crates/skill-http/src/server.rs:94-96`

### Related Issues/PRs

- N/A (this is the initial PRD)

### References

- [Model Context Protocol Spec](https://modelcontextprotocol.io/)
- [Claude Code Documentation](https://code.claude.com/docs)
- [WebAssembly Component Model](https://component-model.bytecodealliance.org/)
