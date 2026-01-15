"""
Type definitions for the Skill Engine SDK.

This module provides type definitions that match the WIT interface
with additional validation and error handling support.
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Callable, Optional, Literal


class ParameterType(str, Enum):
    """Supported parameter types for tool definitions."""
    STRING = "string"
    INTEGER = "integer"
    FLOAT = "float"
    NUMBER = "number"  # Alias for float
    BOOLEAN = "boolean"
    ARRAY = "array"
    OBJECT = "object"
    JSON = "json"       # Alias for object
    FILE = "file"       # File path
    SECRET = "secret"   # Sensitive value (masked in logs)


# String format validators
StringFormat = Literal[
    "email",
    "url",
    "uri",
    "date",
    "datetime",
    "time",
    "uuid",
    "hostname",
    "ipv4",
    "ipv6",
]


@dataclass
class ParameterValidation:
    """Validation constraints for a parameter."""
    # String validations
    pattern: Optional[str] = None
    min_length: Optional[int] = None
    max_length: Optional[int] = None
    format: Optional[StringFormat] = None

    # Number validations
    minimum: Optional[float] = None
    maximum: Optional[float] = None
    exclusive_minimum: Optional[float] = None
    exclusive_maximum: Optional[float] = None

    # Enum validation
    enum: Optional[list[str]] = None


@dataclass
class ParameterDefinition:
    """Definition of a tool parameter with validation support."""
    name: str
    description: str
    param_type: ParameterType = ParameterType.STRING
    required: bool = True
    default: Optional[Any] = None
    enum_values: Optional[list[str]] = None
    validation: Optional[ParameterValidation] = None

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        result: dict[str, Any] = {
            "name": self.name,
            "description": self.description,
            "type": self.param_type.value,
            "required": self.required,
        }
        if self.default is not None:
            result["default"] = self.default
        if self.enum_values:
            result["enum"] = self.enum_values
        if self.validation:
            if self.validation.pattern:
                result["pattern"] = self.validation.pattern
            if self.validation.min_length is not None:
                result["minLength"] = self.validation.min_length
            if self.validation.max_length is not None:
                result["maxLength"] = self.validation.max_length
            if self.validation.format:
                result["format"] = self.validation.format
            if self.validation.minimum is not None:
                result["minimum"] = self.validation.minimum
            if self.validation.maximum is not None:
                result["maximum"] = self.validation.maximum
            if self.validation.enum:
                result["enum"] = self.validation.enum
        return result


@dataclass
class ToolDefinition:
    """Definition of a skill tool."""
    name: str
    description: str
    handler: Callable[..., Any]
    parameters: list[ParameterDefinition] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            "name": self.name,
            "description": self.description,
            "parameters": [p.to_dict() for p in self.parameters],
        }


@dataclass
class SkillMetadata:
    """Metadata about a skill."""
    name: str
    description: str
    version: str = "1.0.0"
    author: Optional[str] = None
    repository: Optional[str] = None
    license: str = "MIT"
    tags: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        result = {
            "name": self.name,
            "description": self.description,
            "version": self.version,
            "license": self.license,
        }
        if self.author:
            result["author"] = self.author
        if self.repository:
            result["repository"] = self.repository
        if self.tags:
            result["tags"] = self.tags
        return result


class ErrorCode(str, Enum):
    """Error codes for structured error handling."""
    VALIDATION_ERROR = "VALIDATION_ERROR"
    AUTH_ERROR = "AUTH_ERROR"
    PERMISSION_ERROR = "PERMISSION_ERROR"
    RATE_LIMIT = "RATE_LIMIT"
    NOT_FOUND = "NOT_FOUND"
    CONFLICT = "CONFLICT"
    TIMEOUT = "TIMEOUT"
    NETWORK_ERROR = "NETWORK_ERROR"
    SERVICE_ERROR = "SERVICE_ERROR"
    INTERNAL_ERROR = "INTERNAL_ERROR"


@dataclass
class SkillError:
    """Structured error with context."""
    code: ErrorCode
    message: str
    details: Optional[dict[str, Any]] = None
    retryable: bool = False
    retry_after: Optional[int] = None

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        result: dict[str, Any] = {
            "code": self.code.value,
            "message": self.message,
            "retryable": self.retryable,
        }
        if self.details:
            result["details"] = self.details
        if self.retry_after:
            result["retry_after"] = self.retry_after
        return result


@dataclass
class ExecutionResult:
    """Result of a tool execution."""
    success: bool
    output: str
    error_message: Optional[str] = None
    data: Optional[dict[str, Any]] = None
    error: Optional[SkillError] = None

    @classmethod
    def ok(cls, output: str, data: Optional[dict[str, Any]] = None) -> "ExecutionResult":
        """Create a successful result."""
        return cls(success=True, output=output, data=data)

    @classmethod
    def error(cls, message: str, code: ErrorCode = ErrorCode.INTERNAL_ERROR) -> "ExecutionResult":
        """Create an error result."""
        return cls(
            success=False,
            output="",
            error_message=message,
            error=SkillError(code=code, message=message)
        )

    @classmethod
    def from_error(cls, error: SkillError) -> "ExecutionResult":
        """Create an execution result from a SkillError."""
        return cls(
            success=False,
            output="",
            error_message=error.message,
            error=error
        )

    @classmethod
    def validation_error(cls, message: str, details: Optional[dict[str, Any]] = None) -> "ExecutionResult":
        """Create a validation error result."""
        return cls(
            success=False,
            output="",
            error_message=message,
            error=SkillError(code=ErrorCode.VALIDATION_ERROR, message=message, details=details)
        )

    @classmethod
    def auth_error(cls, message: str = "Authentication required") -> "ExecutionResult":
        """Create an authentication error result."""
        return cls(
            success=False,
            output="",
            error_message=message,
            error=SkillError(code=ErrorCode.AUTH_ERROR, message=message)
        )

    @classmethod
    def rate_limit_error(cls, retry_after: Optional[int] = None) -> "ExecutionResult":
        """Create a rate limit error result."""
        return cls(
            success=False,
            output="",
            error_message="Rate limit exceeded",
            error=SkillError(
                code=ErrorCode.RATE_LIMIT,
                message="Rate limit exceeded",
                retryable=True,
                retry_after=retry_after
            )
        )

    @classmethod
    def not_found_error(cls, resource: str) -> "ExecutionResult":
        """Create a not found error result."""
        return cls(
            success=False,
            output="",
            error_message=f"{resource} not found",
            error=SkillError(code=ErrorCode.NOT_FOUND, message=f"{resource} not found")
        )

    @classmethod
    def service_error(cls, service: str, message: str) -> "ExecutionResult":
        """Create a service error result."""
        return cls(
            success=False,
            output="",
            error_message=f"{service}: {message}",
            error=SkillError(
                code=ErrorCode.SERVICE_ERROR,
                message=f"{service}: {message}",
                retryable=True
            )
        )

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        result: dict[str, Any] = {
            "success": self.success,
            "output": self.output,
        }
        if self.error_message:
            result["error_message"] = self.error_message
        if self.data:
            result["data"] = self.data
        if self.error:
            result["error"] = self.error.to_dict()
        return result


@dataclass
class ConfigField:
    """Definition of a configuration field."""
    name: str
    description: str
    field_type: ParameterType = ParameterType.STRING
    required: bool = False
    default: Optional[Any] = None
    secret: bool = False

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for YAML serialization."""
        result: dict[str, Any] = {
            "type": self.field_type.value,
            "description": self.description,
        }
        if self.required:
            result["required"] = True
        if self.default is not None:
            result["default"] = self.default
        if self.secret:
            result["secret"] = True
        return result
