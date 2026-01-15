// Simplest possible environment access test

export function getMetadata() {
  // Try accessing environment at function call time, not module load time
  let envInfo = 'Testing...';

  try {
    // Check what's available
    const checks = [];

    if (typeof process !== 'undefined') {
      checks.push('process exists');
      if (process.env) {
        const keys = Object.keys(process.env);
        checks.push(`process.env has ${keys.length} keys`);
        if ('SKILL_AWS_ACCESS_KEY_ID' in process.env) {
          checks.push('AWS_ACCESS_KEY_ID found!');
          envInfo = `SUCCESS: ${process.env.SKILL_AWS_ACCESS_KEY_ID}`;
        } else {
          envInfo = `Keys: ${keys.join(', ')}`;
        }
      } else {
        checks.push('process.env is falsy');
      }
    } else {
      checks.push('process is undefined');
    }

    if (checks.length > 0 && envInfo === 'Testing...') {
      envInfo = checks.join(' | ');
    }
  } catch (e) {
    envInfo = `Error: ${e.message}`;
  }

  return JSON.stringify({
    name: 'simple-test',
    version: '1.0.0',
    description: envInfo,
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
