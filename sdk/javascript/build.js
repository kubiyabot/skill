#!/usr/bin/env node
/**
 * Build script for Skill Engine JavaScript SDK
 *
 * This script:
 * 1. Compiles TypeScript to JavaScript (ES2022)
 * 2. Generates TypeScript declaration files
 * 3. Creates WASM Component using jco componentize (optional)
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import { mkdir } from 'fs/promises';
import { existsSync } from 'fs';

const execAsync = promisify(exec);

async function run(cmd, description) {
  console.log(`\n📦 ${description}...`);
  try {
    const { stdout, stderr } = await execAsync(cmd);
    if (stdout) console.log(stdout);
    if (stderr) console.error(stderr);
    console.log(`✅ ${description} complete`);
  } catch (error) {
    console.error(`❌ ${description} failed:`);
    console.error(error.stdout || error.message);
    throw error;
  }
}

async function build() {
  console.log('🚀 Building Skill Engine SDK...\n');

  // Ensure dist directory exists
  if (!existsSync('./dist')) {
    await mkdir('./dist', { recursive: true });
  }

  // Step 1: Compile TypeScript
  await run('npx tsc', 'TypeScript compilation');

  console.log('\n✨ SDK build complete!');
  console.log('\nOutput:');
  console.log('  - dist/index.js      (ES2022 JavaScript)');
  console.log('  - dist/types.js      (Type definitions runtime)');
  console.log('  - dist/index.d.ts    (TypeScript declarations)');
  console.log('  - dist/types.d.ts    (Type declarations)');

  console.log('\n📚 Next steps:');
  console.log('  1. Import in your skill: import { defineSkill } from "@skill-engine/sdk"');
  console.log('  2. Compile your skill to WASM using jco componentize');
  console.log('  3. Run with Skill Engine: skill run ./your-skill tool-name');
}

build().catch((error) => {
  console.error('\n💥 Build failed:', error.message);
  process.exit(1);
});
