/**
 * Simple Skill - SDK version
 *
 * This demonstrates using the @skill-engine/sdk to create skills
 * with a clean, type-safe API.
 */

import { defineSkill } from '@skill-engine/sdk';

export default defineSkill({
  metadata: {
    name: 'simple-skill',
    version: '1.0.0',
    description: 'A simple example skill with basic tools',
    author: 'Skill Engine Team'
  },

  tools: [
    {
      name: 'hello',
      description: 'Greet someone with a friendly message',
      parameters: [
        {
          name: 'name',
          paramType: 'string',
          description: 'Name of the person to greet',
          required: true
        },
        {
          name: 'greeting',
          paramType: 'string',
          description: 'Custom greeting message',
          required: false,
          defaultValue: 'Hello'
        }
      ],
      handler: async (args) => {
        const name = args.name || 'World';
        const greeting = args.greeting || 'Hello';
        const message = `${greeting}, ${name}! 👋\n\nWelcome to Skill Engine!\n`;

        return {
          success: true,
          output: message,
          errorMessage: null
        };
      }
    },

    {
      name: 'echo',
      description: 'Echo back the provided message',
      parameters: [
        {
          name: 'message',
          paramType: 'string',
          description: 'Message to echo',
          required: true
        },
        {
          name: 'repeat',
          paramType: 'string',
          description: 'Number of times to repeat',
          required: false,
          defaultValue: '1'
        }
      ],
      handler: async (args) => {
        const message = args.message;
        const repeat = parseInt(args.repeat || '1', 10);

        if (!message) {
          return {
            success: false,
            output: '',
            errorMessage: 'Message parameter is required'
          };
        }

        let output = '';
        for (let i = 0; i < repeat; i++) {
          output += message;
          if (i < repeat - 1) {
            output += '\n';
          }
        }

        return {
          success: true,
          output: output + '\n',
          errorMessage: null
        };
      }
    },

    {
      name: 'calculate',
      description: 'Perform basic arithmetic operations',
      parameters: [
        {
          name: 'operation',
          paramType: 'string',
          description: 'Operation: add, subtract, multiply, divide',
          required: true
        },
        {
          name: 'a',
          paramType: 'string',
          description: 'First number',
          required: true
        },
        {
          name: 'b',
          paramType: 'string',
          description: 'Second number',
          required: true
        }
      ],
      handler: async (args) => {
        const operation = args.operation;
        const a = parseFloat(args.a);
        const b = parseFloat(args.b);

        if (isNaN(a) || isNaN(b)) {
          return {
            success: false,
            output: '',
            errorMessage: "Both 'a' and 'b' must be valid numbers"
          };
        }

        let result: number;
        switch (operation) {
          case 'add':
            result = a + b;
            break;
          case 'subtract':
            result = a - b;
            break;
          case 'multiply':
            result = a * b;
            break;
          case 'divide':
            if (b === 0) {
              return {
                success: false,
                output: '',
                errorMessage: 'Cannot divide by zero'
              };
            }
            result = a / b;
            break;
          default:
            return {
              success: false,
              output: '',
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
    }
  ]
});
