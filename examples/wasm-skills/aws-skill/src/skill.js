// AWS Skill - Standalone version for WASM compilation
// This demonstrates a real-world skill with AWS SDK integration

// Environment variable access - cached on module load
const ENV_CACHE = (() => {
  try {
    // Access at module initialization time when WASI provides it
    if (typeof process !== 'undefined' && process.env) {
      return process.env;
    }
    if (typeof globalThis !== 'undefined' && globalThis.process && globalThis.process.env) {
      return globalThis.process.env;
    }
  } catch (e) {
    console.error('Failed to access environment:', e);
  }
  return {};
})();

function getEnv(key, defaultValue = '') {
  return ENV_CACHE[key] || defaultValue;
}

// Export functions matching WIT interface
export function getMetadata() {
  return JSON.stringify({
    name: 'aws-skill',
    version: '1.0.0',
    description: 'AWS operations for S3, EC2, and Lambda',
    author: 'Skill Engine Team'
  });
}

export function getTools() {
  return JSON.stringify([
    {
      name: 's3-list-buckets',
      description: 'List all S3 buckets in the account',
      parameters: []
    },
    {
      name: 's3-list-objects',
      description: 'List objects in an S3 bucket',
      parameters: [
        { name: 'bucket', paramType: 'string', description: 'Bucket name', required: true }
      ]
    },
    {
      name: 's3-get-object',
      description: 'Get object content from S3',
      parameters: [
        { name: 'bucket', paramType: 'string', description: 'Bucket name', required: true },
        { name: 'key', paramType: 'string', description: 'Object key', required: true }
      ]
    },
    {
      name: 's3-put-object',
      description: 'Upload object to S3',
      parameters: [
        { name: 'bucket', paramType: 'string', description: 'Bucket name', required: true },
        { name: 'key', paramType: 'string', description: 'Object key', required: true },
        { name: 'content', paramType: 'string', description: 'Content to upload', required: true }
      ]
    },
    {
      name: 'ec2-list-instances',
      description: 'List EC2 instances',
      parameters: []
    },
    {
      name: 'lambda-list-functions',
      description: 'List Lambda functions',
      parameters: []
    }
  ]);
}

export function executeTool(toolName, argsJson) {
  try {
    const args = JSON.parse(argsJson);

    // Get AWS credentials from environment variables
    const accessKeyId = getEnv('SKILL_AWS_ACCESS_KEY_ID', '');
    const secretAccessKey = getEnv('SKILL_AWS_SECRET_ACCESS_KEY', '');
    const region = getEnv('SKILL_AWS_REGION', 'us-east-1');

    if (!accessKeyId || !secretAccessKey) {
      return JSON.stringify({
        err: 'AWS credentials not configured. Set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY.'
      });
    }

    switch (toolName) {
      case 's3-list-buckets':
        return s3ListBuckets(accessKeyId, secretAccessKey, region);

      case 's3-list-objects':
        if (!args.bucket) {
          return JSON.stringify({ err: 'bucket parameter is required' });
        }
        return s3ListObjects(accessKeyId, secretAccessKey, region, args.bucket);

      case 's3-get-object':
        if (!args.bucket || !args.key) {
          return JSON.stringify({ err: 'bucket and key parameters are required' });
        }
        return s3GetObject(accessKeyId, secretAccessKey, region, args.bucket, args.key);

      case 's3-put-object':
        if (!args.bucket || !args.key || !args.content) {
          return JSON.stringify({ err: 'bucket, key, and content parameters are required' });
        }
        return s3PutObject(accessKeyId, secretAccessKey, region, args.bucket, args.key, args.content);

      case 'ec2-list-instances':
        return ec2ListInstances(accessKeyId, secretAccessKey, region);

      case 'lambda-list-functions':
        return lambdaListFunctions(accessKeyId, secretAccessKey, region);

      default:
        return JSON.stringify({ err: 'Unknown tool: ' + toolName });
    }
  } catch (error) {
    return JSON.stringify({ err: 'Execution error: ' + error.message });
  }
}

export function validateConfig(configJson) {
  try {
    const config = JSON.parse(configJson);

    // Validate required AWS credentials
    const requiredFields = ['AWS_ACCESS_KEY_ID', 'AWS_SECRET_ACCESS_KEY'];
    const missing = [];
    for (const field of requiredFields) {
      if (!config[field]) {
        missing.push(field);
      }
    }

    if (missing.length > 0) {
      return JSON.stringify({
        err: 'Missing required configuration: ' + missing.join(', ')
      });
    }

    return JSON.stringify({ ok: null });
  } catch (error) {
    return JSON.stringify({ err: 'Validation error: ' + error.message });
  }
}

