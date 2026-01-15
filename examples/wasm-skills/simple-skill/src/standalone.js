// Standalone skill for WASM compilation
// This version has SDK code inlined

// Export functions matching WIT interface
export function getMetadata() {
  return JSON.stringify({
    name: 'simple-skill',
    version: '1.0.0',
    description: 'A simple example skill',
    author: 'Skill Engine Team'
  });
}

export function getTools() {
  return JSON.stringify([
    {
      name: 'hello',
      description: 'Greet someone',
      parameters: [
        { name: 'name', paramType: 'string', description: 'Name to greet', required: true }
      ]
    },
    {
      name: 'echo',
      description: 'Echo a message',
      parameters: [
        { name: 'message', paramType: 'string', description: 'Message to echo', required: true }
      ]
    }
  ]);
}

export function executeTool(toolName, argsJson) {
  try {
    const args = JSON.parse(argsJson);

    switch (toolName) {
      case 'hello':
        const name = args.name || 'World';
        return JSON.stringify({
          ok: {
            success: true,
            output: `Hello, ${name}! 👋`,
            errorMessage: null
          }
        });

      case 'echo':
        const message = args.message;
        if (!message) {
          return JSON.stringify({
            err: 'Message parameter is required'
          });
        }
        return JSON.stringify({
          ok: {
            success: true,
            output: message,
            errorMessage: null
          }
        });

      default:
        return JSON.stringify({
          err: `Unknown tool: ${toolName}`
        });
    }
  } catch (error) {
    return JSON.stringify({
      err: `Execution error: ${error.message}`
    });
  }
}

export function validateConfig(configJson) {
  // No config needed for this simple skill
  return JSON.stringify({ ok: null });
}
