---
# SKILL.md Tutorial Example: Weather Skill
# =========================================
#
# This file demonstrates the SKILL.md format for native skills.
# Native skills wrap CLI tools and make them accessible to AI agents.
#
# The frontmatter (between ---) contains metadata about the skill.
# Required fields: name, description
# Optional fields: allowed-tools, version, author

name: weather
description: Get weather information for any city. Demonstrates API integration patterns.
version: 1.0.0
author: Skill Engine Team

# allowed-tools specifies which tools can be executed.
# - Bash: Allows shell commands (curl, jq, etc.)
# - Other options: kubectl, git, docker, etc.
allowed-tools: Bash
---

# Weather Skill

This skill demonstrates how to build an API integration skill using the SKILL.md format.

## Overview

The weather skill fetches weather data from a public API and formats it for display.
It shows patterns for:
- API integration with curl
- JSON parsing with jq
- Error handling
- Optional parameters with defaults

## When to Use

Use this skill when you need to:
- Check current weather for a location
- Get weather forecasts
- Integrate weather data into workflows

## Tools

<!--
Tool definitions start with ### <tool-name>
The description follows the heading.
Parameters are listed in a specific format.
-->

### current

Get current weather for a city.

This tool fetches real-time weather data including temperature, conditions, and humidity.

<!--
Parameters use this format:
- `name` (required|optional, type): Description
Types: string, integer, number, boolean, array
-->

**Parameters:**
- `city` (required, string): City name (e.g., "London", "New York", "Tokyo")
- `units` (optional, string): Temperature units - "metric" for Celsius, "imperial" for Fahrenheit (default: metric)

**Example:**
```bash
# Basic usage
skill run weather:current city="London"

# With units
skill run weather:current city="Paris" units="metric"
skill run weather:current city="New York" units="imperial"
```

### forecast

Get weather forecast for upcoming days.

Returns a multi-day weather forecast for the specified location.

**Parameters:**
- `city` (required, string): City name
- `days` (optional, integer): Number of days to forecast (1-5, default: 3)

**Example:**
```bash
# 3-day forecast (default)
skill run weather:forecast city="Berlin"

# 5-day forecast
skill run weather:forecast city="Tokyo" days=5
```

## Implementation Notes

<!--
This section documents how the skill works internally.
It's helpful for maintainers and contributors.
-->

This skill uses:
- **curl**: For HTTP requests to weather API
- **jq**: For JSON parsing and formatting
- **Environment variable**: `WEATHER_API_KEY` for authentication

The API used is OpenWeatherMap (https://openweathermap.org/api).
You'll need to configure your API key before using this skill.

## Configuration

<!--
Document any configuration requirements here.
This helps users set up the skill correctly.
-->

Set your OpenWeatherMap API key:
```bash
export WEATHER_API_KEY="your-api-key-here"
```

Or use skill config:
```bash
skill config weather
```
