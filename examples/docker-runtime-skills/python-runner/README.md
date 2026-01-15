# Python Runner Skill

Execute Python scripts in a sandboxed Docker environment.

## When to Use

Use this skill when you need to:
- Run arbitrary Python code safely
- Process data with pandas, numpy, etc.
- Execute Python scripts in isolation
- Quick data analysis tasks
- File processing with Python

**Choose this over WASM skills when:**
- You need access to Python standard library
- Your code requires pip packages
- You're working with files on disk
- You need scientific computing libraries

## Features

| Feature | Status |
|---------|--------|
| Python 3.12 | ✅ |
| pip/setuptools | ✅ |
| File system access | ✅ (current directory) |
| Network access | ⚙️ (configurable) |
| Memory limits | ✅ (512MB default) |
| Automatic cleanup | ✅ |

## Quick Start

```bash
# Install the skill
skill install ./examples/docker-runtime-skills/python-runner

# Run a simple script
skill run python-runner -- -c "print('Hello from Python!')"
```

## Usage

### Basic Commands

```bash
# Check Python version
skill run python-runner -- --version

# Run a script file
skill run python-runner -- script.py

# Run with arguments
skill run python-runner -- script.py --input data.json --output result.json

# One-liner
skill run python-runner -- -c "print('Hello from Python!')"

# Interactive shell
skill run python-runner
```

### Working with Files

```bash
# Process a JSON file
skill run python-runner -- -c "
import json
with open('data.json') as f:
    data = json.load(f)
print(f'Items: {len(data)}')
"

# Read and transform CSV
skill run python-runner -- -c "
import csv
with open('data.csv') as f:
    reader = csv.DictReader(f)
    for row in reader:
        print(row['name'])
"
```

### Using pip Packages

For pip packages, use the network-enabled variant:

```bash
# Install and use a package
skill run python-runner-pip -- -c "
import subprocess
subprocess.run(['pip', 'install', '-q', 'requests'])
import requests
print(requests.get('https://api.github.com').json())
"

# Or with pip entrypoint
skill run pip-runner -- install requests pandas numpy
```

## Configuration

### Basic (No Network)

The default configuration for sandboxed execution:

```toml
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
rm = true
```

### With Network (for pip/APIs)

Enable network access when needed:

```toml
[skills.python-runner-pip]
source = "docker:python:3.12-slim"
runtime = "docker"
description = "Python with network access"

[skills.python-runner-pip.docker]
image = "python:3.12-slim"
entrypoint = "python3"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
memory = "512m"
network = "bridge"
rm = true
```

### Custom pip Runner

For pip commands specifically:

```toml
[skills.pip-runner]
source = "docker:python:3.12-slim"
runtime = "docker"
description = "pip package manager"

[skills.pip-runner.docker]
image = "python:3.12-slim"
entrypoint = "pip"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
memory = "1g"
network = "bridge"
rm = true
```

### Data Science Variant

For heavy data processing:

```toml
[skills.python-data]
source = "docker:python:3.12-slim"
runtime = "docker"
description = "Python for data science"

[skills.python-data.docker]
image = "python:3.12-slim"
entrypoint = "python3"
volumes = ["${PWD}:/workdir"]
working_dir = "/workdir"
memory = "2g"
network = "bridge"
rm = true
```

## Security

| Control | Setting | Description |
|---------|---------|-------------|
| Network | `none` by default | No internet access unless configured |
| Memory | 512MB limit | Prevents memory exhaustion |
| Volumes | Current directory only | No access to host system |
| Image | Slim variant | Smaller attack surface (~150MB) |
| User | Non-root | Runs as unprivileged user |
| Cleanup | Auto-remove | Container deleted after execution |

### Security Best Practices

1. **Use `network = "none"` for untrusted code**
2. **Review scripts before execution**
3. **Limit volume mounts** to only what's needed
4. **Set appropriate memory limits** for data processing

## Use Cases

| Scenario | Network | Command |
|----------|---------|---------|
| Data processing | `none` | `skill run python-runner -- process.py data.csv` |
| File conversion | `none` | `skill run python-runner -- convert.py input.json` |
| API calls | `bridge` | `skill run python-runner-pip -- fetch_api.py` |
| pip install | `bridge` | `skill run pip-runner -- install pandas` |
| Data analysis | `bridge` | `skill run python-data -- analyze.py` |

## Examples

### JSON Processing

```python
# transform.py
import json
import sys

with open(sys.argv[1]) as f:
    data = json.load(f)

transformed = [
    {**item, 'processed': True}
    for item in data
]

print(json.dumps(transformed, indent=2))
```

```bash
skill run python-runner -- transform.py input.json > output.json
```

### CSV Analysis

```python
# analyze.py
import csv
import sys
from collections import Counter

with open(sys.argv[1]) as f:
    reader = csv.DictReader(f)
    data = list(reader)

print(f"Total rows: {len(data)}")
print(f"Columns: {', '.join(data[0].keys())}")

# Count by category
if 'category' in data[0]:
    counts = Counter(row['category'] for row in data)
    for cat, count in counts.most_common(5):
        print(f"  {cat}: {count}")
```

```bash
skill run python-runner -- analyze.py data.csv
```

### Data Processing Pipeline

```python
# pipeline.py
import json
import sys

def load_data(path):
    with open(path) as f:
        return json.load(f)

def transform(data):
    return [
        {
            'id': item['id'],
            'name': item['name'].upper(),
            'value': item['value'] * 2
        }
        for item in data
    ]

def save_data(data, path):
    with open(path, 'w') as f:
        json.dump(data, f, indent=2)

if __name__ == '__main__':
    input_path = sys.argv[1]
    output_path = sys.argv[2]

    data = load_data(input_path)
    transformed = transform(data)
    save_data(transformed, output_path)

    print(f"Processed {len(data)} items")
```

```bash
skill run python-runner -- pipeline.py input.json output.json
```

## Docker Image

| Property | Value |
|----------|-------|
| Image | `python:3.12-slim` |
| Size | ~150MB |
| Python | 3.12.x |
| pip | Included |
| setuptools | Included |
| OS | Debian slim |

## Troubleshooting

### "ModuleNotFoundError" Error

For standard library modules, they should be available. For pip packages, use the network variant:
```bash
skill run python-runner-pip -- -c "
import subprocess
subprocess.run(['pip', 'install', '-q', 'pandas'])
import pandas as pd
print(pd.__version__)
"
```

### Network Requests Fail

Use the network-enabled variant:
```bash
skill run python-runner-pip -- fetch-script.py
```

### Out of Memory

Increase the memory limit for data processing:
```toml
[skills.python-runner.docker]
memory = "2g"  # Increase to 2GB for pandas/numpy
```

### Permission Denied

Ensure the script is readable:
```bash
chmod +r script.py
```

### Slow Package Installation

Pre-build an image with your packages:
```dockerfile
FROM python:3.12-slim
RUN pip install pandas numpy requests
```

Then use your custom image in the manifest.

## Comparison with Node Runner

| Feature | Python Runner | Node Runner |
|---------|--------------|-------------|
| Language | Python 3.12 | Node.js 20 |
| Package Manager | pip | npm |
| Data Science | Excellent | Limited |
| Web/API | Good | Excellent |
| Image Size | ~150MB | ~50MB |
| Async | asyncio | Native |

## Related Skills

- [Node Runner](../node-runner) - JavaScript execution
- [WASM Skills](/guides/developing-skills) - For portable, sandboxed code
