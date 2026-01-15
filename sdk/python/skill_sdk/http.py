"""
HTTP Client for Skill Engine SDK.

Provides a requests-based HTTP client with:
- Automatic retry with exponential backoff
- Auth-aware request helpers
- JSON request/response handling
- Error handling integration

Example:
    # Basic usage
    client = SkillHttpClient(base_url="https://api.example.com")
    response = client.get("/users")

    # With authentication
    client = create_authenticated_client(
        base_url="https://api.github.com",
        auth_type="bearer",
        token_key="GITHUB_TOKEN"
    )
    repos = client.get("/user/repos")
"""

import base64
import os
import time
from dataclasses import dataclass, field
from typing import Any, Literal, Optional
from urllib.parse import urljoin

# Note: In WASM context, we use fetch API. This module provides
# a compatible interface that works in both environments.

try:
    import requests
    HAS_REQUESTS = True
except ImportError:
    HAS_REQUESTS = False


AuthType = Literal["bearer", "basic", "api-key"]


@dataclass
class HttpResponse:
    """HTTP response with parsed body."""
    ok: bool
    status: int
    status_text: str
    headers: dict[str, str]
    data: Any


@dataclass
class HttpClientOptions:
    """Options for creating an HTTP client."""
    base_url: str = ""
    headers: dict[str, str] = field(default_factory=dict)
    timeout: float = 30.0
    retries: int = 3
    retry_delay: float = 1.0


class SkillHttpClient:
    """
    HTTP client for making requests from skills.

    Uses the requests library (available in standard Python).

    Example:
        client = SkillHttpClient(
            base_url="https://api.example.com",
            headers={"X-Custom-Header": "value"}
        )

        # GET request
        users = client.get("/users")

        # POST request
        new_user = client.post("/users", {"name": "John"})

        # With error handling
        response = client.get("/data")
        if not response.ok:
            print(f"Request failed: {response.status}")
    """

    def __init__(
        self,
        base_url: str = "",
        headers: Optional[dict[str, str]] = None,
        timeout: float = 30.0,
        retries: int = 3,
        retry_delay: float = 1.0,
    ):
        self.base_url = base_url.rstrip("/") if base_url else ""
        self.default_headers = {
            "Content-Type": "application/json",
            "Accept": "application/json",
            **(headers or {}),
        }
        self.timeout = timeout
        self.retries = retries
        self.retry_delay = retry_delay

        if not HAS_REQUESTS:
            raise ImportError(
                "requests library is required for HTTP client. "
                "Install with: pip install requests"
            )

    def request(
        self,
        method: str,
        url: str,
        body: Optional[Any] = None,
        headers: Optional[dict[str, str]] = None,
        timeout: Optional[float] = None,
        retries: Optional[int] = None,
        retry_delay: Optional[float] = None,
    ) -> HttpResponse:
        """
        Make an HTTP request with automatic retry and error handling.

        Args:
            method: HTTP method (GET, POST, PUT, DELETE, etc.)
            url: URL path (appended to base_url)
            body: Request body (will be JSON serialized)
            headers: Additional headers
            timeout: Request timeout in seconds
            retries: Number of retry attempts
            retry_delay: Delay between retries in seconds

        Returns:
            HttpResponse with parsed body
        """
        full_url = urljoin(self.base_url + "/", url.lstrip("/")) if self.base_url else url
        request_headers = {**self.default_headers, **(headers or {})}
        request_timeout = timeout or self.timeout
        max_retries = retries if retries is not None else self.retries
        delay = retry_delay or self.retry_delay

        last_error: Optional[Exception] = None

        for attempt in range(max_retries + 1):
            try:
                response = requests.request(
                    method=method,
                    url=full_url,
                    json=body if body is not None else None,
                    headers=request_headers,
                    timeout=request_timeout,
                )

                # Parse response
                content_type = response.headers.get("content-type", "")
                if "application/json" in content_type:
                    try:
                        data = response.json()
                    except ValueError:
                        data = response.text
                else:
                    data = response.text

                return HttpResponse(
                    ok=response.ok,
                    status=response.status_code,
                    status_text=response.reason or "",
                    headers=dict(response.headers),
                    data=data,
                )

            except requests.exceptions.Timeout as e:
                last_error = e
                # Don't retry on timeout
                break

            except requests.exceptions.RequestException as e:
                last_error = e
                if attempt < max_retries:
                    # Exponential backoff
                    sleep_time = delay * (2 ** attempt)
                    time.sleep(sleep_time)
                    continue
                break

        # All retries failed
        raise ConnectionError(
            f"Request to {full_url} failed after {max_retries + 1} attempts: {last_error}"
        )

    def get(
        self,
        url: str,
        headers: Optional[dict[str, str]] = None,
        timeout: Optional[float] = None,
    ) -> HttpResponse:
        """Make a GET request."""
        return self.request("GET", url, headers=headers, timeout=timeout)

    def post(
        self,
        url: str,
        body: Optional[Any] = None,
        headers: Optional[dict[str, str]] = None,
        timeout: Optional[float] = None,
    ) -> HttpResponse:
        """Make a POST request."""
        return self.request("POST", url, body=body, headers=headers, timeout=timeout)

    def put(
        self,
        url: str,
        body: Optional[Any] = None,
        headers: Optional[dict[str, str]] = None,
        timeout: Optional[float] = None,
    ) -> HttpResponse:
        """Make a PUT request."""
        return self.request("PUT", url, body=body, headers=headers, timeout=timeout)

    def patch(
        self,
        url: str,
        body: Optional[Any] = None,
        headers: Optional[dict[str, str]] = None,
        timeout: Optional[float] = None,
    ) -> HttpResponse:
        """Make a PATCH request."""
        return self.request("PATCH", url, body=body, headers=headers, timeout=timeout)

    def delete(
        self,
        url: str,
        headers: Optional[dict[str, str]] = None,
        timeout: Optional[float] = None,
    ) -> HttpResponse:
        """Make a DELETE request."""
        return self.request("DELETE", url, headers=headers, timeout=timeout)


