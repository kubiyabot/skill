# Core Concepts

Understanding how Skill Engine works will help you build better skills and integration.

## Architecture Overview

Skill Engine connects AI Agents (like Claude, ChatGPT, or custom agents) to real-world tools via a secure, portable runtime.

```mermaid
graph LR
    Agent[AI Agent] <--> Protocol[MCP / CLI]
    Protocol <--> Runtime[Skill Engine Runtime]
    Runtime <--> Sandbox[WASM Sandbox]
    Sandbox <--> Skill[Your Skill]
```

## key Components

### 1. Skill

A **Skill** is a package of tools that an AI agent can use.

- **Portable**: Runs on any OS (Mac, Linux, Windows).
- **Sandboxed**: Runs inside WebAssembly (WASM) for security.
- **Language Agnostic**: Can be written in JavaScript, TypeScript, Python, or Rust.

### 2. Tools

A **Tool** is a specific function within a skill.

- Example: `aws:s3-list`, `github:create-issue`.
- Defined with a JSON schema so the AI knows how to call it.

### 3. Runtime

The **Skill Engine Runtime** executes your skills.

- **JIT Compilation**: Compiles JS/TS to WASM on the fly.
- **Security**: precise capability control (network, filesystem).
- **Caching**: Compiles once, runs instantly (<10ms).

### 4. MCP (Model Context Protocol)

Skill Engine acts as an **MCP Server**. This means any MCP-compliant client (like Claude Desktop or Cursor) can automatically discover and use your installed skills.

## Security Model

Security is a core design principle.

- **Isolation**: Skills cannot access your file system or network unless explicitly allowed.
- **Capabilities**: You must grant permissions in the manifest/config (e.g., `allow_network: true`).
- **Secret Management**: API keys and secrets are stored in your OS keychain, not in plain text.

## How Execution Works

1. **Discovery**: The agent asks "What tools do I have?"
2. **Selection**: The agent selects a tool (e.g., `greet(name="Alice")`).
3. **Execution**:
   - Runtime loads the WASM module.
   - Injects configuration and secrets.
   - Executes the function in the sandbox.
4. **Result**: The output is returned to the agent.
