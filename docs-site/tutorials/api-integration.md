# API Integration Tutorial

Learn how to build a skill that integrates with external APIs. This tutorial creates a weather skill that fetches data from a public API.

**Time:** 20 minutes

## What You'll Build

A "weather" skill that:
- Fetches current weather for a city
- Handles API errors gracefully
- Uses configuration for API keys

## Prerequisites

- Skill CLI installed
- Node.js 18+ installed
- Basic JavaScript knowledge
- An API key from [OpenWeatherMap](https://openweathermap.org/api) (free tier available)

## Step 1: Create the Project

```bash
mkdir weather-skill
cd weather-skill
```

## Step 2: Create the Skill

Create `skill.js`:

```javascript
// weather-skill/skill.js

// Store config globally (set during validateConfig)
let config = {};

export function getMetadata() {
  return {
    name: "weather",
    version: "1.0.0",
    description: "Get current weather information for any city",
    author: "Your Name"
  };
}

export function getTools() {
  return [
    {
      name: "current",
      description: "Get current weather for a city",
      parameters: [
        {
          name: "city",
          paramType: "string",
          description: "City name (e.g., 'London', 'New York')",
          required: true
        },
        {
          name: "units",
          paramType: "string",
          description: "Temperature units: 'metric' (Celsius) or 'imperial' (Fahrenheit)",
          required: false
        }
      ]
    },
    {
      name: "forecast",
      description: "Get weather forecast for a city",
      parameters: [
        {
          name: "city",
          paramType: "string",
          description: "City name",
          required: true
        },
        {
          name: "days",
          paramType: "integer",
          description: "Number of days (1-5)",
          required: false
        }
      ]
    }
  ];
}

export async function executeTool(toolName, argsJson) {
  const args = JSON.parse(argsJson);

  try {
    if (toolName === "current") {
      return await getCurrentWeather(args);
    }

    if (toolName === "forecast") {
      return await getForecast(args);
    }

    return {
      success: false,
      output: "",
      errorMessage: `Unknown tool: ${toolName}`
    };
  } catch (error) {
    return {
      success: false,
      output: "",
      errorMessage: `API error: ${error.message}`
    };
  }
}

async function getCurrentWeather(args) {
  const { city, units = "metric" } = args;
  const apiKey = config.api_key;

  if (!apiKey) {
    return {
      success: false,
      output: "",
      errorMessage: "API key not configured. Run: skill config weather"
    };
  }

  const url = `https://api.openweathermap.org/data/2.5/weather?q=${encodeURIComponent(city)}&units=${units}&appid=${apiKey}`;

  const response = await fetch(url);

  if (!response.ok) {
    if (response.status === 404) {
      return {
        success: false,
        output: "",
        errorMessage: `City not found: ${city}`
      };
    }
    throw new Error(`HTTP ${response.status}`);
  }

  const data = await response.json();
  const tempUnit = units === "metric" ? "°C" : "°F";

  const output = `
Weather in ${data.name}, ${data.sys.country}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Temperature: ${data.main.temp}${tempUnit}
Feels like:  ${data.main.feels_like}${tempUnit}
Conditions:  ${data.weather[0].description}
Humidity:    ${data.main.humidity}%
Wind:        ${data.wind.speed} m/s
`.trim();

  return {
    success: true,
    output: output + "\n",
    errorMessage: null
  };
}

async function getForecast(args) {
  const { city, days = 3 } = args;
  const apiKey = config.api_key;

  if (!apiKey) {
    return {
      success: false,
      output: "",
      errorMessage: "API key not configured. Run: skill config weather"
    };
  }

  const url = `https://api.openweathermap.org/data/2.5/forecast?q=${encodeURIComponent(city)}&units=metric&cnt=${days * 8}&appid=${apiKey}`;

  const response = await fetch(url);

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }

  const data = await response.json();

  let output = `Forecast for ${data.city.name}\n`;
  output += "━".repeat(40) + "\n";

  // Group by day
  const dailyData = {};
  for (const item of data.list) {
    const date = item.dt_txt.split(" ")[0];
    if (!dailyData[date]) {
      dailyData[date] = item;
    }
  }

  for (const [date, item] of Object.entries(dailyData).slice(0, days)) {
    output += `${date}: ${item.main.temp}°C - ${item.weather[0].description}\n`;
  }

  return {
    success: true,
    output: output,
    errorMessage: null
  };
}

