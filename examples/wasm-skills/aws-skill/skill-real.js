/**
 * AWS Skill - Real AWS SDK Integration
 *
 * This version uses the real AWS SDK and supports LocalStack for testing.
 *
 * Set LOCALSTACK=true to use LocalStack endpoint
 * Set AWS_ENDPOINT_URL for custom endpoints
 */

import { S3Client, ListObjectsV2Command, GetObjectCommand, PutObjectCommand } from '@aws-sdk/client-s3';
import { EC2Client, DescribeInstancesCommand } from '@aws-sdk/client-ec2';
import { LambdaClient, InvokeCommand } from '@aws-sdk/client-lambda';
import { readFile, writeFile } from 'fs/promises';

/**
 * Get skill metadata
 */
export function getMetadata() {
  return {
    name: "aws-skill",
    version: "1.0.0",
    description: "AWS service integration for S3, EC2, and Lambda operations",
    author: "Skill Engine Team"
  };
}

/**
 * Define available tools
 */
export function getTools() {
  return [
    {
      name: "s3-list",
      description: "List objects in an S3 bucket",
      parameters: [
        {
          name: "bucket",
          paramType: "string",
          description: "S3 bucket name",
          required: true
        },
        {
          name: "prefix",
          paramType: "string",
          description: "Filter objects by prefix (folder path)",
          required: false,
          defaultValue: ""
        },
        {
          name: "max-keys",
          paramType: "number",
          description: "Maximum number of objects to return",
          required: false,
          defaultValue: "1000"
        }
      ]
    },
    {
      name: "s3-upload",
      description: "Upload a file to S3",
      parameters: [
        {
          name: "bucket",
          paramType: "string",
          description: "Destination S3 bucket",
          required: true
        },
        {
          name: "key",
          paramType: "string",
          description: "Object key (path) in S3",
          required: true
        },
        {
          name: "file",
          paramType: "string",
          description: "Local file path to upload",
          required: true
        }
      ]
    },
    {
      name: "s3-download",
      description: "Download a file from S3",
      parameters: [
        {
          name: "bucket",
          paramType: "string",
          description: "Source S3 bucket",
          required: true
        },
        {
          name: "key",
          paramType: "string",
          description: "Object key (path) in S3",
          required: true
        },
        {
          name: "output",
          paramType: "string",
          description: "Local file path to save to",
          required: true
        }
      ]
    },
    {
      name: "ec2-list",
      description: "List EC2 instances",
      parameters: [
        {
          name: "state",
          paramType: "string",
          description: "Filter by instance state (running, stopped, terminated)",
          required: false,
          defaultValue: ""
        },
        {
          name: "tag",
          paramType: "string",
          description: "Filter by tag (format: key=value)",
          required: false,
          defaultValue: ""
        }
      ]
    },
    {
      name: "lambda-invoke",
      description: "Invoke a Lambda function",
      parameters: [
        {
          name: "function",
          paramType: "string",
          description: "Lambda function name or ARN",
          required: true
        },
        {
          name: "payload",
          paramType: "string",
          description: "JSON payload to send",
          required: false,
          defaultValue: "{}"
        },
        {
          name: "async",
          paramType: "boolean",
          description: "Invoke asynchronously",
          required: false,
          defaultValue: "false"
        }
      ]
    }
  ];
}

/**
 * Execute a tool
 */
export async function executeTool(toolName, argsJson) {
  try {
    const args = JSON.parse(argsJson);

    // Get AWS configuration
    const awsConfig = getAWSConfig();

    switch (toolName) {
      case "s3-list":
        return await handleS3List(args, awsConfig);
      case "s3-upload":
        return await handleS3Upload(args, awsConfig);
      case "s3-download":
        return await handleS3Download(args, awsConfig);
      case "ec2-list":
        return await handleEC2List(args, awsConfig);
      case "lambda-invoke":
        return await handleLambdaInvoke(args, awsConfig);
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
      errorMessage: `Error executing tool: ${error.message}\n${error.stack}`
    };
  }
}

/**
 * Validate configuration
 */
export async function validateConfig() {
  const accessKeyId = process.env.SKILL_AWS_ACCESS_KEY_ID;
  const secretAccessKey = process.env.SKILL_AWS_SECRET_ACCESS_KEY;
  const region = process.env.SKILL_REGION;

  if (!accessKeyId || !secretAccessKey) {
    return {
      err: "AWS credentials not configured. Run: skill config aws-skill"
    };
  }

  if (!region) {
    return {
      err: "AWS region not configured. Set 'region' in your config."
    };
  }

  return { ok: null };
}

// Helper Functions

/**
 * Get AWS configuration from environment
 */
