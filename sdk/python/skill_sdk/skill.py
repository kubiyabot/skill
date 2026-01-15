"""
Core skill definition and decorator system.
"""

import functools
import inspect
import json
import os
from typing import Any, Callable, Optional, TypeVar, get_type_hints

from skill_sdk.types import (
    ConfigField,
    ExecutionResult,
    ParameterDefinition,
    ParameterType,
    ParameterValidation,
    SkillMetadata,
    ToolDefinition,
)
from skill_sdk.exceptions import (
    ConfigurationError,
    ExecutionError,
    ParameterError,
    ToolNotFoundError,
    ValidationError,
)

T = TypeVar("T")

# Global registry for skill configuration
_skill_registry: dict[str, "SkillInstance"] = {}
_current_skill: Optional["SkillInstance"] = None


class SkillInstance:
    """Runtime instance of a skill."""

    def __init__(self, metadata: SkillMetadata, config_fields: list[ConfigField]):
        self.metadata = metadata
        self.config_fields = config_fields
        self.tools: dict[str, ToolDefinition] = {}
        self._config: dict[str, Any] = {}
        self._instance: Any = None

    def register_tool(self, tool: ToolDefinition) -> None:
        """Register a tool with this skill."""
        self.tools[tool.name] = tool

    def get_config(self, key: str, default: Any = None) -> Any:
        """Get a configuration value."""
        # First check environment variables (prefixed with skill name)
        env_key = f"{self.metadata.name.upper().replace('-', '_')}_{key.upper()}"
        if env_key in os.environ:
            return os.environ[env_key]
        return self._config.get(key, default)

    def set_config(self, config: dict[str, Any]) -> None:
        """Set configuration values."""
        self._config.update(config)

    def validate_config(self) -> list[str]:
        """Validate that all required config fields are present."""
        errors = []
        for field in self.config_fields:
            if field.required:
                value = self.get_config(field.name)
                if value is None:
                    errors.append(f"Missing required config: {field.name}")
        return errors

    def get_tools(self) -> list[dict[str, Any]]:
        """Get all tool definitions as dictionaries."""
        return [tool.to_dict() for tool in self.tools.values()]

    def get_metadata(self) -> dict[str, Any]:
        """Get skill metadata as dictionary."""
        return self.metadata.to_dict()

    def execute_tool(
        self, tool_name: str, args: dict[str, Any]
    ) -> ExecutionResult:
        """Execute a tool by name with given arguments."""
        if tool_name not in self.tools:
            raise ToolNotFoundError(tool_name)

        tool = self.tools[tool_name]

        # Validate required parameters
        for param in tool.parameters:
            if param.required and param.name not in args:
                if param.default is None:
                    raise ParameterError(
                        f"Missing required parameter: {param.name}",
                        param.name
                    )

        # Add defaults for missing optional parameters
        for param in tool.parameters:
            if param.name not in args and param.default is not None:
                args[param.name] = param.default

        try:
            # Get the instance if we have one, otherwise use None (for static methods)
            instance = self._instance

            # Call the handler
            if instance is not None:
                result = tool.handler(instance, **args)
            else:
                result = tool.handler(**args)

            # Convert result to ExecutionResult if needed
            if isinstance(result, ExecutionResult):
                return result
            elif isinstance(result, dict):
                return ExecutionResult.ok(
                    json.dumps(result, default=str),
                    data=result
                )
            else:
                return ExecutionResult.ok(str(result))

        except ValidationError as e:
            return ExecutionResult.error(f"Validation error: {e.message}")
        except ExecutionError as e:
            return ExecutionResult.error(f"Execution error: {e.message}")
        except Exception as e:
            return ExecutionResult.error(f"Unexpected error: {str(e)}")


def _python_type_to_param_type(python_type: type) -> ParameterType:
    """Convert Python type hint to ParameterType."""
    type_mapping = {
        str: ParameterType.STRING,
        int: ParameterType.INTEGER,
        float: ParameterType.FLOAT,
        bool: ParameterType.BOOLEAN,
        list: ParameterType.ARRAY,
        dict: ParameterType.OBJECT,
    }
    return type_mapping.get(python_type, ParameterType.STRING)


