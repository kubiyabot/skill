// Test WASI environment import

// Try importing WASI environment
let getEnvironment;
try {
  const wasiEnv = await import('wasi:cli/environment');
  getEnvironment = wasiEnv.getEnvironment;
  console.log('WASI environment imported successfully');
} catch (e) {
  console.error('Failed to import WASI environment:', e);
}

export function getMetadata() {
  let info = 'No WASI env';
  try {
    if (getEnvironment) {
      const env = getEnvironment();
      info = `WASI env: ${env.length} vars`;
    }
  } catch (e) {
    info = `Error: ${e.message}`;
  }

  return JSON.stringify({
    name: 'wasi-test',
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
