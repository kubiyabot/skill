---
name: python-example
description: Example Python skill demonstrating the SDK decorator pattern
allowed-tools:
  - Read
  - Bash
version: 1.0.0
author: Skill Engine Team
tags:
  - example
  - python
  - demo
  - calculator
  - greeting
category: utility
---

# Python Example Skill

A comprehensive example demonstrating Python SDK features including decorators, configuration, multiple tools, and proper error handling.

## When to Use

- Learning how to build skills with the Python SDK
- Testing the skill-engine runtime with Python-based skills
- As a template for creating new Python skills

## Tools Provided

### greet

Greet someone with a personalized message. Supports formal and informal greeting styles.

**Usage**:
```bash
skill run python-example:greet --name "Alice"
skill run python-example:greet --name "Alice" --formal true
```

**Parameters**:
- `name` (optional): The name of the person to greet (default: "World")
- `formal` (optional): Use formal greeting style (default: false)

**Example**:
```bash
$ skill run python-example:greet --name "Developer"
Hello, Developer!

$ skill run python-example:greet --name "CEO" --formal true
Good day, CEO. How may I assist you?
```

### echo

Echo back a message with optional transformations like uppercase or reverse.

**Usage**:
```bash
skill run python-example:echo --message "Hello World"
skill run python-example:echo --message "hello" --uppercase true
skill run python-example:echo --message "stressed" --reverse true
```

**Parameters**:
- `message` (required): The message to echo
- `uppercase` (optional): Convert message to uppercase (default: false)
- `reverse` (optional): Reverse the message (default: false)

**Example**:
```bash
$ skill run python-example:echo --message "stressed" --reverse true
desserts
```

### calculate

Perform basic arithmetic operations on two numbers.

**Usage**:
```bash
skill run python-example:calculate --a 10 --b 5 --operation add
skill run python-example:calculate --a 100 --b 7 --operation divide
```

**Parameters**:
- `a` (required): First number
- `b` (required): Second number
- `operation` (optional): Operation to perform - add, subtract, multiply, divide (default: "add")

**Example**:
```bash
$ skill run python-example:calculate --a 15 --b 3 --operation multiply
15 multiply 3 = 45
```

### process_list

Process a JSON array of numbers and return statistics including count, sum, min, max, and average.

**Usage**:
```bash
skill run python-example:process_list --items '[1, 2, 3, 4, 5]'
```

**Parameters**:
- `items` (required): JSON array of numbers to process

**Example**:
```bash
$ skill run python-example:process_list --items '[10, 20, 30, 40, 50]'
Processed 5 items
{
  "count": 5,
  "sum": 150,
  "min": 10,
  "max": 50,
  "average": 30.0
}
```

### status

Get current skill configuration and status including available tools.

**Usage**:
```bash
skill run python-example:status
```

**Example**:
```bash
$ skill run python-example:status
{
  "skill_name": "python-example",
  "version": "1.0.0",
  "config": {
    "greeting_prefix": "Hello",
    "max_items": 100
  },
  "tools_available": ["greet", "echo", "calculate", "process_list", "status"]
}
```

## Configuration

This skill supports the following configuration options:

```bash
# Set greeting prefix
skill config python-example --set greeting_prefix="Hi there"

# Set maximum items for list processing
skill config python-example --set max_items=50
```

## Building from Source

```bash
cd examples/python-skill

# Install dependencies
pip install -e ../../sdk/python[build]

# Build to WASM component
skill-sdk build

# Install in skill-engine
skill install ./python-example.wasm
```

## Development

```bash
# Run tests
skill-sdk test

# Validate project structure
skill-sdk validate

# Show skill info
skill-sdk info
```
