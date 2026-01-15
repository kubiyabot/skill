"""
CLI tools for the Skill Engine SDK.
"""

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Optional

import typer
from rich.console import Console
from rich.panel import Panel
from rich.syntax import Syntax
from rich.table import Table

app = typer.Typer(
    name="skill-sdk",
    help="Skill Engine SDK - Build and manage skill plugins",
    add_completion=False,
)
console = Console()


@app.command()
def init(
    name: str = typer.Argument(..., help="Name of the skill to create"),
    directory: Optional[str] = typer.Option(None, "--dir", "-d", help="Directory to create skill in"),
    template: str = typer.Option("basic", "--template", "-t", help="Template to use (basic, advanced, api)"),
) -> None:
    """Initialize a new skill project."""
    skill_dir = Path(directory or name)

    if skill_dir.exists():
        console.print(f"[red]Error:[/red] Directory '{skill_dir}' already exists")
        raise typer.Exit(1)

    # Create directory structure
    skill_dir.mkdir(parents=True)
    (skill_dir / "src").mkdir()
    (skill_dir / "tests").mkdir()

    # Create skill.yaml
    skill_yaml = f"""name: {name}
version: 0.1.0
description: {name} skill
author: Your Name

component_model: true
wit_interface: skill-engine:interface@1.0.0

config:
  # Add configuration fields here
  # api_key:
  #   type: string
  #   required: true
  #   secret: true

tools:
  - name: hello
    description: Greet someone
    handler: src/main.py:hello
"""
    (skill_dir / "skill.yaml").write_text(skill_yaml)

    # Create main Python file based on template
    if template == "basic":
        main_py = _generate_basic_template(name)
    elif template == "advanced":
        main_py = _generate_advanced_template(name)
    elif template == "api":
        main_py = _generate_api_template(name)
    else:
        main_py = _generate_basic_template(name)

    (skill_dir / "src" / "main.py").write_text(main_py)

    # Create __init__.py
    (skill_dir / "src" / "__init__.py").write_text("")

    # Create test file
    test_py = f'''"""Tests for {name} skill."""

import pytest
from src.main import {_to_class_name(name)}


def test_hello():
    """Test the hello tool."""
    skill = {_to_class_name(name)}()
    result = skill.hello(name="Test")
    assert "Test" in result
'''
    (skill_dir / "tests" / f"test_{name.replace('-', '_')}.py").write_text(test_py)

    # Create pyproject.toml
    pyproject = f"""[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "{name}"
version = "0.1.0"
description = "{name} skill for Skill Engine"
requires-python = ">=3.10"
dependencies = [
    "skill-engine-sdk>=0.1.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0",
    "pytest-asyncio>=0.21",
]
"""
    (skill_dir / "pyproject.toml").write_text(pyproject)

    # Create SKILL.md
    skill_md = _generate_skill_md(name)
    (skill_dir / "SKILL.md").write_text(skill_md)

    # Create .gitignore
    gitignore = """__pycache__/
*.py[cod]
*$py.class
.pytest_cache/
dist/
build/
*.egg-info/
.venv/
.env
"""
    (skill_dir / ".gitignore").write_text(gitignore)

    console.print(Panel.fit(
        f"[green]Created skill project:[/green] {skill_dir}\n\n"
        f"[cyan]Next steps:[/cyan]\n"
        f"  1. cd {skill_dir}\n"
        f"  2. Edit src/main.py to add your tools\n"
        f"  3. Edit SKILL.md to document your skill\n"
        f"  4. Run: skill-sdk build\n"
        f"  5. Run: skill install ./{name}.wasm",
        title="Skill Created",
    ))