function getAWSConfig() {
  const isLocalStack = process.env.LOCALSTACK === 'true';
  const customEndpoint = process.env.AWS_ENDPOINT_URL;

  const config = {
    region: process.env.SKILL_REGION || "us-east-1",
    credentials: {
      accessKeyId: process.env.SKILL_AWS_ACCESS_KEY_ID || "test",
      secretAccessKey: process.env.SKILL_AWS_SECRET_ACCESS_KEY || "test"
    }
  };

  // LocalStack or custom endpoint
  if (isLocalStack) {
    config.endpoint = customEndpoint || "http://localhost:4566";
    config.forcePathStyle = true; // Required for S3 with LocalStack
  } else if (customEndpoint) {
    config.endpoint = customEndpoint;
  }

  return config;
}

/**
 * Create S3 client with config
 */
function createS3Client(config) {
  return new S3Client(config);
}

/**
 * Create EC2 client with config
 */
function createEC2Client(config) {
  return new EC2Client(config);
}

/**
 * Create Lambda client with config
 */
function createLambdaClient(config) {
  return new LambdaClient(config);
}

// Tool Handlers

/**
 * Handle S3 list objects
 */
async function handleS3List(args, awsConfig) {
  const { bucket, prefix = "", "max-keys": maxKeys = 1000 } = args;

  if (!bucket) {
    return {
      success: false,
      output: "",
      errorMessage: "Parameter 'bucket' is required"
    };
  }

  try {
    const client = createS3Client(awsConfig);
    const command = new ListObjectsV2Command({
      Bucket: bucket,
      Prefix: prefix || undefined,
      MaxKeys: parseInt(maxKeys, 10)
    });

    const response = await client.send(command);
    const objects = response.Contents || [];

    let output = `\n📦 S3 Bucket: ${bucket}\n`;
    output += `🌍 Region: ${awsConfig.region}\n`;
    if (prefix) {
      output += `📁 Prefix: ${prefix}\n`;
    }
    output += `\n`;

    if (objects.length === 0) {
      output += `No objects found.\n`;
    } else {
      output += `${"Key".padEnd(50)} ${"Size".padEnd(12)} ${"Last Modified"}\n`;
      output += `${"-".repeat(80)}\n`;

      for (const obj of objects) {
        const size = formatBytes(obj.Size || 0);
        const date = obj.LastModified ? new Date(obj.LastModified).toLocaleDateString() : "N/A";
        const key = (obj.Key || "").padEnd(50).substring(0, 50);
        output += `${key} ${size.padEnd(12)} ${date}\n`;
      }
    }

    output += `\n✓ Found ${objects.length} objects\n`;

    return {
      success: true,
      output,
      errorMessage: null
    };
  } catch (error) {
    return {
      success: false,
      output: "",
      errorMessage: `S3 Error: ${error.message}`
    };
  }
}

/**
 * Handle S3 upload
 */
async function handleS3Upload(args, awsConfig) {
  const { bucket, key, file } = args;

  if (!bucket || !key || !file) {
    return {
      success: false,
      output: "",
      errorMessage: "Parameters 'bucket', 'key', and 'file' are required"
    };
  }

  try {
    // Read file
    const fileContent = await readFile(file);

    const client = createS3Client(awsConfig);
    const command = new PutObjectCommand({
      Bucket: bucket,
      Key: key,
      Body: fileContent
    });

    await client.send(command);

    const endpoint = awsConfig.endpoint || `https://s3.${awsConfig.region}.amazonaws.com`;
    const url = `${endpoint}/${bucket}/${key}`;

    const output = `
✓ File uploaded successfully

Source: ${file}
Destination: s3://${bucket}/${key}
Region: ${awsConfig.region}
Size: ${formatBytes(fileContent.length)}

URL: ${url}
`;

    return {
      success: true,
      output,
      errorMessage: null
    };
  } catch (error) {
    return {
      success: false,
      output: "",
      errorMessage: `S3 Upload Error: ${error.message}`
    };
  }
}

/**
 * Handle S3 download
 */
async function handleS3Download(args, awsConfig) {
  const { bucket, key, output: outputPath } = args;

  if (!bucket || !key || !outputPath) {
    return {
      success: false,
      output: "",
      errorMessage: "Parameters 'bucket', 'key', and 'output' are required"
    };
  }

  try {
    const client = createS3Client(awsConfig);
    const command = new GetObjectCommand({
      Bucket: bucket,
      Key: key
    });

    const response = await client.send(command);

    // Convert stream to buffer
    const chunks = [];
    for await (const chunk of response.Body) {
      chunks.push(chunk);
    }
    const buffer = Buffer.concat(chunks);

    // Write to file
    await writeFile(outputPath, buffer);

    const output = `
✓ File downloaded successfully

Source: s3://${bucket}/${key}
Destination: ${outputPath}
Region: ${awsConfig.region}
Size: ${formatBytes(buffer.length)}

File saved to local filesystem.
`;

    return {
      success: true,
      output,
      errorMessage: null
    };
  } catch (error) {
    return {
      success: false,
      output: "",
      errorMessage: `S3 Download Error: ${error.message}`
    };
  }
}