def get_config_value(key: str, default: Optional[str] = None) -> Optional[str]:
    """Get a configuration value from environment variables."""
    # Try SKILL_ prefixed version first
    value = os.environ.get(f"SKILL_{key}")
    if value:
        return value

    # Try direct key
    value = os.environ.get(key)
    if value:
        return value

    return default


def create_authenticated_client(
    base_url: str,
    auth_type: AuthType,
    token_key: Optional[str] = None,
    header_name: Optional[str] = None,
    headers: Optional[dict[str, str]] = None,
    timeout: float = 30.0,
    retries: int = 3,
) -> SkillHttpClient:
    """
    Create an HTTP client with authentication configured from skill config.

    The token/key is read from environment variables (set by `skill auth login`).

    Args:
        base_url: Base URL for all requests
        auth_type: Type of authentication (bearer, basic, api-key)
        token_key: Environment variable containing the token
        header_name: Custom header name for API key auth
        headers: Additional default headers
        timeout: Request timeout in seconds
        retries: Number of retry attempts

    Returns:
        Configured SkillHttpClient

    Example:
        # Bearer token auth (OAuth2, JWT)
        github = create_authenticated_client(
            base_url="https://api.github.com",
            auth_type="bearer",
            token_key="GITHUB_TOKEN"
        )

        # API key auth
        openai = create_authenticated_client(
            base_url="https://api.openai.com/v1",
            auth_type="api-key",
            token_key="OPENAI_API_KEY",
            header_name="Authorization"
        )

        # Basic auth
        api = create_authenticated_client(
            base_url="https://api.example.com",
            auth_type="basic",
            token_key="API_CREDENTIALS"  # Format: "username:password"
        )
    """
    auth_headers: dict[str, str] = dict(headers) if headers else {}

    # Get default token key if not provided
    if token_key is None:
        token_key = _get_default_token_key(auth_type)

    # Get token from config
    token = get_config_value(token_key)

    if not token:
        import warnings
        warnings.warn(
            f"Auth token not found in config key '{token_key}'. "
            f"Run 'skill auth login' to configure authentication."
        )

    # Set auth header based on type
    if token:
        if auth_type == "bearer":
            auth_headers["Authorization"] = f"Bearer {token}"

        elif auth_type == "basic":
            # Token should be in format "username:password"
            encoded = base64.b64encode(token.encode()).decode()
            auth_headers["Authorization"] = f"Basic {encoded}"

        elif auth_type == "api-key":
            name = header_name or "X-API-Key"
            auth_headers[name] = token

    return SkillHttpClient(
        base_url=base_url,
        headers=auth_headers,
        timeout=timeout,
        retries=retries,
    )


def _get_default_token_key(auth_type: AuthType) -> str:
    """Get the default token key for an auth type."""
    defaults = {
        "bearer": "ACCESS_TOKEN",
        "api-key": "API_KEY",
        "basic": "CREDENTIALS",
    }
    return defaults.get(auth_type, "TOKEN")


def is_rate_limited(response: HttpResponse) -> bool:
    """Check if a response indicates a rate limit error."""
    return response.status == 429


def get_retry_after(response: HttpResponse) -> Optional[int]:
    """Get retry-after value from response headers (in seconds)."""
    retry_after = response.headers.get("retry-after") or response.headers.get("Retry-After")
    if not retry_after:
        return None

    # Could be a number of seconds
    try:
        return int(retry_after)
    except ValueError:
        pass

    # Could be an HTTP date
    try:
        from email.utils import parsedate_to_datetime
        from datetime import datetime, timezone

        date = parsedate_to_datetime(retry_after)
        now = datetime.now(timezone.utc)
        delta = (date - now).total_seconds()
        return max(0, int(delta))
    except (ValueError, TypeError):
        pass

    return None


def fetch_json(url: str, method: str = "GET", body: Optional[Any] = None) -> Any:
    """
    Simple fetch wrapper with JSON handling.

    For quick one-off requests without creating a client.

    Args:
        url: Full URL to fetch
        method: HTTP method
        body: Request body

    Returns:
        Parsed JSON response

    Raises:
        ConnectionError: If the request fails
        ValueError: If the response is not successful

    Example:
        data = fetch_json("https://api.example.com/user/1")
    """
    client = SkillHttpClient()
    response = client.request(method, url, body=body)

    if not response.ok:
        raise ValueError(
            f"Request failed with status {response.status}: {response.status_text}"
        )

    return response.data