export function validateConfig(configJson) {
  if (!configJson) {
    return { ok: null }; // No config is OK, will prompt when needed
  }

  try {
    config = JSON.parse(configJson);

    if (!config.api_key) {
      return {
        ok: null,
        error: "Missing 'api_key' in configuration"
      };
    }

    return { ok: null };
  } catch (e) {
    return {
      ok: null,
      error: `Invalid config: ${e.message}`
    };
  }
}
```

## Step 3: Create SKILL.md

```markdown
---
name: weather
description: Get current weather and forecasts for any city worldwide
---

# Weather Skill

Fetch real-time weather data from OpenWeatherMap API.

## Configuration

This skill requires an OpenWeatherMap API key:

\`\`\`bash
skill config weather
\`\`\`

Get a free API key at: https://openweathermap.org/api

## Tools

### current

Get current weather conditions for a city.

**Parameters:**
- \`city\` (required, string): City name (e.g., "London", "New York")
- \`units\` (optional, string): "metric" for Celsius, "imperial" for Fahrenheit

**Example:**
\`\`\`bash
skill run weather:current city="Tokyo"
skill run weather:current city="Paris" units="metric"
\`\`\`

### forecast

Get weather forecast for upcoming days.

**Parameters:**
- \`city\` (required, string): City name
- \`days\` (optional, integer): Number of days (1-5, default: 3)

**Example:**
\`\`\`bash
skill run weather:forecast city="Berlin" days=5
\`\`\`
```

## Step 4: Install and Configure

Install the skill:
```bash
skill install .
```

Configure your API key:
```bash
skill config weather
```

Enter your OpenWeatherMap API key when prompted.

## Step 5: Test the Skill

```bash
# Get current weather
skill run weather:current city="London"

# Get forecast
skill run weather:forecast city="Tokyo" days=3
```

## Key Concepts

### Configuration Management

Skills can require configuration (like API keys):

```javascript
export function validateConfig(configJson) {
  config = JSON.parse(configJson);
  if (!config.api_key) {
    return { error: "Missing api_key" };
  }
  return { ok: null };
}
```

Users configure with `skill config <skill-name>`.

### Error Handling

Always handle API errors gracefully:

```javascript
try {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
} catch (error) {
  return {
    success: false,
    output: "",
    errorMessage: `API error: ${error.message}`
  };
}
```

### Async Operations

Use `async/await` for API calls:

```javascript
export async function executeTool(toolName, argsJson) {
  // async operations work naturally
  const response = await fetch(url);
}
```

## Best Practices

### 1. Validate Input

Check required parameters exist:

```javascript
if (!args.city) {
  return {
    success: false,
    errorMessage: "City is required"
  };
}
```

### 2. Handle Rate Limits

Respect API rate limits and inform users:

```javascript
if (response.status === 429) {
  return {
    success: false,
    errorMessage: "Rate limit exceeded. Please try again later."
  };
}
```

### 3. Format Output for Humans

Make output readable:

```javascript
const output = `
Temperature: ${temp}°C
Conditions:  ${conditions}
`.trim();
```

### 4. Secure API Keys

- Never log API keys
- Use `skill config` for secure storage
- Don't commit keys to git

## Next Steps

- Add caching to reduce API calls
- Support more weather providers
- Add weather alerts
- Learn about [testing skills](./testing-skills)

## Troubleshooting

**"API key not configured" error:**
- Run `skill config weather` and enter your key

**"City not found" error:**
- Check spelling
- Try with country code: "London,UK"

**Rate limit errors:**
- Free tier: 60 calls/minute
- Wait and try again
