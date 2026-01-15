#!/usr/bin/env node
/**
 * Skill Engine SDK CLI
 *
 * Provides commands for skill development:
 * - componentize: Compile JavaScript skill to WASM Component
 * - validate: Validate skill structure
 * - init: Create new skill from template
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import { access, constants } from 'fs/promises';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const execAsync = promisify(exec);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

interface ComponentizeOptions {
  input: string;
  output: string;
  wit?: string;
  world?: string;
  debug?: boolean;
}

/**
 * Componentize a JavaScript skill into a WASM Component
 */
async function componentize(options: ComponentizeOptions) {
  console.log('🔨 Componentizing skill to WASM...\n');

  const {
    input,
    output,
    wit = resolve(__dirname, '../../../wit'),
    world = 'skill',
    debug = false,
  } = options;

  // Validate input file exists
  try {
    await access(input, constants.R_OK);
  } catch {
    console.error(`❌ Input file not found: ${input}`);
    process.exit(1);
  }

  // Validate WIT directory exists
  try {
    await access(wit, constants.R_OK);
  } catch {
    console.error(`❌ WIT directory not found: ${wit}`);
    console.error('   Make sure wit/skill-interface.wit exists');
    process.exit(1);
  }

  console.log('Input:  ', input);
  console.log('Output: ', output);
  console.log('WIT:    ', wit);
  console.log('World:  ', world);
  console.log('');

  // Build jco componentize command
  const cmd = [
    'npx @bytecodealliance/jco componentize',
    `"${input}"`,
    `--wit "${wit}"`,
    `--world-name ${world}`,
    `--out "${output}"`,
    debug ? '--debug' : '',
  ]
    .filter(Boolean)
    .join(' ');

  try {
    console.log('Running:', cmd);
    console.log('');

    const { stdout, stderr } = await execAsync(cmd, {
      maxBuffer: 10 * 1024 * 1024, // 10MB buffer for output
    });

    if (stdout) console.log(stdout);
    if (stderr) console.error(stderr);

    console.log('\n✅ Componentization complete!');
    console.log(`\nWASM Component: ${output}`);
    console.log('\nNext steps:');
    console.log(`  skill run "${output}" <tool-name> [args...]`);
  } catch (error: any) {
    console.error('\n❌ Componentization failed:');
    console.error(error.stdout || error.message);
    console.error('\nCommon issues:');
    console.error('  - Check that your skill exports match the WIT interface');
    console.error('  - Ensure @bytecodealliance/jco is installed');
    console.error('  - Verify wit/skill-interface.wit exists');
    process.exit(1);
  }
}

/**
 * Validate a skill's structure
 */
async function validate(skillPath: string) {
  console.log('🔍 Validating skill structure...\n');

  try {
    // Try to load the skill module
    const skillModule = await import(resolve(skillPath));

    if (!skillModule.default) {
      throw new Error('Skill must export a default export');
    }

    const skill = skillModule.default;

    // Check required exports
    const requiredExports = ['getMetadata', 'getTools', 'executeTool', 'validateConfig'];
    const missingExports = requiredExports.filter((name) => typeof skill[name] !== 'function');

    if (missingExports.length > 0) {
      throw new Error(`Missing required exports: ${missingExports.join(', ')}`);
    }

    // Get and validate metadata
    const metadata = skill.getMetadata();
    console.log('Metadata:');
    console.log(`  Name:        ${metadata.name}`);
    console.log(`  Version:     ${metadata.version}`);
    console.log(`  Description: ${metadata.description}`);
    console.log(`  Author:      ${metadata.author}`);

    // Get and validate tools
    const tools = skill.getTools();
    console.log(`\nTools: ${tools.length}`);
    for (const tool of tools) {
      console.log(`\n  ${tool.name}`);
      console.log(`    Description: ${tool.description}`);
      console.log(`    Parameters:  ${tool.parameters.length}`);
      for (const param of tool.parameters) {
        const requiredStr = param.required ? 'required' : 'optional';
        console.log(`      - ${param.name} (${param.paramType}, ${requiredStr})`);
      }
    }

    console.log('\n✅ Skill structure is valid!');
  } catch (error: any) {
    console.error('\n❌ Validation failed:');
    console.error(error.message);
    process.exit(1);
  }
}

/**
 * Show usage information
 */
function showUsage() {
  console.log(`
Skill Engine SDK CLI

Usage:
  skill-sdk componentize <input.js> -o <output.wasm> [options]
  skill-sdk validate <skill.js>
  skill-sdk init <skill-name>
  skill-sdk --help

Commands:
  componentize    Compile JavaScript skill to WASM Component
  validate        Validate skill structure and exports
  init            Create new skill from template (coming soon)

Options:
  -o, --output <file>     Output WASM file (required for componentize)
  --wit <dir>             WIT directory path (default: ../../../wit)
  --world <name>          WIT world name (default: skill)
  --debug                 Enable debug output
  -h, --help              Show this help

Examples:
  # Compile skill to WASM
  skill-sdk componentize dist/skill.js -o skill.wasm

  # Validate skill structure
  skill-sdk validate dist/skill.js

  # Use custom WIT directory
  skill-sdk componentize skill.js -o skill.wasm --wit ./wit
`);
}

/**
 * Main CLI entry point
 */
async function main() {
  const args = process.argv.slice(2);

  if (args.length === 0 || args[0] === '--help' || args[0] === '-h') {
    showUsage();
    process.exit(0);
  }

  const command = args[0];

  switch (command) {
    case 'componentize': {
      const input = args[1];
      if (!input) {
        console.error('❌ Input file required');
        console.error('Usage: skill-sdk componentize <input.js> -o <output.wasm>');
        process.exit(1);
      }

      let output = 'skill.wasm';
      let wit: string | undefined;
      let world: string | undefined;
      let debug = false;

      // Parse options
      for (let i = 2; i < args.length; i++) {
        const arg = args[i];
        if (arg === '-o' || arg === '--output') {
          output = args[++i] || 'skill.wasm';
        } else if (arg === '--wit') {
          wit = args[++i];
        } else if (arg === '--world') {
          world = args[++i];
        } else if (arg === '--debug') {
          debug = true;
        }
      }

      await componentize({ input, output, wit, world, debug });
      break;
    }

    case 'validate': {
      const skillPath = args[1];
      if (!skillPath) {
        console.error('❌ Skill path required');
        console.error('Usage: skill-sdk validate <skill.js>');
        process.exit(1);
      }
      await validate(skillPath);
      break;
    }

    case 'init': {
      console.error('❌ init command not yet implemented');
      console.error('Coming soon: skill-sdk init <skill-name>');
      process.exit(1);
    }

    default:
      console.error(`❌ Unknown command: ${command}`);
      console.error('Run "skill-sdk --help" for usage information');
      process.exit(1);
  }
}

main().catch((error) => {
  console.error('\n💥 Unexpected error:', error.message);
  process.exit(1);
});
