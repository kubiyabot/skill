"""
Exception classes for the Skill Engine SDK.
"""


class SkillError(Exception):
    """Base exception for all skill-related errors."""

    def __init__(self, message: str, code: str = "SKILL_ERROR"):
        self.message = message
        self.code = code
        super().__init__(message)


class ConfigurationError(SkillError):
    """Raised when there is a configuration error."""

    def __init__(self, message: str):
        super().__init__(message, code="CONFIGURATION_ERROR")


class ValidationError(SkillError):
    """Raised when input validation fails."""

    def __init__(self, message: str, field: str | None = None):
        self.field = field
        super().__init__(message, code="VALIDATION_ERROR")


class ExecutionError(SkillError):
    """Raised when tool execution fails."""

    def __init__(self, message: str, tool: str | None = None):
        self.tool = tool
        super().__init__(message, code="EXECUTION_ERROR")


class ToolNotFoundError(SkillError):
    """Raised when a requested tool is not found."""

    def __init__(self, tool_name: str):
        self.tool_name = tool_name
        super().__init__(f"Tool not found: {tool_name}", code="TOOL_NOT_FOUND")


class ParameterError(ValidationError):
    """Raised when a parameter is invalid or missing."""

    def __init__(self, message: str, parameter: str):
        self.parameter = parameter
        super().__init__(message, field=parameter)
