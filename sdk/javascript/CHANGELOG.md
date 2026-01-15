# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-13

### Added

- Initial release of `@skill-engine/sdk`
- `defineSkill()` - Create skills with metadata, tools, and handlers
- `getConfig<T>()` - Type-safe configuration from environment variables
- Parameter validation with support for:
  - Types: string, number, boolean, file, json, array, secret
  - Formats: email, url, uuid, date, datetime, ipv4, ipv6, hostname
  - Constraints: minLength, maxLength, pattern, minimum, maximum, enum
- HTTP client (`SkillHttpClient`) with:
  - Automatic retry with exponential backoff
  - Request/response timeout handling
  - JSON serialization/deserialization
- `createAuthenticatedClient()` for Bearer, API Key, and Basic auth
- JSON Schema generation for MCP integration
- Structured error handling with typed error codes
- Full TypeScript support with type declarations
- CLI tools for skill validation and componentization