def _extract_parameters(func: Callable[..., Any]) -> list[ParameterDefinition]:
    """Extract parameter definitions from function signature and type hints."""
    sig = inspect.signature(func)
    hints = get_type_hints(func) if hasattr(func, "__annotations__") else {}

    params = []
    for name, param in sig.parameters.items():
        # Skip 'self' parameter
        if name == "self":
            continue

        # Get type hint
        param_type = hints.get(name, str)
        if hasattr(param_type, "__origin__"):  # Handle Optional, List, etc.
            param_type = param_type.__args__[0] if param_type.__args__ else str

        # Check if required (no default value)
        required = param.default is inspect.Parameter.empty
        default = None if required else param.default

        # Check for param decorator metadata
        description = getattr(func, f"_param_{name}_description", f"Parameter: {name}")

        # Check for validation constraints from @param decorator
        validation = _extract_validation(func, name)

        # Check for enum values (can be on validation or as separate attribute)
        enum_values = getattr(func, f"_param_{name}_enum", None)

        params.append(ParameterDefinition(
            name=name,
            description=description,
            param_type=_python_type_to_param_type(param_type),
            required=required,
            default=default,
            enum_values=enum_values,
            validation=validation,
        ))

    return params


def _extract_validation(func: Callable[..., Any], param_name: str) -> Optional[ParameterValidation]:
    """Extract validation constraints from @param decorator metadata."""
    # Check if any validation constraints exist
    pattern = getattr(func, f"_param_{param_name}_pattern", None)
    min_length = getattr(func, f"_param_{param_name}_min_length", None)
    max_length = getattr(func, f"_param_{param_name}_max_length", None)
    format_type = getattr(func, f"_param_{param_name}_format", None)
    minimum = getattr(func, f"_param_{param_name}_minimum", None)
    maximum = getattr(func, f"_param_{param_name}_maximum", None)
    enum = getattr(func, f"_param_{param_name}_enum", None)

    # Only create validation if at least one constraint exists
    if any([pattern, min_length, max_length, format_type, minimum, maximum, enum]):
        return ParameterValidation(
            pattern=pattern,
            min_length=min_length,
            max_length=max_length,
            format=format_type,
            minimum=minimum,
            maximum=maximum,
            enum=enum,
        )

    return None


def tool(
    description: str,
    name: Optional[str] = None,
) -> Callable[[Callable[..., T]], Callable[..., T]]:
    """
    Decorator to mark a method as a skill tool.

    Args:
        description: Description of what the tool does
        name: Optional custom name (defaults to function name)

    Example:
        @tool(description="Greet someone by name")
        def hello(self, name: str = "World") -> str:
            return f"Hello, {name}!"
    """
    def decorator(func: Callable[..., T]) -> Callable[..., T]:
        # Store tool metadata on the function
        func._is_tool = True  # type: ignore
        func._tool_name = name or func.__name__  # type: ignore
        func._tool_description = description  # type: ignore

        @functools.wraps(func)
        def wrapper(*args: Any, **kwargs: Any) -> T:
            return func(*args, **kwargs)

        # Copy metadata to wrapper
        wrapper._is_tool = True  # type: ignore
        wrapper._tool_name = func._tool_name  # type: ignore
        wrapper._tool_description = func._tool_description  # type: ignore

        return wrapper

    return decorator


def param(
    name: str,
    description: str,
    *,
    # Validation constraints
    pattern: Optional[str] = None,
    min_length: Optional[int] = None,
    max_length: Optional[int] = None,
    format: Optional[str] = None,
    minimum: Optional[float] = None,
    maximum: Optional[float] = None,
    enum: Optional[list[str]] = None,
) -> Callable[[Callable[..., T]], Callable[..., T]]:
    """
    Decorator to add parameter documentation and validation to a tool.

    Args:
        name: Parameter name
        description: Parameter description
        pattern: Regex pattern for string validation
        min_length: Minimum string length
        max_length: Maximum string length
        format: String format (email, url, date, uuid, etc.)
        minimum: Minimum numeric value
        maximum: Maximum numeric value
        enum: List of allowed values

    Example:
        @tool(description="Create a user")
        @param("email", "User's email address", format="email")
        @param("name", "User's full name", min_length=1, max_length=100)
        @param("age", "User's age", minimum=0, maximum=150)
        @param("role", "User's role", enum=["admin", "user", "guest"])
        def create_user(self, email: str, name: str, age: int, role: str) -> dict:
            return {"email": email, "name": name, "age": age, "role": role}
    """
    def decorator(func: Callable[..., T]) -> Callable[..., T]:
        setattr(func, f"_param_{name}_description", description)

        # Store validation constraints
        if pattern is not None:
            setattr(func, f"_param_{name}_pattern", pattern)
        if min_length is not None:
            setattr(func, f"_param_{name}_min_length", min_length)
        if max_length is not None:
            setattr(func, f"_param_{name}_max_length", max_length)
        if format is not None:
            setattr(func, f"_param_{name}_format", format)
        if minimum is not None:
            setattr(func, f"_param_{name}_minimum", minimum)
        if maximum is not None:
            setattr(func, f"_param_{name}_maximum", maximum)
        if enum is not None:
            setattr(func, f"_param_{name}_enum", enum)

        return func
    return decorator


