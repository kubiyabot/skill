"""
Runtime execution support for skills.
"""

import json
import sys
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from skill_sdk.skill import SkillInstance


def run_skill(skill: "SkillInstance") -> None:
    """
    Run a skill in CLI mode.

    This reads commands from stdin and writes results to stdout,
    compatible with the skill-engine runtime protocol.
    """
    print(f"Skill '{skill.metadata.name}' v{skill.metadata.version} loaded", file=sys.stderr)
    print(f"Tools: {', '.join(skill.tools.keys())}", file=sys.stderr)

    # Validate config
    config_errors = skill.validate_config()
    if config_errors:
        print(f"Warning: Configuration issues: {', '.join(config_errors)}", file=sys.stderr)

    # Process commands from stdin
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            command = json.loads(line)
            response = process_command(skill, command)
            print(json.dumps(response))
            sys.stdout.flush()
        except json.JSONDecodeError as e:
            print(json.dumps({
                "error": f"Invalid JSON: {e}",
                "success": False
            }))
            sys.stdout.flush()


def process_command(skill: "SkillInstance", command: dict) -> dict:
    """Process a single command and return the response."""
    cmd_type = command.get("type", "")

    if cmd_type == "get_metadata":
        return {
            "success": True,
            "metadata": skill.get_metadata()
        }

    elif cmd_type == "get_tools":
        return {
            "success": True,
            "tools": skill.get_tools()
        }

    elif cmd_type == "execute":
        tool_name = command.get("tool", "")
        args = command.get("args", {})

        result = skill.execute_tool(tool_name, args)
        return result.to_dict()

    elif cmd_type == "validate_config":
        errors = skill.validate_config()
        return {
            "success": len(errors) == 0,
            "errors": errors
        }

    else:
        return {
            "success": False,
            "error": f"Unknown command type: {cmd_type}"
        }


# WASI Component Model exports
# These functions are called by the skill-engine runtime

def get_metadata() -> str:
    """Get skill metadata (Component Model export)."""
    from skill_sdk.skill import get_current_skill
    skill = get_current_skill()
    if skill:
        return json.dumps(skill.get_metadata())
    return "{}"


def get_tools() -> str:
    """Get tool definitions (Component Model export)."""
    from skill_sdk.skill import get_current_skill
    skill = get_current_skill()
    if skill:
        return json.dumps(skill.get_tools())
    return "[]"


def execute_tool(tool_name: str, args_json: str) -> str:
    """Execute a tool (Component Model export)."""
    from skill_sdk.skill import get_current_skill
    skill = get_current_skill()
    if not skill:
        return json.dumps({"success": False, "error": "No skill loaded"})

    try:
        args = json.loads(args_json) if args_json else {}
        result = skill.execute_tool(tool_name, args)
        return json.dumps(result.to_dict())
    except Exception as e:
        return json.dumps({"success": False, "error": str(e)})


def validate_config() -> str:
    """Validate configuration (Component Model export)."""
    from skill_sdk.skill import get_current_skill
    skill = get_current_skill()
    if not skill:
        return json.dumps({"success": False, "errors": ["No skill loaded"]})

    errors = skill.validate_config()
    return json.dumps({"success": len(errors) == 0, "errors": errors})
