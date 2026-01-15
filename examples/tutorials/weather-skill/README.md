# Weather Skill Tutorial

This example demonstrates how to create a native skill using the SKILL.md format.

## What You'll Learn

1. **SKILL.md structure** - How to write skill metadata and tool definitions
2. **Parameters** - Required vs optional, types, defaults
3. **Documentation** - Best practices for skill documentation
4. **API integration** - Patterns for calling external APIs

## Prerequisites

- Skill CLI installed (`skill --version`)
- OpenWeatherMap API key (free at https://openweathermap.org/api)

## Step-by-Step Guide

### Step 1: Understand the Structure

Open `SKILL.md` and notice:

1. **Frontmatter** (YAML between `---`):
   ```yaml
   name: weather
   description: Get weather information...
   allowed-tools: Bash
   ```

2. **Tool definitions** (Markdown headings):
   ```markdown
   ### current
   Get current weather for a city.
   ```

3. **Parameters** (Bullet list format):
   ```markdown
   **Parameters:**
   - `city` (required, string): City name
   ```

### Step 2: Install the Skill

```bash
# From this directory
skill install .

# Verify installation
skill list
skill info weather
```

### Step 3: Configure the API Key

```bash
# Option 1: Environment variable
export WEATHER_API_KEY="your-api-key"

# Option 2: Skill config (stored securely)
skill config weather
```

### Step 4: Test the Skill

```bash
# Get current weather
skill run weather:current city="London"

# Get forecast
skill run weather:forecast city="Tokyo" days=5
```

## Key Concepts

### Parameter Format

```markdown
- `name` (required|optional, type): Description
```

- **name**: Parameter name (no spaces)
- **required/optional**: Whether the parameter must be provided
- **type**: One of `string`, `integer`, `number`, `boolean`, `array`

### Allowed Tools

The `allowed-tools` field controls what commands can be executed:

| Tool | What It Allows |
|------|----------------|
| `Bash` | Shell commands (curl, jq, etc.) |
| `kubectl` | Kubernetes commands |
| `git` | Git commands |
| `docker` | Docker commands |

### Best Practices

1. **Clear descriptions**: Help AI agents understand when to use each tool
2. **Helpful examples**: Show realistic usage patterns
3. **Document requirements**: API keys, dependencies, etc.
4. **Error handling**: Describe what happens on failure

## Files in This Example

| File | Purpose |
|------|---------|
| `SKILL.md` | Skill definition (heavily commented) |
| `README.md` | This tutorial guide |

## Next Steps

- Try modifying the skill to add a new tool
- Create your own skill for a different API
- Explore [WASM skills](/tutorials/first-wasm-skill) for custom logic

## Related Documentation

- [SKILL.md Format Reference](/api/skill-md-format)
- [Your First Native Skill](/tutorials/first-native-skill)
- [API Integration Tutorial](/tutorials/api-integration)