// AWS SDK Integration (simplified for WASM - using mock data for demo)
// In production, this would make actual AWS API calls using fetch

function s3ListBuckets(accessKeyId, secretAccessKey, region) {
  const buckets = [
    { name: 'my-app-bucket', creationDate: '2024-01-15T10:30:00Z' },
    { name: 'data-backup-bucket', creationDate: '2024-02-20T14:20:00Z' },
    { name: 'logs-archive-bucket', creationDate: '2024-03-10T09:15:00Z' }
  ];

  let output = 'Found ' + buckets.length + ' S3 buckets:\n\n';
  output += 'Using AWS Config:\n';
  output += '  Access Key: ' + accessKeyId + '\n';
  output += '  Secret Key: ' + secretAccessKey.substring(0, 10) + '...\n';
  output += '  Region: ' + region + '\n\n';

  for (const b of buckets) {
    output += '  • ' + b.name + ' (created: ' + b.creationDate + ')\n';
  }

  return JSON.stringify({
    ok: {
      success: true,
      output: output,
      errorMessage: null
    }
  });
}

function s3ListObjects(accessKeyId, secretAccessKey, region, bucket) {
  const objects = [
    { key: 'data/2024/file1.json', size: 1024, lastModified: '2024-12-01T10:00:00Z' },
    { key: 'data/2024/file2.json', size: 2048, lastModified: '2024-12-05T15:30:00Z' },
    { key: 'logs/app.log', size: 4096, lastModified: '2024-12-18T08:20:00Z' }
  ];

  let output = 'Objects in bucket \'' + bucket + '\':\n\n';
  for (const o of objects) {
    output += '  • ' + o.key + ' (' + o.size + ' bytes, modified: ' + o.lastModified + ')\n';
  }

  return JSON.stringify({
    ok: {
      success: true,
      output: output,
      errorMessage: null
    }
  });
}

function s3GetObject(accessKeyId, secretAccessKey, region, bucket, key) {
  const content = 'This is the content of ' + key + ' from ' + bucket + '.\nLast accessed: ' + new Date().toISOString();

  return JSON.stringify({
    ok: {
      success: true,
      output: content,
      errorMessage: null
    }
  });
}

function s3PutObject(accessKeyId, secretAccessKey, region, bucket, key, content) {
  const output = 'Successfully uploaded to s3://' + bucket + '/' + key + '\nSize: ' + content.length + ' bytes';

  return JSON.stringify({
    ok: {
      success: true,
      output: output,
      errorMessage: null
    }
  });
}

function ec2ListInstances(accessKeyId, secretAccessKey, region) {
  const instances = [
    { id: 'i-1234567890abcdef0', type: 't3.micro', state: 'running', publicIp: '54.123.45.67' },
    { id: 'i-0fedcba9876543210', type: 't3.small', state: 'stopped', publicIp: null }
  ];

  let output = 'EC2 Instances in ' + region + ':\n\n';
  for (const i of instances) {
    const ip = i.publicIp ? ' - ' + i.publicIp : '';
    output += '  • ' + i.id + ' (' + i.type + ') - ' + i.state + ip + '\n';
  }

  return JSON.stringify({
    ok: {
      success: true,
      output: output,
      errorMessage: null
    }
  });
}

function lambdaListFunctions(accessKeyId, secretAccessKey, region) {
  const functions = [
    { name: 'api-handler', runtime: 'nodejs20.x', memorySize: 256, lastModified: '2024-12-15T10:30:00Z' },
    { name: 'data-processor', runtime: 'python3.12', memorySize: 512, lastModified: '2024-12-10T14:20:00Z' },
    { name: 'image-resizer', runtime: 'nodejs20.x', memorySize: 1024, lastModified: '2024-12-01T09:15:00Z' }
  ];

  let output = 'Lambda Functions in ' + region + ':\n\n';
  for (const f of functions) {
    output += '  • ' + f.name + ' (' + f.runtime + ') - ' + f.memorySize + 'MB\n';
  }

  return JSON.stringify({
    ok: {
      success: true,
      output: output,
      errorMessage: null
    }
  });
}
