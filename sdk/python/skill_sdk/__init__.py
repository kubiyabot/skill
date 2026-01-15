"""
Skill Engine SDK for Python

A Python SDK for building Skill Engine plugins with the WASM Component Model.

Example:
    from skill_sdk import Skill, tool, param, ExecutionResult
    from skill_sdk.http import create_authenticated_client

    @Skill(
        name="my-skill",
        description="My awesome skill",
        version="1.0.0"
    )
    class MySkill:
        @tool(description="Greet someone")
        @param("name", "Name to greet", min_length=1, max_length=100)
        def hello(self, name: str = "World") -> ExecutionResult:
            return ExecutionResult.ok(f"Hello, {name}!")

        @tool(description="Create a user")
        @param("email", "User's email", format="email")
        @param("age", "User's age", minimum=0, maximum=150)
        @param("role", "User's role", enum=["admin", "user", "guest"])
        def create_user(self, email: str, age: int, role: str) -> ExecutionResult:
            return ExecutionResult.ok(f"Created user {email}")

    if __name__ == "__main__":
        MySkill.run()
"""

from skill_sdk.skill import Skill, tool, param, config
from skill_sdk.types import (
    ExecutionResult,
    ToolDefinition,
    ParameterDefinition,
    ParameterType,
    ParameterValidation,
    SkillMetadata,
    ErrorCode,
    SkillError as SkillErrorType,
)
from skill_sdk.exceptions import (
    SkillError,
    ConfigurationError,
    ValidationError,
    ExecutionError,
)

# HTTP client utilities
from skill_sdk.http import (
    SkillHttpClient,
    HttpResponse,
    create_authenticated_client,
    fetch_json,
    is_rate_limited,
    get_retry_after,
)

# Validation utilities
from skill_sdk.validation import (
    validate_parameter,
    validate_string,
    validate_number,
    validate_format,
    validate_email,
    validate_url,
    validate_uuid,
)

# Schema generation
from skill_sdk.schema import (
    generate_tool_schema,
    generate_skill_schema,
    generate_mcp_tools,
)

__version__ = "0.1.0"

__all__ = [
    # Core decorators
    "Skill",
    "tool",
    "param",
    "config",
    # Types
    "ExecutionResult",
    "ToolDefinition",
    "ParameterDefinition",
    "ParameterType",
    "ParameterValidation",
    "SkillMetadata",
    "ErrorCode",
    "SkillErrorType",
    # Exceptions
    "SkillError",
    "ConfigurationError",
    "ValidationError",
    "ExecutionError",
    # HTTP Client
    "SkillHttpClient",
    "HttpResponse",
    "create_authenticated_client",
    "fetch_json",
    "is_rate_limited",
    "get_retry_after",
    # Validation
    "validate_parameter",
    "validate_string",
    "validate_number",
    "validate_format",
    "validate_email",
    "validate_url",
    "validate_uuid",
    # Schema
    "generate_tool_schema",
    "generate_skill_schema",
    "generate_mcp_tools",
]