def config(
    name: str,
    description: str,
    required: bool = False,
    default: Any = None,
    secret: bool = False,
) -> Callable[[type[T]], type[T]]:
    """
    Decorator to add configuration field to a skill.

    Args:
        name: Configuration field name
        description: Field description
        required: Whether the field is required
        default: Default value
        secret: Whether this is a secret (e.g., API key)

    Example:
        @config("api_key", "API key for the service", required=True, secret=True)
        @Skill(name="my-skill", description="My skill")
        class MySkill:
            pass
    """
    def decorator(cls: type[T]) -> type[T]:
        if not hasattr(cls, "_config_fields"):
            cls._config_fields = []  # type: ignore

        cls._config_fields.append(ConfigField(  # type: ignore
            name=name,
            description=description,
            required=required,
            default=default,
            secret=secret,
        ))
        return cls
    return decorator


class Skill:
    """
    Decorator to define a skill class.

    Example:
        @Skill(
            name="my-skill",
            description="My awesome skill",
            version="1.0.0"
        )
        class MySkill:
            @tool(description="Say hello")
            def hello(self, name: str = "World") -> str:
                return f"Hello, {name}!"
    """

    def __init__(
        self,
        name: str,
        description: str,
        version: str = "1.0.0",
        author: Optional[str] = None,
        repository: Optional[str] = None,
        tags: Optional[list[str]] = None,
    ):
        self.metadata = SkillMetadata(
            name=name,
            description=description,
            version=version,
            author=author,
            repository=repository,
            tags=tags or [],
        )

    def __call__(self, cls: type[T]) -> type[T]:
        """Decorate a class as a skill."""
        global _current_skill

        # Get config fields from class if present
        config_fields = getattr(cls, "_config_fields", [])

        # Create skill instance
        skill_instance = SkillInstance(self.metadata, config_fields)

        # Find and register all tools
        for name, method in inspect.getmembers(cls, predicate=inspect.isfunction):
            if getattr(method, "_is_tool", False):
                tool_name = getattr(method, "_tool_name", name)
                tool_desc = getattr(method, "_tool_description", "")
                parameters = _extract_parameters(method)

                tool_def = ToolDefinition(
                    name=tool_name,
                    description=tool_desc,
                    handler=method,
                    parameters=parameters,
                )
                skill_instance.register_tool(tool_def)

        # Register globally
        _skill_registry[self.metadata.name] = skill_instance
        _current_skill = skill_instance

        # Add class methods
        original_init = cls.__init__ if hasattr(cls, "__init__") else None

        def new_init(self_instance: Any, *args: Any, **kwargs: Any) -> None:
            skill_instance._instance = self_instance
            if original_init:
                original_init(self_instance, *args, **kwargs)

        cls.__init__ = new_init  # type: ignore

        # Add helper methods to the class
        cls.get_config = staticmethod(  # type: ignore
            lambda key, default=None: skill_instance.get_config(key, default)
        )
        cls.get_tools = staticmethod(skill_instance.get_tools)  # type: ignore
        cls.get_metadata = staticmethod(skill_instance.get_metadata)  # type: ignore
        cls.execute_tool = staticmethod(skill_instance.execute_tool)  # type: ignore

        # Add run method for CLI execution
        @staticmethod
        def run() -> None:
            """Run the skill in CLI mode."""
            from skill_sdk.runtime import run_skill
            run_skill(skill_instance)

        cls.run = run  # type: ignore

        return cls


def get_current_skill() -> Optional[SkillInstance]:
    """Get the currently active skill instance."""
    return _current_skill


def get_skill(name: str) -> Optional[SkillInstance]:
    """Get a skill instance by name."""
    return _skill_registry.get(name)
