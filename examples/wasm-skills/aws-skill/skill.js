/**
 * AWS Skill - Amazon Web Services Integration
 *
 * Provides secure access to AWS services including S3, EC2, and Lambda.
 * Run directly without build steps:
 *   skill run ./examples/aws-skill s3-list bucket=my-bucket
 */

// Note: In a real implementation, you would import AWS SDK:
// import { S3Client, ListObjectsV2Command, GetObjectCommand, PutObjectCommand } from '@aws-sdk/client-s3';
// import { EC2Client, DescribeInstancesCommand } from '@aws-sdk/client-ec2';
// import { LambdaClient, InvokeCommand } from '@aws-sdk/client-lambda';

// For this example, we'll simulate the AWS SDK calls

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

    // Get AWS credentials from environment
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
      errorMessage: `Error executing tool: ${error.message}`
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

// Tool Handlers

/**
 * Get AWS configuration from environment
 */
function getAWSConfig() {
  return {
    accessKeyId: process.env.SKILL_AWS_ACCESS_KEY_ID || "",
    secretAccessKey: process.env.SKILL_AWS_SECRET_ACCESS_KEY || "",
    region: process.env.SKILL_REGION || "us-east-1"
  };
}

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

  // In real implementation, use AWS SDK:
  // const client = new S3Client({ region: awsConfig.region, credentials: { ... } });
  // const command = new ListObjectsV2Command({ Bucket: bucket, Prefix: prefix, MaxKeys: maxKeys });
  // const response = await client.send(command);

  // Simulated response for demonstration
  const simulatedObjects = [
    { Key: `${prefix}file1.txt`, Size: 1024, LastModified: new Date().toISOString() },
    { Key: `${prefix}file2.jpg`, Size: 204800, LastModified: new Date().toISOString() },
    { Key: `${prefix}data/report.pdf`, Size: 512000, LastModified: new Date().toISOString() }
  ];

  let output = `\n📦 S3 Bucket: ${bucket}\n`;
  output += `🌍 Region: ${awsConfig.region}\n`;
  if (prefix) {
    output += `📁 Prefix: ${prefix}\n`;
  }
  output += `\n`;

  output += `${"Key".padEnd(50)} ${"Size".padEnd(12)} ${"Last Modified"}\n`;
  output += `${"-".repeat(80)}\n`;

  for (const obj of simulatedObjects) {
    const size = formatBytes(obj.Size);
    const date = new Date(obj.LastModified).toLocaleDateString();
    output += `${obj.Key.padEnd(50)} ${size.padEnd(12)} ${date}\n`;
  }

  output += `\n✓ Found ${simulatedObjects.length} objects\n`;

  return {
    success: true,
    output,
    errorMessage: null
  };
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

  // In real implementation:
  // const fileContent = await fs.readFile(file);
  // const client = new S3Client({ region: awsConfig.region, credentials: { ... } });
  // const command = new PutObjectCommand({ Bucket: bucket, Key: key, Body: fileContent });
  // await client.send(command);

  const output = `
✓ File uploaded successfully

Source: ${file}
Destination: s3://${bucket}/${key}
Region: ${awsConfig.region}

The file is now available at:
https://${bucket}.s3.${awsConfig.region}.amazonaws.com/${key}
`;

  return {
    success: true,
    output,
    errorMessage: null
  };
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

  // In real implementation:
  // const client = new S3Client({ region: awsConfig.region, credentials: { ... } });
  // const command = new GetObjectCommand({ Bucket: bucket, Key: key });
  // const response = await client.send(command);
  // await fs.writeFile(outputPath, response.Body);

  const output = `
✓ File downloaded successfully

Source: s3://${bucket}/${key}
Destination: ${outputPath}
Region: ${awsConfig.region}

File saved to local filesystem.
`;

  return {
    success: true,
    output,
    errorMessage: null
  };
}

/**
 * Handle EC2 list instances
 */
async function handleEC2List(args, awsConfig) {
  const { state = "", tag = "" } = args;

  // In real implementation:
  // const client = new EC2Client({ region: awsConfig.region, credentials: { ... } });
  // const filters = [];
  // if (state) filters.push({ Name: 'instance-state-name', Values: [state] });
  // if (tag) { ... }
  // const command = new DescribeInstancesCommand({ Filters: filters });
  // const response = await client.send(command);

  // Simulated response
  const simulatedInstances = [
    {
      InstanceId: "i-1234567890abcdef0",
      InstanceType: "t3.medium",
      State: "running",
      PrivateIpAddress: "10.0.1.50",
      LaunchTime: new Date(Date.now() - 86400000).toISOString(),
      Tags: [{ Key: "Name", Value: "web-server-1" }]
    },
    {
      InstanceId: "i-0987654321fedcba0",
      InstanceType: "t3.large",
      State: "running",
      PrivateIpAddress: "10.0.1.51",
      LaunchTime: new Date(Date.now() - 172800000).toISOString(),
      Tags: [{ Key: "Name", Value: "api-server-1" }]
    }
  ];

  let output = `\n💻 EC2 Instances\n`;
  output += `🌍 Region: ${awsConfig.region}\n`;
  if (state) output += `📊 State Filter: ${state}\n`;
  if (tag) output += `🏷️  Tag Filter: ${tag}\n`;
  output += `\n`;

  for (const instance of simulatedInstances) {
    const name = instance.Tags.find(t => t.Key === "Name")?.Value || "N/A";
    const uptime = Math.floor((Date.now() - new Date(instance.LaunchTime).getTime()) / 86400000);

    output += `Instance: ${instance.InstanceId}\n`;
    output += `  Name: ${name}\n`;
    output += `  Type: ${instance.InstanceType}\n`;
    output += `  State: ${instance.State}\n`;
    output += `  IP: ${instance.PrivateIpAddress}\n`;
    output += `  Uptime: ${uptime} days\n`;
    output += `\n`;
  }

  output += `✓ Found ${simulatedInstances.length} instances\n`;

  return {
    success: true,
    output,
    errorMessage: null
  };
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

  // In real implementation:
  // const client = new LambdaClient({ region: awsConfig.region, credentials: { ... } });
  // const command = new InvokeCommand({
  //   FunctionName: functionName,
  //   Payload: payload,
  //   InvocationType: isAsync ? 'Event' : 'RequestResponse'
  // });
  // const response = await client.send(command);

  // Simulated response
  const simulatedResponse = {
    StatusCode: 200,
    ExecutedVersion: "$LATEST",
    Payload: JSON.stringify({ result: "success", data: "Processed 42 items" })
  };

  let output = `\n⚡ Lambda Function Invoked\n`;
  output += `📦 Function: ${functionName}\n`;
  output += `🌍 Region: ${awsConfig.region}\n`;
  output += `🔄 Type: ${isAsync ? "Async" : "Sync"}\n`;
  output += `\n`;
  output += `Response:\n`;
  output += `  Status: ${simulatedResponse.StatusCode}\n`;
  output += `  Version: ${simulatedResponse.ExecutedVersion}\n`;
  output += `\n`;
  output += `Payload:\n`;
  output += JSON.stringify(JSON.parse(simulatedResponse.Payload), null, 2);
  output += `\n\n✓ Function executed successfully\n`;

  return {
    success: true,
    output,
    errorMessage: null
  };
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
