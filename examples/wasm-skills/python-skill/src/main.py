"""
Example Python skill demonstrating the Skill Engine SDK.

This skill shows how to:
- Define tools with parameters
- Use configuration
- Return different result types
- Handle errors gracefully
"""

import json
from typing import Optional

# Import from the SDK (would be installed as a package)
import sys
sys.path.insert(0, "../../sdk/python")

from skill_sdk import Skill, tool, param, config, ExecutionResult


@config("greeting_prefix", "Prefix for greeting messages", default="Hello")
@config("max_items", "Maximum items to process", default=100)
@Skill(
    name="python-example",
    description="Example Python skill demonstrating SDK features",
    version="1.0.0",
    author="Skill Engine Team",
    tags=["example", "python", "demo"],
)
class PythonExampleSkill:
    """Example skill showing Python SDK capabilities."""

    def __init__(self):
        """Initialize the skill with configuration."""
        self.greeting_prefix = self.get_config("greeting_prefix", "Hello")
        self.max_items = int(self.get_config("max_items", 100))

    @tool(description="Greet someone with a personalized message")
    @param("name", "The name of the person to greet")
    @param("formal", "Use formal greeting style")
    def greet(self, name: str = "World", formal: bool = False) -> str:
        """Generate a greeting message."""
        if formal:
            return f"Good day, {name}. How may I assist you?"
        return f"{self.greeting_prefix}, {name}!"

    @tool(description="Echo back a message with optional transformation")
    @param("message", "The message to echo")
    @param("uppercase", "Convert message to uppercase")
    @param("reverse", "Reverse the message")
    def echo(
        self,
        message: str,
        uppercase: bool = False,
        reverse: bool = False,
    ) -> str:
        """Echo a message with optional transformations."""
        result = message
        if uppercase:
            result = result.upper()
        if reverse:
            result = result[::-1]
        return result

    @tool(description="Perform basic arithmetic operations")
    @param("a", "First number")
    @param("b", "Second number")
    @param("operation", "Operation to perform: add, subtract, multiply, divide")
    def calculate(
        self,
        a: float,
        b: float,
        operation: str = "add",
    ) -> ExecutionResult:
        """Perform a calculation on two numbers."""
        operations = {
            "add": lambda x, y: x + y,
            "subtract": lambda x, y: x - y,
            "multiply": lambda x, y: x * y,
            "divide": lambda x, y: x / y if y != 0 else None,
        }

        if operation not in operations:
            return ExecutionResult.error(
                f"Unknown operation: {operation}. Valid: {', '.join(operations.keys())}"
            )

        result = operations[operation](a, b)

        if result is None:
            return ExecutionResult.error("Division by zero")

        return ExecutionResult.ok(
            f"{a} {operation} {b} = {result}",
            data={"result": result, "operation": operation, "a": a, "b": b}
        )

    @tool(description="Process a list of items and return statistics")
    @param("items", "JSON array of numbers to process")
    def process_list(self, items: str) -> ExecutionResult:
        """Process a list of items and return statistics."""
        try:
            data = json.loads(items)

            if not isinstance(data, list):
                return ExecutionResult.error("Input must be a JSON array")

            if len(data) > self.max_items:
                return ExecutionResult.error(
                    f"Too many items ({len(data)}). Maximum: {self.max_items}"
                )

            # Calculate statistics
            numbers = [float(x) for x in data if isinstance(x, (int, float))]

            if not numbers:
                return ExecutionResult.error("No valid numbers in input")

            stats = {
                "count": len(numbers),
                "sum": sum(numbers),
                "min": min(numbers),
                "max": max(numbers),
                "average": sum(numbers) / len(numbers),
            }

            return ExecutionResult.ok(
                f"Processed {len(numbers)} items",
                data=stats
            )

        except json.JSONDecodeError as e:
            return ExecutionResult.error(f"Invalid JSON: {e}")
        except ValueError as e:
            return ExecutionResult.error(f"Invalid number: {e}")

    @tool(description="Get current skill configuration and status")
    def status(self) -> dict:
        """Return skill status and configuration."""
        return {
            "skill_name": "python-example",
            "version": "1.0.0",
            "config": {
                "greeting_prefix": self.greeting_prefix,
                "max_items": self.max_items,
            },
            "tools_available": [
                "greet",
                "echo",
                "calculate",
                "process_list",
                "status",
            ],
        }


if __name__ == "__main__":
    # This allows running the skill directly for testing
    PythonExampleSkill.run()
