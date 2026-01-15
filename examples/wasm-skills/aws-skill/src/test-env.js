// Test environment variable access

console.log('=== Environment Test ===');
console.log('typeof process:', typeof process);
console.log('typeof globalThis:', typeof globalThis);

if (typeof process !== 'undefined') {
  console.log('process exists');
  console.log('typeof process.env:', typeof process.env);
  if (process.env) {
    const keys = Object.keys(process.env);
    console.log('process.env keys:', keys.length);
    console.log('First 10 keys:', keys.slice(0, 10));
    console.log('SKILL_AWS_ACCESS_KEY_ID:', process.env.SKILL_AWS_ACCESS_KEY_ID);
  }
}

export function getMetadata() {
  let info = 'No process';
  if (typeof process !== 'undefined' && process.env) {
    const keys = Object.keys(process.env);
    info = `Found ${keys.length} env vars`;
    if (keys.includes('SKILL_AWS_ACCESS_KEY_ID')) {
      info += ` - AWS key found!`;
    }
  }

  return JSON.stringify({
    name: 'test',
    version: '1.0.0',
    description: info,
    author: 'Test'
  });
}

export function getTools() {
  return JSON.stringify([]);
}

export function executeTool(name, args) {
  return JSON.stringify({err: 'not implemented'});
}

export function validateConfig(config) {
  return JSON.stringify({ok: null});
}
