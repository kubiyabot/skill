---
name: test-skill
description: test-skill - brief description of what this skill does
allowed-tools:
  - Read
  - Bash
# Optional metadata:
# version: 1.0.0
# author: Your Name
# category: utility
# tags:
#   - example
#   - template
---

# Test Skill

Brief description of what this skill does and its main purpose.

## When to Use

- Use case 1: Describe when to use this skill
- Use case 2: Another scenario
- Use case 3: Additional context

## Tools Provided

### example-tool
Description of what example-tool does.

**Usage**:
```bash
skill run test-skill:example-tool --param1 value1 --param2 value2
```

**Parameters**:
- `param1` (required): Description of first parameter
- `param2` (optional): Description of second parameter

**Example**:
```bash
# Example usage with realistic values
skill run test-skill@default:example-tool --param1 "example value"
```

## Configuration

Describe any configuration required for this skill:

```bash
skill config test-skill --set api_key=YOUR_API_KEY
skill config test-skill --set region=us-east-1
```

## Examples

### Example Workflow 1
```bash
skill run test-skill@default:example-tool --param1 value
```

## Security Notes

- Credentials are stored securely in the system keychain
- Each instance has isolated configuration
- All API calls use TLS encryption