/**
 * Handle EC2 list instances
 */
async function handleEC2List(args, awsConfig) {
  const { state = "", tag = "" } = args;

  try {
    const client = createEC2Client(awsConfig);

    const filters = [];
    if (state) {
      filters.push({
        Name: 'instance-state-name',
        Values: [state]
      });
    }
    if (tag) {
      const [tagKey, tagValue] = tag.split('=');
      if (tagKey && tagValue) {
        filters.push({
          Name: `tag:${tagKey}`,
          Values: [tagValue]
        });
      }
    }

    const command = new DescribeInstancesCommand({
      Filters: filters.length > 0 ? filters : undefined
    });

    const response = await client.send(command);
    const instances = [];

    // Flatten reservations to get all instances
    for (const reservation of response.Reservations || []) {
      instances.push(...(reservation.Instances || []));
    }

    let output = `\n💻 EC2 Instances\n`;
    output += `🌍 Region: ${awsConfig.region}\n`;
    if (state) output += `📊 State Filter: ${state}\n`;
    if (tag) output += `🏷️  Tag Filter: ${tag}\n`;
    output += `\n`;

    if (instances.length === 0) {
      output += `No instances found.\n`;
    } else {
      for (const instance of instances) {
        const name = instance.Tags?.find(t => t.Key === "Name")?.Value || "N/A";
        const uptime = instance.LaunchTime
          ? Math.floor((Date.now() - new Date(instance.LaunchTime).getTime()) / 86400000)
          : 0;

        output += `Instance: ${instance.InstanceId}\n`;
        output += `  Name: ${name}\n`;
        output += `  Type: ${instance.InstanceType}\n`;
        output += `  State: ${instance.State?.Name || "unknown"}\n`;
        output += `  IP: ${instance.PrivateIpAddress || "N/A"}\n`;
        output += `  Uptime: ${uptime} days\n`;
        output += `\n`;
      }
    }

    output += `✓ Found ${instances.length} instances\n`;

    return {
      success: true,
      output,
      errorMessage: null
    };
  } catch (error) {
    return {
      success: false,
      output: "",
      errorMessage: `EC2 Error: ${error.message}`
    };
  }
}

/**
 * Handle Lambda invoke
 */
async function handleLambdaInvoke(args, awsConfig) {
  const { function: functionName, payload = "{}", async: isAsync = false } = args;

  if (!functionName) {
    return {
      success: false,
      output: "",
      errorMessage: "Parameter 'function' is required"
    };
  }

  // Validate JSON payload
  try {
    JSON.parse(payload);
  } catch (e) {
    return {
      success: false,
      output: "",
      errorMessage: `Invalid JSON payload: ${e.message}`
    };
  }

  try {
    const client = createLambdaClient(awsConfig);
    const command = new InvokeCommand({
      FunctionName: functionName,
      Payload: new TextEncoder().encode(payload),
      InvocationType: isAsync ? 'Event' : 'RequestResponse'
    });

    const response = await client.send(command);

    let output = `\n⚡ Lambda Function Invoked\n`;
    output += `📦 Function: ${functionName}\n`;
    output += `🌍 Region: ${awsConfig.region}\n`;
    output += `🔄 Type: ${isAsync ? "Async" : "Sync"}\n`;
    output += `\n`;
    output += `Response:\n`;
    output += `  Status: ${response.StatusCode}\n`;
    output += `  ExecutedVersion: ${response.ExecutedVersion || "$LATEST"}\n`;

    if (response.Payload) {
      output += `\n`;
      output += `Payload:\n`;
      const payloadStr = new TextDecoder().decode(response.Payload);
      try {
        const payloadObj = JSON.parse(payloadStr);
        output += JSON.stringify(payloadObj, null, 2);
      } catch {
        output += payloadStr;
      }
    }

    if (response.FunctionError) {
      output += `\n\n⚠️  Function Error: ${response.FunctionError}\n`;
    }

    output += `\n\n✓ Function executed successfully\n`;

    return {
      success: true,
      output,
      errorMessage: null
    };
  } catch (error) {
    return {
      success: false,
      output: "",
      errorMessage: `Lambda Error: ${error.message}`
    };
  }
}

// Utility Functions

/**
 * Format bytes to human-readable string
 */
function formatBytes(bytes) {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
}
