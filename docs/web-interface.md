# Web Interface

The Skill Engine Web Interface provides a browser-based UI for managing and executing skills. Built with Yew (Rust WASM framework), it offers a modern, responsive experience for skill management.

## Getting Started

### Starting the Web Server

```bash
# Start on default port (3000)
skill web

# Start on custom port
skill web --port 8080

# Start with verbose logging
skill web --port 3001 -v
```

The server provides:
- **Web UI**: `http://127.0.0.1:3000/`
- **REST API**: `http://127.0.0.1:3000/api/...`

## Features

### Dashboard
- Overview of installed skills
- Execution history and statistics
- Quick access to recent actions

### Skills Browser
- List all installed skills with search and filter
- View skill details, tools, and instances
- Install new skills from Git URLs or registry

### Run Page
- Execute skill tools with dynamic parameter forms
- **Service Requirements**: Skills can declare host service dependencies
- Real-time service status indicators
- Output formatting (JSON, Raw, Pretty)

### Settings
- Configure search settings
- Manage API configuration
- View system health

## Service Requirements

Skills can declare host services they depend on. The web interface automatically handles these requirements.

### How It Works

1. **Declaration**: Skills declare service requirements in the manifest
2. **Detection**: The UI checks service status when a skill is selected
3. **Start Services**: If required services are not running, a banner appears with "Start" buttons
4. **Execution**: Once all required services are running, execution is enabled

### Example: Kubernetes Skill

The Kubernetes skill requires `kubectl proxy` to be running:

```toml
# .skill-engine.toml
[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
description = "Kubernetes cluster management"

[[skills.kubernetes.services]]
name = "kubectl-proxy"
description = "Kubernetes API proxy for executing commands"
optional = false
default_port = 8001
```

When you select the Kubernetes skill:
- If `kubectl-proxy` is not running, an amber banner appears
- Click "Start" to launch the proxy automatically
- A green indicator shows when the service is running
- The proxy URL is automatically injected into skill execution

### Defining Service Requirements

Add a `[[skills.<name>.services]]` section for each required service:

```toml
[[skills.my-skill.services]]
name = "service-name"           # Unique service identifier
description = "Human-readable description"
optional = false                # true = enhances functionality, false = required
default_port = 8080            # Default port to use
```

Fields:
- **name**: Service identifier (used for URL injection: `SERVICE_NAME_URL`)
- **description**: Shown in the UI banner
- **optional**: If `false`, execution is blocked until service is running
- **default_port**: Port to use when starting the service

### URL Injection

When a required service is running, its URL is automatically injected:
- **Environment variable**: `SERVICE_NAME_URL` (e.g., `KUBECTL_PROXY_URL`)
- **Tool argument**: `_service_name_url` (e.g., `_kubectl_proxy_url`)

## API Reference

### Skills Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/skills` | List all skills (paginated) |
| GET | `/api/skills/:name` | Get skill details |
| POST | `/api/skills` | Install a new skill |
| DELETE | `/api/skills/:name` | Uninstall a skill |

### Execution Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/execute` | Execute a skill tool |
| GET | `/api/executions` | List execution history |
| GET | `/api/executions/:id` | Get execution details |

### Service Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/services` | List all services and status |
| POST | `/api/services/start` | Start a service |
| POST | `/api/services/stop` | Stop a service |

### Example: Execute a Tool

```bash
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "skill": "kubernetes",
    "tool": "get",
    "args": {
      "resource": "pods",
      "namespace": "default"
    }
  }'
```

### Example: Start a Service

```bash
curl -X POST http://localhost:3000/api/services/start \
  -H "Content-Type: application/json" \
  -d '{
    "service": "kubectl-proxy",
    "port": 8001
  }'
```

## Skill Response Format

When fetching skill details, the response includes service requirements:

```json
{
  "name": "kubernetes",
  "version": "0.1.0",
  "description": "Kubernetes cluster management",
  "runtime": "wasm",
  "tools_count": 20,
  "required_services": [
    {
      "name": "kubectl-proxy",
      "description": "Kubernetes API proxy for executing commands",
      "optional": false,
      "default_port": 8001,
      "status": {
        "name": "kubectl-proxy",
        "running": true,
        "port": 8001,
        "url": "http://127.0.0.1:8001"
      }
    }
  ],
  "tools": [...],
  "instances": [...]
}
```

## Building the Web UI

The web UI is built with Trunk and compiled to WASM:

```bash
# Install dependencies
cargo install trunk
rustup target add wasm32-unknown-unknown

# Development build
cd crates/skill-web
trunk build

# Production build (optimized)
trunk build --release
```

## Architecture

```
skill-web/
├── src/
│   ├── api/           # API client and types
│   ├── components/    # Reusable UI components
│   ├── pages/         # Page components (Dashboard, Run, Skills, etc.)
│   ├── store/         # Yewdux state management
│   └── main.rs        # Application entry point
├── index.html         # HTML template
├── styles.css         # Tailwind CSS styles
└── Trunk.toml         # Build configuration
```

The UI uses:
- **Yew 0.21**: Rust framework for WebAssembly applications
- **Yewdux**: State management
- **Tailwind CSS**: Styling
- **Trunk**: Build tool for Rust WASM apps