@app.command()
def build(
    path: str = typer.Argument(".", help="Path to skill project"),
    output: Optional[str] = typer.Option(None, "--output", "-o", help="Output path for .wasm file"),
) -> None:
    """Build a skill project into a WASM component."""
    skill_path = Path(path)

    if not (skill_path / "skill.yaml").exists():
        console.print("[red]Error:[/red] No skill.yaml found. Is this a skill project?")
        raise typer.Exit(1)

    # Read skill.yaml
    import yaml
    with open(skill_path / "skill.yaml") as f:
        skill_config = yaml.safe_load(f)

    skill_name = skill_config.get("name", "skill")
    output_path = output or f"{skill_name}.wasm"

    console.print(f"[cyan]Building skill:[/cyan] {skill_name}")

    # Check if componentize-py is installed
    try:
        subprocess.run(
            ["componentize-py", "--version"],
            capture_output=True,
            check=True
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        console.print(
            "[yellow]Warning:[/yellow] componentize-py not found.\n"
            "Install it with: pip install componentize-py"
        )
        raise typer.Exit(1)

    # Build with componentize-py
    console.print("[cyan]Compiling to WASM Component...[/cyan]")

    try:
        result = subprocess.run(
            [
                "componentize-py",
                "-d", str(skill_path / "wit"),
                "-w", "skill",
                "componentize",
                str(skill_path / "src" / "main.py"),
                "-o", output_path,
            ],
            capture_output=True,
            text=True,
        )

        if result.returncode != 0:
            console.print(f"[red]Build failed:[/red]\n{result.stderr}")
            raise typer.Exit(1)

        console.print(f"[green]Build successful![/green] Output: {output_path}")

    except Exception as e:
        console.print(f"[red]Build error:[/red] {e}")
        raise typer.Exit(1)


@app.command()
def test(
    path: str = typer.Argument(".", help="Path to skill project"),
) -> None:
    """Run tests for a skill project."""
    skill_path = Path(path)

    console.print("[cyan]Running tests...[/cyan]")

    result = subprocess.run(
        [sys.executable, "-m", "pytest", "-v", str(skill_path / "tests")],
    )

    raise typer.Exit(result.returncode)


@app.command()
def validate(
    path: str = typer.Argument(".", help="Path to skill project"),
) -> None:
    """Validate a skill project structure and configuration."""
    skill_path = Path(path)
    errors: list[str] = []
    warnings: list[str] = []

    # Check required files
    required_files = ["skill.yaml", "src/main.py", "SKILL.md"]
    for file in required_files:
        if not (skill_path / file).exists():
            errors.append(f"Missing required file: {file}")

    # Check skill.yaml
    if (skill_path / "skill.yaml").exists():
        import yaml
        try:
            with open(skill_path / "skill.yaml") as f:
                config = yaml.safe_load(f)

            required_fields = ["name", "version", "description"]
            for field in required_fields:
                if field not in config:
                    errors.append(f"skill.yaml missing required field: {field}")

            # Check tools
            if "tools" not in config or not config["tools"]:
                warnings.append("No tools defined in skill.yaml")

        except yaml.YAMLError as e:
            errors.append(f"Invalid skill.yaml: {e}")

    # Check SKILL.md
    if (skill_path / "SKILL.md").exists():
        content = (skill_path / "SKILL.md").read_text()
        if "---" not in content:
            warnings.append("SKILL.md missing YAML frontmatter")

    # Display results
    if errors:
        console.print("[red]Validation failed:[/red]")
        for error in errors:
            console.print(f"  [red]✗[/red] {error}")
    else:
        console.print("[green]Validation passed![/green]")

    if warnings:
        console.print("\n[yellow]Warnings:[/yellow]")
        for warning in warnings:
            console.print(f"  [yellow]![/yellow] {warning}")

    if errors:
        raise typer.Exit(1)


@app.command()
def info(
    path: str = typer.Argument(".", help="Path to skill project"),
) -> None:
    """Show information about a skill project."""
    skill_path = Path(path)

    if not (skill_path / "skill.yaml").exists():
        console.print("[red]Error:[/red] No skill.yaml found")
        raise typer.Exit(1)

    import yaml
    with open(skill_path / "skill.yaml") as f:
        config = yaml.safe_load(f)

    table = Table(title=f"Skill: {config.get('name', 'Unknown')}")
    table.add_column("Property", style="cyan")
    table.add_column("Value")

    table.add_row("Name", config.get("name", "N/A"))
    table.add_row("Version", config.get("version", "N/A"))
    table.add_row("Description", config.get("description", "N/A"))
    table.add_row("Author", config.get("author", "N/A"))

    tools = config.get("tools", [])
    table.add_row("Tools", ", ".join(t.get("name", "") for t in tools))

    console.print(table)


def _to_class_name(name: str) -> str:
    """Convert skill name to class name."""
    return "".join(word.capitalize() for word in name.replace("-", "_").split("_"))


def _generate_basic_template(name: str) -> str:
    """Generate basic skill template."""
    class_name = _to_class_name(name)
    return f'''"""
{name} skill for Skill Engine.
"""

from skill_sdk import Skill, tool, param


@Skill(
    name="{name}",
    description="{name} - A basic skill",
    version="0.1.0",
)
class {class_name}:
    """Basic skill implementation."""

    @tool(description="Greet someone by name")
    @param("name", "The name to greet")
    def hello(self, name: str = "World") -> str:
        """Say hello to someone."""
        return f"Hello, {{name}}!"

    @tool(description="Echo back a message")
    @param("message", "The message to echo")
    def echo(self, message: str) -> str:
        """Echo a message."""
        return message


if __name__ == "__main__":
    {class_name}.run()
'''


def _generate_advanced_template(name: str) -> str:
    """Generate advanced skill template with config."""
    class_name = _to_class_name(name)
    return f'''"""
{name} skill for Skill Engine.
"""

from skill_sdk import Skill, tool, param, config, ExecutionResult


@config("api_key", "API key for external service", required=True, secret=True)
@config("endpoint", "API endpoint URL", default="https://api.example.com")
@Skill(
    name="{name}",
    description="{name} - An advanced skill with configuration",
    version="0.1.0",
)
class {class_name}:
    """Advanced skill with configuration support."""

    def __init__(self):
        """Initialize the skill."""
        self.api_key = self.get_config("api_key")
        self.endpoint = self.get_config("endpoint")

    @tool(description="Make an API request")
    @param("path", "API path to request")
    @param("method", "HTTP method (GET, POST, etc.)")
    def api_request(self, path: str, method: str = "GET") -> ExecutionResult:
        """Make an API request to the configured endpoint."""
        url = f"{{self.endpoint}}/{{path}}"

        # TODO: Implement actual API request
        return ExecutionResult.ok(
            f"Would request {{method}} {{url}}",
            data={{"url": url, "method": method}}
        )

    @tool(description="Get current configuration")
    def get_settings(self) -> dict:
        """Return current skill settings."""
        return {{
            "endpoint": self.endpoint,
            "has_api_key": bool(self.api_key),
        }}


if __name__ == "__main__":
    {class_name}.run()
'''


def _generate_api_template(name: str) -> str:
    """Generate API-focused skill template."""
    class_name = _to_class_name(name)
    return f'''"""
{name} API skill for Skill Engine.
"""

import json
from typing import Any, Optional

from skill_sdk import Skill, tool, param, config, ExecutionResult


@config("api_key", "API key", required=True, secret=True)
@config("base_url", "Base URL for the API", default="https://api.example.com/v1")
@config("timeout", "Request timeout in seconds", default=30)
@Skill(
    name="{name}",
    description="{name} - API integration skill",
    version="0.1.0",
    tags=["api", "integration"],
)
class {class_name}:
    """API integration skill."""

    def __init__(self):
        """Initialize API client."""
        self.api_key = self.get_config("api_key")
        self.base_url = self.get_config("base_url")
        self.timeout = int(self.get_config("timeout", 30))

    def _make_request(
        self,
        method: str,
        endpoint: str,
        data: Optional[dict] = None,
    ) -> dict[str, Any]:
        """Make an HTTP request to the API."""
        url = f"{{self.base_url}}/{{endpoint}}"

        # In a real implementation, use httpx or requests
        # For now, return a mock response
        return {{
            "url": url,
            "method": method,
            "data": data,
            "status": "mock",
        }}

    @tool(description="List resources from the API")
    @param("resource", "Resource type to list")
    @param("limit", "Maximum number of results")
    def list_resources(self, resource: str, limit: int = 10) -> ExecutionResult:
        """List resources from the API."""
        try:
            result = self._make_request("GET", f"{{resource}}?limit={{limit}}")
            return ExecutionResult.ok(
                f"Listed {{resource}}",
                data=result
            )
        except Exception as e:
            return ExecutionResult.error(str(e))

    @tool(description="Get a specific resource by ID")
    @param("resource", "Resource type")
    @param("resource_id", "Resource ID")
    def get_resource(self, resource: str, resource_id: str) -> ExecutionResult:
        """Get a specific resource by ID."""
        try:
            result = self._make_request("GET", f"{{resource}}/{{resource_id}}")
            return ExecutionResult.ok(
                f"Got {{resource}} {{resource_id}}",
                data=result
            )
        except Exception as e:
            return ExecutionResult.error(str(e))

    @tool(description="Create a new resource")
    @param("resource", "Resource type")
    @param("data", "Resource data as JSON string")
    def create_resource(self, resource: str, data: str) -> ExecutionResult:
        """Create a new resource."""
        try:
            parsed_data = json.loads(data)
            result = self._make_request("POST", resource, data=parsed_data)
            return ExecutionResult.ok(
                f"Created {{resource}}",
                data=result
            )
        except json.JSONDecodeError:
            return ExecutionResult.error("Invalid JSON data")
        except Exception as e:
            return ExecutionResult.error(str(e))


if __name__ == "__main__":
    {class_name}.run()
'''


def _generate_skill_md(name: str) -> str:
    """Generate SKILL.md template."""
    return f'''---
name: {name}
description: {name} skill for Skill Engine
allowed-tools:
  - Read
  - Bash
---

# {name.replace("-", " ").title()}

Description of what this skill does.

## When to Use

- Use case 1
- Use case 2

## Tools Provided

### hello
Greet someone by name.

**Usage**:
```bash
skill run {name}:hello --name "World"
```

**Parameters**:
- `name` (optional): Name to greet (defaults to "World")

**Example**:
```bash
skill run {name}@default:hello --name "Alice"
```

## Configuration

This skill supports the following configuration:

```bash
# Set API key (if needed)
skill config {name} --set api_key=YOUR_API_KEY
```

## Examples

### Basic greeting
```bash
skill run {name}@default:hello
```
'''


if __name__ == "__main__":
    app()
