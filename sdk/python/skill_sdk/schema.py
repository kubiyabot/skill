"""
JSON Schema generation for Skill Engine SDK.

Generates JSON Schema from tool definitions for:
- MCP (Model Context Protocol) integration
- Documentation generation
- Input validation

Example:
    from skill_sdk.schema import generate_tool_schema, generate_mcp_tools

    tool = ToolDefinition(
        name="create-user",
        description="Create a new user",
        parameters=[
            ParameterDefinition(
                name="email",
                description="User email",
                validation=ParameterValidation(format="email")
            )
        ]
    )

    schema = generate_tool_schema(tool)
    # Returns JSON Schema object for the tool's parameters
"""

from typing import Any

from skill_sdk.types import (
    ParameterDefinition,
    ParameterType,
    SkillMetadata,
    ToolDefinition,
)


def generate_tool_schema(tool: ToolDefinition) -> dict[str, Any]:
    """
    Generate JSON Schema for a tool's parameters.

    Args:
        tool: Tool definition

    Returns:
        JSON Schema object

    Example:
        schema = generate_tool_schema(my_tool)
        # {
        #     "type": "object",
        #     "title": "create-user",
        #     "description": "Create a new user",
        #     "properties": { ... },
        #     "required": ["email"]
        # }
    """
    properties: dict[str, Any] = {}
    required: list[str] = []

    for param in tool.parameters:
        properties[param.name] = _parameter_to_schema(param)
        if param.required:
            required.append(param.name)

    schema: dict[str, Any] = {
        "type": "object",
        "title": tool.name,
        "description": tool.description,
        "properties": properties,
        "additionalProperties": False,
    }

    if required:
        schema["required"] = required

    return schema


def _parameter_to_schema(param: ParameterDefinition) -> dict[str, Any]:
    """Convert a parameter definition to JSON Schema."""
    schema: dict[str, Any] = {
        "type": _param_type_to_json_type(param.param_type),
        "description": param.description,
    }

    # Add default value
    if param.default is not None:
        schema["default"] = param.default

    # Add enum values
    if param.enum_values:
        schema["enum"] = param.enum_values

    # Add validation constraints
    if param.validation:
        v = param.validation

        if v.pattern:
            schema["pattern"] = v.pattern
        if v.min_length is not None:
            schema["minLength"] = v.min_length
        if v.max_length is not None:
            schema["maxLength"] = v.max_length
        if v.minimum is not None:
            schema["minimum"] = v.minimum
        if v.maximum is not None:
            schema["maximum"] = v.maximum
        if v.exclusive_minimum is not None:
            schema["exclusiveMinimum"] = v.exclusive_minimum
        if v.exclusive_maximum is not None:
            schema["exclusiveMaximum"] = v.exclusive_maximum
        if v.enum:
            schema["enum"] = v.enum
        if v.format:
            schema["format"] = v.format

    # Special handling for array type
    if param.param_type == ParameterType.ARRAY:
        schema["items"] = {"type": "string"}

    # Special handling for object/json type
    if param.param_type in (ParameterType.OBJECT, ParameterType.JSON):
        schema["additionalProperties"] = True

    return schema


def _param_type_to_json_type(param_type: ParameterType) -> str:
    """Convert parameter type to JSON Schema type."""
    type_mapping = {
        ParameterType.STRING: "string",
        ParameterType.INTEGER: "integer",
        ParameterType.FLOAT: "number",
        ParameterType.NUMBER: "number",
        ParameterType.BOOLEAN: "boolean",
        ParameterType.ARRAY: "array",
        ParameterType.OBJECT: "object",
        ParameterType.JSON: "object",
        ParameterType.FILE: "string",
        ParameterType.SECRET: "string",
    }
    return type_mapping.get(param_type, "string")


def generate_skill_schema(
    metadata: SkillMetadata,
    tools: list[ToolDefinition],
) -> dict[str, Any]:
    """
    Generate complete skill schema with all tools.

    Args:
        metadata: Skill metadata
        tools: List of tool definitions

    Returns:
        Complete schema object

    Example:
        schema = generate_skill_schema(metadata, tools)
        # {
        #     "$schema": "http://json-schema.org/draft-07/schema#",
        #     "title": "my-skill",
        #     "description": "My awesome skill",
        #     "version": "1.0.0",
        #     "tools": [ ... ]
        # }
    """
    return {
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": metadata.name,
        "description": metadata.description,
        "version": metadata.version,
        "author": metadata.author,
        "tools": [
            {
                "name": tool.name,
                "description": tool.description,
                "inputSchema": generate_tool_schema(tool),
            }
            for tool in tools
        ],
    }


