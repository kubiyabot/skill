/**
 * Simple Skill - A minimal example showing how to create skills
 *
 * This skill can be run directly without any build steps:
 *   skill run ./examples/simple-skill hello name=World
 */

// Define skill metadata
export function getMetadata() {
  return {
    name: "simple-skill",
    version: "1.0.0",
    description: "A simple example skill with basic tools",
    author: "Skill Engine Team"
  };
}

// Define available tools
export function getTools() {
  return [
    {
      name: "hello",
      description: "Greet someone with a friendly message",
      parameters: [
        {
          name: "name",
          paramType: "string",
          description: "Name of the person to greet",
          required: true
        },
        {
          name: "greeting",
          paramType: "string",
          description: "Custom greeting message",
          required: false,
          defaultValue: "Hello"
        }
      ]
    },
    {
      name: "echo",
      description: "Echo back the provided message",
      parameters: [
        {
          name: "message",
          paramType: "string",
          description: "Message to echo",
          required: true
        },
        {
          name: "repeat",
          paramType: "number",
          description: "Number of times to repeat",
          required: false,
          defaultValue: "1"
        }
      ]
    },
    {
      name: "calculate",
      description: "Perform basic arithmetic operations",
      parameters: [
        {
          name: "operation",
          paramType: "string",
          description: "Operation: add, subtract, multiply, divide",
          required: true
        },
        {
          name: "a",
          paramType: "number",
          description: "First number",
          required: true
        },
        {
          name: "b",
          paramType: "number",
          description: "Second number",
          required: true
        }
      ]
    }
  ];
}

// Execute a tool
export async function executeTool(toolName, argsJson) {
  try {
    // Parse arguments
    const args = JSON.parse(argsJson);

    // Route to appropriate tool handler
    switch (toolName) {
      case "hello":
        return handleHello(args);
      case "echo":
        return handleEcho(args);
      case "calculate":
        return handleCalculate(args);
      default:
        return {
          success: false,
          output: "",
          errorMessage: `Unknown tool: ${toolName}`
        };
    }
  } catch (error) {
    return {
      success: false,
      output: "",
      errorMessage: `Error executing tool: ${error.message}`
    };
  }
}

// Validate configuration (optional)
export async function validateConfig() {
  // No configuration needed for this simple skill
  return { ok: null };
}

// Tool Handlers

function handleHello(args) {
  const name = args.name || "World";
  const greeting = args.greeting || "Hello";

  const message = `${greeting}, ${name}! 👋\n\nWelcome to Skill Engine!\n`;

  return {
    success: true,
    output: message,
    errorMessage: null
  };
}

function handleEcho(args) {
  const message = args.message;
  const repeat = parseInt(args.repeat || "1", 10);

  if (!message) {
    return {
      success: false,
      output: "",
      errorMessage: "Message parameter is required"
    };
  }

  let output = "";
  for (let i = 0; i < repeat; i++) {
    output += message;
    if (i < repeat - 1) {
      output += "\n";
    }
  }

  return {
    success: true,
    output: output + "\n",
    errorMessage: null
  };
}

function handleCalculate(args) {
  const operation = args.operation;
  const a = parseFloat(args.a);
  const b = parseFloat(args.b);

  if (isNaN(a) || isNaN(b)) {
    return {
      success: false,
      output: "",
      errorMessage: "Both 'a' and 'b' must be valid numbers"
    };
  }

  let result;
  switch (operation) {
    case "add":
      result = a + b;
      break;
    case "subtract":
      result = a - b;
      break;
    case "multiply":
      result = a * b;
      break;
    case "divide":
      if (b === 0) {
        return {
          success: false,
          output: "",
          errorMessage: "Cannot divide by zero"
        };
      }
      result = a / b;
      break;
    default:
      return {
        success: false,
        output: "",
        errorMessage: `Unknown operation: ${operation}. Use: add, subtract, multiply, or divide`
      };
  }

  const output = `${a} ${operation} ${b} = ${result}\n`;

  return {
    success: true,
    output,
    errorMessage: null
  };
}
