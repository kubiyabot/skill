# Skill Engine Documentation

Welcome to the Skill Engine documentation. This folder contains detailed guides and references for using and developing with Skill Engine.

## User Guides

| Document | Description |
|----------|-------------|
| [Skill Development](skill-development.md) | How to create skills (JavaScript, TypeScript, WASM) |
| [Web Interface](web-interface.md) | Browser-based UI for skill management and execution |
| [RAG Search](rag-search.md) | Advanced search pipeline configuration and usage |

## Architecture & Internals

| Document | Description |
|----------|-------------|
| [Project Status](project-status.md) | Current implementation status and roadmap |
| [Implementation Status](implementation-status.md) | Detailed implementation notes |

## Development Notes

| Document | Description |
|----------|-------------|
| [Example Skills Findings](example-skills-findings.md) | Notes on example skill implementations |
| [Testing Progress](testing-progress.md) | Test coverage and testing notes |
| [Session Progress](session-progress.md) | Development session notes |

## Quick Links

- [Main README](../README.md) - Project overview and quick start
- [Examples](../examples/) - Example skills
- [SDK Documentation](../sdk/) - JavaScript and Python SDKs

## Documentation Structure

```
docs/
├── README.md                    # This file
├── skill-development.md         # Skill authoring guide
├── web-interface.md            # Web UI documentation
├── rag-search.md               # RAG pipeline documentation
├── project-status.md           # Implementation status
├── implementation-status.md    # Detailed impl notes
├── example-skills-findings.md  # Example analysis
├── testing-progress.md         # Testing notes
└── session-progress.md         # Dev session notes
```

## Contributing to Docs

When adding new documentation:

1. Use lowercase filenames with hyphens (e.g., `new-feature.md`)
2. Add an entry to this README
3. Include a clear title and description
4. Use code examples where appropriate
5. Keep content up to date with code changes