def generate_mcp_tools(
    metadata: SkillMetadata,
    tools: list[ToolDefinition],
) -> list[dict[str, Any]]:
    """
    Generate MCP-compatible tool definitions.

    MCP (Model Context Protocol) uses a specific format for tool definitions.
    This function generates that format from skill tools.

    Args:
        metadata: Skill metadata (for tool name prefixing)
        tools: List of tool definitions

    Returns:
        Array of MCP tool definitions

    Example:
        mcp_tools = generate_mcp_tools(metadata, tools)
        # [
        #     {
        #         "name": "my-skill:create-user",
        #         "description": "Create a new user",
        #         "inputSchema": { ... }
        #     }
        # ]
    """
    return [
        {
            "name": f"{metadata.name}:{tool.name}",
            "description": tool.description,
            "inputSchema": generate_tool_schema(tool),
        }
        for tool in tools
    ]


def validate_against_schema(value: Any, schema: dict[str, Any]) -> list[str]:
    """
    Validate a value against a JSON Schema.

    Simple validation for common cases. For complex validation,
    use a dedicated library like jsonschema.

    Args:
        value: Value to validate
        schema: JSON Schema to validate against

    Returns:
        Validation errors (empty list if valid)
    """
    errors: list[str] = []
    schema_type = schema.get("type")

    if schema_type == "object" and isinstance(value, dict):
        # Check required properties
        required = schema.get("required", [])
        for prop in required:
            if prop not in value:
                errors.append(f"Missing required property: {prop}")

        # Validate each property
        properties = schema.get("properties", {})
        for key, prop_schema in properties.items():
            if key in value:
                prop_errors = validate_against_schema(value[key], prop_schema)
                errors.extend(f"{key}: {e}" for e in prop_errors)

    elif schema_type == "string" and isinstance(value, str):
        errors.extend(_validate_string(value, schema))

    elif schema_type in ("number", "integer") and isinstance(value, (int, float)):
        errors.extend(_validate_number(value, schema))

    elif schema_type == "array" and isinstance(value, list):
        items_schema = schema.get("items")
        if items_schema:
            for i, item in enumerate(value):
                item_errors = validate_against_schema(item, items_schema)
                errors.extend(f"[{i}]: {e}" for e in item_errors)

    elif value is not None:
        # Type mismatch
        actual_type = "array" if isinstance(value, list) else type(value).__name__
        if schema_type and actual_type != schema_type:
            # Allow int for number type
            if not (schema_type == "number" and isinstance(value, int)):
                errors.append(f"Expected {schema_type}, got {actual_type}")

    return errors


def _validate_string(value: str, schema: dict[str, Any]) -> list[str]:
    """Validate a string against schema constraints."""
    errors: list[str] = []

    min_length = schema.get("minLength")
    if min_length is not None and len(value) < min_length:
        errors.append(f"String too short (min: {min_length})")

    max_length = schema.get("maxLength")
    if max_length is not None and len(value) > max_length:
        errors.append(f"String too long (max: {max_length})")

    pattern = schema.get("pattern")
    if pattern:
        import re
        if not re.match(pattern, value):
            errors.append(f"String does not match pattern: {pattern}")

    enum = schema.get("enum")
    if enum and value not in enum:
        errors.append(f"Value must be one of: {', '.join(str(e) for e in enum)}")

    return errors


def _validate_number(value: float, schema: dict[str, Any]) -> list[str]:
    """Validate a number against schema constraints."""
    errors: list[str] = []

    minimum = schema.get("minimum")
    if minimum is not None and value < minimum:
        errors.append(f"Value too small (min: {minimum})")

    maximum = schema.get("maximum")
    if maximum is not None and value > maximum:
        errors.append(f"Value too large (max: {maximum})")

    exclusive_minimum = schema.get("exclusiveMinimum")
    if exclusive_minimum is not None and value <= exclusive_minimum:
        errors.append(f"Value must be greater than {exclusive_minimum}")

    exclusive_maximum = schema.get("exclusiveMaximum")
    if exclusive_maximum is not None and value >= exclusive_maximum:
        errors.append(f"Value must be less than {exclusive_maximum}")

    return errors
