# Agentic Skills Marketplace

This directory contains curated skill definitions for the Agentic Skills Marketplace web application. These are self-contained, production-ready skills that can be used by any AI agent.

## Directory Structure

```
marketplace/
├── schema.json          # JSON schema for skill manifest validation
├── categories.json      # Category taxonomy and metadata
├── skills/              # Individual skill manifests (JSON)
│   ├── kubernetes.json
│   ├── github.json
│   └── ...
└── README.md           # This file
```

## Adding a New Skill

### 1. Create a Skill Manifest

Create a new JSON file in `skills/` directory with the skill ID as the filename (e.g., `my-skill.json`).

### 2. Required Fields

Every skill manifest must include:

- `id` - Unique identifier (kebab-case, URL-safe)
- `name` - Display name
- `type` - Runtime type (`wasm`, `native`, or `docker`)
- `description` - Short description (20-200 characters)
- `version` - Semantic version (e.g., "1.0.0")
- `author` - Author information with at least a `name`
- `categories` - Array of 1-3 categories from predefined list
- `installation` - Installation configuration with `source` path

### 3. Example Manifest

```json
{
  "id": "my-skill",
  "name": "My Skill",
  "icon": "https://cdn.simpleicons.org/yourbrand",
  "skillMdUrl": "https://raw.githubusercontent.com/yourusername/repo/main/path/to/SKILL.md",
  "type": "native",
  "description": "A brief description of what this skill does and when to use it.",
  "version": "1.0.0",
  "author": {
    "name": "Your Name",
    "github": "yourusername"
  },
  "categories": ["development"],
  "badges": ["community"],
  "installation": {
    "source": "./examples/native-skills/my-skill",
    "requiresAuth": false
  },
  "tools": [
    {
      "name": "example-tool",
      "description": "What this tool does",
      "parameters": [
        {
          "name": "input",
          "type": "string",
          "required": true,
          "description": "Input parameter description"
        }
      ]
    }
  ],
  "examples": [
    {
      "title": "Basic usage",
      "code": "skill run my-skill:example-tool input=\"hello\""
    }
  ],
  "links": {
    "github": "https://github.com/kubiyabot/skill/tree/main/examples/native-skills/my-skill"
  }
}
```

### 4. Validate Your Manifest

Run the validation script to ensure your manifest is correct:

```bash
cd marketplace-web
npm run validate
```

### 5. Submit a Pull Request

Once validated, submit a PR with your new skill manifest. Include:

- The skill manifest JSON file
- Updated examples/code if needed
- Description of what the skill does

## Available Categories

Choose 1-3 categories from:

- `featured` - Hand-picked essential skills
- `devops` - Kubernetes, Docker, Terraform
- `cloud` - AWS, Azure, GCP integrations
- `database` - PostgreSQL, Redis, MongoDB
- `api` - GitHub, Slack, HTTP client
- `messaging` - Slack, email, notifications
- `monitoring` - Prometheus, metrics, logs
- `media` - FFmpeg, ImageMagick
- `development` - Git, CI/CD, testing
- `utilities` - Python runner, Node.js runner

## Badges

You can add badges to highlight skill status:

- `official` - Officially maintained by Skill Engine team
- `featured` - Featured on homepage
- `verified` - Verified by community
- `community` - Community-contributed

## Skill Types

### WASM Skills (`type: "wasm"`)

WebAssembly skills with sandboxed execution. Usually API integrations.

**Example:** GitHub, Slack, HTTP client

### Native Skills (`type: "native"`)

CLI wrapper skills that execute native commands.

**Example:** Kubernetes, Git, Terraform

### Docker Skills (`type: "docker"`)

Skills that run inside Docker containers for isolation.

**Example:** FFmpeg, Python runner, ImageMagick

## Icon and Documentation

### Skill Icons

Add a custom SVG icon to your skill using the `icon` field:

```json
{
  "icon": "https://cdn.simpleicons.org/kubernetes"
}
```

**Recommended sources:**
- [Simple Icons CDN](https://simpleicons.org/) - 2,800+ brand logos
- Custom hosted SVG files
- Format: `https://cdn.simpleicons.org/[brand-name]`

### SKILL.md Documentation

Link to your skill's comprehensive SKILL.md file using the `skillMdUrl` field:

```json
{
  "skillMdUrl": "https://raw.githubusercontent.com/kubiyabot/skill/main/examples/native-skills/kubernetes-skill/SKILL.md"
}
```

**What is SKILL.md?**
- Complete documentation with all tools, parameters, and examples
- Automatically rendered on the skill detail page
- Supports full markdown with frontmatter
- Should include: tool descriptions, parameters, examples, configuration, security notes

**Format:**
```markdown
---
name: my-skill
description: Brief description
allowed-tools: Bash, skill-run
---

# My Skill

## When to Use
...

## Tools Provided

### tool-name
Description...
```

The marketplace will automatically fetch and render your SKILL.md with proper syntax highlighting and formatting.

## Best Practices

1. **Keep descriptions concise** - 20-200 characters for card display
2. **Add comprehensive tools** - Document all parameters with types
3. **Include examples** - Show real-world usage with 3-5 examples
4. **Link to documentation** - Provide GitHub and docs links
5. **Specify requirements** - List required CLI tools and platforms
6. **Use clear naming** - Choose descriptive, searchable names
7. **Add custom icons** - Use brand logos from Simple Icons CDN
8. **Create SKILL.md** - Provide complete documentation for your skill

## Questions?

Open an issue on GitHub or reach out to the maintainers.
