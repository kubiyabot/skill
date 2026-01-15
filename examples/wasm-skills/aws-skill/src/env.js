// Environment variable access for WASI
// Jco/StarlingMonkey provides WASI environment access

// Cache environment on module load
let envCache = null;

function initializeEnv() {
  if (envCache) return envCache;

  envCache = {};

  // Try to get environment from WASI
  try {
    // In jco, environment is available via import
    // For now, we'll try the globalThis approach
    if (typeof globalThis !== 'undefined') {
      // Check for process.env at module init time
      if (globalThis.process && globalThis.process.env) {
        envCache = globalThis.process.env;
      }
    }
  } catch (e) {
    console.error('Failed to initialize environment:', e);
  }

  return envCache;
}

// Initialize on module load
initializeEnv();

export function getEnv(key, defaultValue = '') {
  const env = initializeEnv();
  return env[key] || defaultValue;
}

export function hasEnv(key) {
  const env = initializeEnv();
  return key in env;
}

export function getAllEnv() {
  return initializeEnv();
}
