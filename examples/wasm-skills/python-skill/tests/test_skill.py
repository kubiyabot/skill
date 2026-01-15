"""Tests for the Python example skill."""

import json
import pytest
import sys
from pathlib import Path

# Add the SDK to path for testing
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent / "sdk" / "python"))

from src.main import PythonExampleSkill


class TestPythonExampleSkill:
    """Test suite for PythonExampleSkill."""

    def setup_method(self):
        """Set up test fixtures."""
        self.skill = PythonExampleSkill()

    def test_greet_default(self):
        """Test greeting with default name."""
        result = self.skill.greet()
        assert "World" in result
        assert "Hello" in result

    def test_greet_custom_name(self):
        """Test greeting with custom name."""
        result = self.skill.greet(name="Alice")
        assert "Alice" in result

    def test_greet_formal(self):
        """Test formal greeting."""
        result = self.skill.greet(name="CEO", formal=True)
        assert "Good day" in result
        assert "CEO" in result
        assert "assist" in result

    def test_echo_basic(self):
        """Test basic echo."""
        result = self.skill.echo(message="Hello World")
        assert result == "Hello World"

    def test_echo_uppercase(self):
        """Test echo with uppercase."""
        result = self.skill.echo(message="hello", uppercase=True)
        assert result == "HELLO"

    def test_echo_reverse(self):
        """Test echo with reverse."""
        result = self.skill.echo(message="stressed", reverse=True)
        assert result == "desserts"

    def test_echo_both(self):
        """Test echo with both transformations."""
        result = self.skill.echo(message="hello", uppercase=True, reverse=True)
        assert result == "OLLEH"

    def test_calculate_add(self):
        """Test addition."""
        result = self.skill.calculate(a=5, b=3, operation="add")
        assert result.success
        assert result.data["result"] == 8

    def test_calculate_subtract(self):
        """Test subtraction."""
        result = self.skill.calculate(a=10, b=4, operation="subtract")
        assert result.success
        assert result.data["result"] == 6

    def test_calculate_multiply(self):
        """Test multiplication."""
        result = self.skill.calculate(a=7, b=6, operation="multiply")
        assert result.success
        assert result.data["result"] == 42

    def test_calculate_divide(self):
        """Test division."""
        result = self.skill.calculate(a=20, b=4, operation="divide")
        assert result.success
        assert result.data["result"] == 5

    def test_calculate_divide_by_zero(self):
        """Test division by zero."""
        result = self.skill.calculate(a=10, b=0, operation="divide")
        assert not result.success
        assert "zero" in result.error_message.lower()

    def test_calculate_invalid_operation(self):
        """Test invalid operation."""
        result = self.skill.calculate(a=5, b=3, operation="modulo")
        assert not result.success
        assert "Unknown operation" in result.error_message

    def test_process_list_valid(self):
        """Test processing valid list."""
        result = self.skill.process_list(items="[1, 2, 3, 4, 5]")
        assert result.success
        assert result.data["count"] == 5
        assert result.data["sum"] == 15
        assert result.data["min"] == 1
        assert result.data["max"] == 5
        assert result.data["average"] == 3.0

    def test_process_list_invalid_json(self):
        """Test processing invalid JSON."""
        result = self.skill.process_list(items="not valid json")
        assert not result.success
        assert "Invalid JSON" in result.error_message

    def test_process_list_not_array(self):
        """Test processing non-array JSON."""
        result = self.skill.process_list(items='{"key": "value"}')
        assert not result.success
        assert "array" in result.error_message.lower()

    def test_process_list_empty_numbers(self):
        """Test processing array with no valid numbers."""
        result = self.skill.process_list(items='["a", "b", "c"]')
        assert not result.success
        assert "No valid numbers" in result.error_message

    def test_status(self):
        """Test status output."""
        result = self.skill.status()
        assert result["skill_name"] == "python-example"
        assert result["version"] == "1.0.0"
        assert "config" in result
        assert "tools_available" in result
        assert "greet" in result["tools_available"]
        assert "calculate" in result["tools_available"]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
