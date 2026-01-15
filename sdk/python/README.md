# Skill Engine Python SDK

Build powerful skill plugins for Skill Engine using Python.

## Installation

```bash
pip install skill-engine-sdk
```

For development with WASM compilation:

```bash
pip install skill-engine-sdk[build]
```

## Quick Start

### 1. Create a new skill project

```bash
skill-sdk init my-awesome-skill
cd my-awesome-skill
```

### 2. Edit your skill

```python
# src/main.py
from skill_sdk import Skill, tool, param

@Skill(
    name="my-awesome-skill",
    description="My awesome skill that does amazing things",
    version="1.0.0",
)
class MyAwesomeSkill:
    @tool(description="Greet someone by name")
    @param("name", "The name to greet")
    def hello(self, name: str = "World") -> str:
        return f"Hello, {name}!"

    @tool(description="Calculate the sum of two numbers")
    @param("a", "First number")
    @param("b", "Second number")
    def add(self, a: int, b: int) -> int:
        return a + b

if __name__ == "__main__":
    MyAwesomeSkill.run()
```

### 3. Build and install

```bash
# Build to WASM
skill-sdk build

# Install in skill-engine
skill install ./my-awesome-skill.wasm
```

### 4. Run your tools

```bash
skill run my-awesome-skill:hello --name "Developer"
skill run my-awesome-skill:add --a 5 --b 3
```

## Features

### Decorators

- `@Skill()` - Define a skill class
- `@tool()` - Mark a method as a tool
- `@param()` - Document a parameter
- `@config()` - Define configuration fields

### Configuration

Skills can define configuration fields that users set at install time:

```python
@config("api_key", "API key for service", required=True, secret=True)
@config("region", "AWS region", default="us-east-1")
@Skill(name="my-skill", description="My skill")
class MySkill:
    def __init__(self):
        # Access config values
        self.api_key = self.get_config("api_key")
        self.region = self.get_config("region")
```

### Return Types

Tools can return various types:

```python
@tool(description="Return a string")
def string_result(self) -> str:
    return "Hello!"

@tool(description="Return a dict")
def dict_result(self) -> dict:
    return {"status": "ok", "count": 42}

@tool(description="Return ExecutionResult for more control")
def custom_result(self) -> ExecutionResult:
    return ExecutionResult.ok(
        "Operation completed",
        data={"details": "..."}
    )
```

## Project Structure

```
my-skill/
├── skill.yaml        # Skill manifest
├── SKILL.md          # Documentation (for semantic search)
├── pyproject.toml    # Python project config
├── src/
│   ├── __init__.py
│   └── main.py       # Skill implementation
└── tests/
    └── test_skill.py
```

## CLI Commands

```bash
# Create a new skill project
skill-sdk init <name> [--template basic|advanced|api]

# Build to WASM component
skill-sdk build [--output skill.wasm]

# Validate project structure
skill-sdk validate

# Show skill info
skill-sdk info

# Run tests
skill-sdk test
```

## Templates

### Basic Template
Simple skill with basic tools:
```bash
skill-sdk init my-skill --template basic
```

### Advanced Template
Skill with configuration support:
```bash
skill-sdk init my-skill --template advanced
```

### API Template
API integration skill with HTTP helpers:
```bash
skill-sdk init my-skill --template api
```

## Documentation with SKILL.md

Document your skill for semantic search:

```markdown
---
name: my-skill
description: Brief description for search
allowed-tools:
  - Read
  - Bash
---

# My Skill

Detailed description...

## Tools Provided

### tool-name
What this tool does.

**Usage**:
```bash
skill run my-skill:tool-name --param value
```

**Parameters**:
- `param` (required): Description
```

## Testing

```python
# tests/test_skill.py
import pytest
from src.main import MySkill

def test_hello():
    skill = MySkill()
    result = skill.hello(name="Test")
    assert "Test" in result

def test_add():
    skill = MySkill()
    assert skill.add(2, 3) == 5
```

Run tests:
```bash
skill-sdk test
# or
pytest -v tests/
```

## Type Safety

The SDK uses Python type hints for:
- Parameter type inference
- Return type validation
- IDE autocompletion

```python
@tool(description="Process data")
def process(
    self,
    data: str,           # Required string
    count: int = 10,     # Optional int with default
    enabled: bool = True # Optional bool with default
) -> dict:
    return {"processed": data, "count": count}
```

## Error Handling

```python
from skill_sdk import ExecutionResult, ValidationError

@tool(description="Safe operation")
def safe_operation(self, value: str) -> ExecutionResult:
    if not value:
        return ExecutionResult.error("Value cannot be empty")

    try:
        result = some_operation(value)
        return ExecutionResult.ok(f"Success: {result}")
    except Exception as e:
        return ExecutionResult.error(f"Operation failed: {e}")
```

## License

MIT
