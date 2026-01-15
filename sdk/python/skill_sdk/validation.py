"""
Parameter validation for Skill Engine SDK.

Provides validation functions for parameter constraints including:
- String length and pattern validation
- Number range validation
- Format validation (email, url, etc.)
- Enum validation

Example:
    from skill_sdk.validation import validate_parameter, ValidationError

    errors = validate_parameter("test@example.com", "email", format="email")
    if errors:
        raise ValidationError(f"Invalid email: {errors[0]}")
"""

import re
from typing import Any, Optional

from skill_sdk.types import ParameterDefinition, ParameterValidation, StringFormat


# Format validators - regular expressions for common formats
FORMAT_VALIDATORS: dict[str, re.Pattern[str]] = {
    "email": re.compile(r"^[^\s@]+@[^\s@]+\.[^\s@]+$"),
    "url": re.compile(r"^https?://.+"),
    "uri": re.compile(r"^[a-z][a-z0-9+.-]*:", re.IGNORECASE),
    "uuid": re.compile(
        r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
        re.IGNORECASE,
    ),
    "date": re.compile(r"^\d{4}-\d{2}-\d{2}$"),
    "datetime": re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}"),
    "time": re.compile(r"^\d{2}:\d{2}:\d{2}"),
    "ipv4": re.compile(r"^(\d{1,3}\.){3}\d{1,3}$"),
    "ipv6": re.compile(r"^([0-9a-f]{1,4}:){7}[0-9a-f]{1,4}$", re.IGNORECASE),
    "hostname": re.compile(
        r"^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)*$",
        re.IGNORECASE,
    ),
}


def validate_string(
    value: str,
    *,
    min_length: Optional[int] = None,
    max_length: Optional[int] = None,
    pattern: Optional[str] = None,
    format: Optional[StringFormat] = None,
    enum: Optional[list[str]] = None,
) -> list[str]:
    """
    Validate a string value against constraints.

    Args:
        value: String to validate
        min_length: Minimum length
        max_length: Maximum length
        pattern: Regex pattern to match
        format: Format constraint (email, url, etc.)
        enum: List of allowed values

    Returns:
        List of validation errors (empty if valid)
    """
    errors: list[str] = []

    if min_length is not None and len(value) < min_length:
        errors.append(f"must be at least {min_length} characters")

    if max_length is not None and len(value) > max_length:
        errors.append(f"must be at most {max_length} characters")

    if pattern is not None:
        try:
            if not re.match(pattern, value):
                errors.append(f"must match pattern: {pattern}")
        except re.error as e:
            errors.append(f"invalid pattern: {e}")

    if format is not None:
        validator = FORMAT_VALIDATORS.get(format)
        if validator and not validator.match(value):
            errors.append(f"invalid {format} format")

    if enum is not None and value not in enum:
        errors.append(f"must be one of: {', '.join(enum)}")

    return errors


def validate_number(
    value: float,
    *,
    minimum: Optional[float] = None,
    maximum: Optional[float] = None,
    exclusive_minimum: Optional[float] = None,
    exclusive_maximum: Optional[float] = None,
) -> list[str]:
    """
    Validate a number value against constraints.

    Args:
        value: Number to validate
        minimum: Minimum value (inclusive)
        maximum: Maximum value (inclusive)
        exclusive_minimum: Minimum value (exclusive)
        exclusive_maximum: Maximum value (exclusive)

    Returns:
        List of validation errors (empty if valid)
    """
    errors: list[str] = []

    if minimum is not None and value < minimum:
        errors.append(f"must be at least {minimum}")

    if maximum is not None and value > maximum:
        errors.append(f"must be at most {maximum}")

    if exclusive_minimum is not None and value <= exclusive_minimum:
        errors.append(f"must be greater than {exclusive_minimum}")

    if exclusive_maximum is not None and value >= exclusive_maximum:
        errors.append(f"must be less than {exclusive_maximum}")

    return errors


def validate_parameter(
    value: Any,
    param: ParameterDefinition,
) -> list[str]:
    """
    Validate a value against a parameter definition.

    Args:
        value: Value to validate
        param: Parameter definition with constraints

    Returns:
        List of validation errors (empty if valid)
    """
    errors: list[str] = []
    validation = param.validation

    if validation is None:
        # Check enum_values on the parameter itself (legacy)
        if param.enum_values and isinstance(value, str):
            if value not in param.enum_values:
                errors.append(f"must be one of: {', '.join(param.enum_values)}")
        return errors

    # String validation
    if isinstance(value, str):
        errors.extend(
            validate_string(
                value,
                min_length=validation.min_length,
                max_length=validation.max_length,
                pattern=validation.pattern,
                format=validation.format,
                enum=validation.enum,
            )
        )

    # Number validation
    elif isinstance(value, (int, float)):
        errors.extend(
            validate_number(
                value,
                minimum=validation.minimum,
                maximum=validation.maximum,
                exclusive_minimum=validation.exclusive_minimum,
                exclusive_maximum=validation.exclusive_maximum,
            )
        )

    return errors


def validate_format(value: str, format_type: StringFormat) -> bool:
    """
    Check if a string matches a format.

    Args:
        value: String to validate
        format_type: Format to check (email, url, etc.)

    Returns:
        True if valid, False otherwise
    """
    validator = FORMAT_VALIDATORS.get(format_type)
    if validator is None:
        return True  # Unknown format, assume valid
    return bool(validator.match(value))


def validate_email(value: str) -> bool:
    """Check if a string is a valid email format."""
    return validate_format(value, "email")


def validate_url(value: str) -> bool:
    """Check if a string is a valid URL format."""
    return validate_format(value, "url")


def validate_uuid(value: str) -> bool:
    """Check if a string is a valid UUID format."""
    return validate_format(value, "uuid")


def validate_date(value: str) -> bool:
    """Check if a string is a valid date format (YYYY-MM-DD)."""
    return validate_format(value, "date")


def validate_datetime(value: str) -> bool:
    """Check if a string is a valid datetime format."""
    return validate_format(value, "datetime")


def validate_ipv4(value: str) -> bool:
    """Check if a string is a valid IPv4 address format."""
    if not validate_format(value, "ipv4"):
        return False
    # Additional check: each octet must be 0-255
    try:
        parts = value.split(".")
        return all(0 <= int(part) <= 255 for part in parts)
    except ValueError:
        return False


def validate_ipv6(value: str) -> bool:
    """Check if a string is a valid IPv6 address format."""
    return validate_format(value, "ipv6")
