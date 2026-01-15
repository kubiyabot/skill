---
name: aws-skill
version: 1.0.0
description: AWS service integration for S3, EC2, and Lambda operations
author: Skill Engine Team
allowed-tools: ["s3-list", "s3-upload", "s3-download", "ec2-list", "lambda-invoke"]
---

# AWS Skill

Interact with Amazon Web Services (AWS) directly from the command line or through MCP. This skill provides secure, authenticated access to core AWS services including S3 storage, EC2 compute, and Lambda serverless functions.

## What is AWS?

Amazon Web Services (AWS) is the world's most comprehensive cloud computing platform, offering over 200 services for compute, storage, databases, networking, and more. This skill focuses on the most commonly used services:

- **S3 (Simple Storage Service)**: Object storage for files, backups, and static assets
- **EC2 (Elastic Compute Cloud)**: Virtual servers for running applications
- **Lambda**: Serverless compute for running code without managing servers

## When to Use This Skill

**Use this skill when you need to**:
- List, upload, or download files from S3 buckets
- Check running EC2 instances and their status
- Invoke Lambda functions programmatically
- Automate AWS operations without switching to the AWS Console

**Common scenarios**:
- "Upload this build artifact to S3"
- "Show me all running EC2 instances in production"
- "Invoke the data-processing Lambda function"
- "Download the latest backup from S3"

## Prerequisites

1. **AWS Account**: You need an active AWS account
2. **AWS Credentials**: IAM access key ID and secret access key
3. **IAM Permissions**: Your credentials need appropriate permissions for the services you'll use

### Getting AWS Credentials

1. Log into AWS Console
2. Navigate to IAM → Users → Your User
3. Security Credentials tab → Create Access Key
4. Choose "Command Line Interface (CLI)"
5. Save the Access Key ID and Secret Access Key

**Security Note**: Never commit AWS credentials to source control. This skill stores them securely in your system keychain.

## Configuration

Create a `skill.config.toml` file or configure after installation:

```toml
[config]
aws_access_key_id = "AKIAIOSFODNN7EXAMPLE"
aws_secret_access_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
region = "us-east-1"
```

Or configure interactively:

```bash
skill config aws-skill --instance prod
# Follow the prompts to enter credentials
```

### Multi-Account Support

You can install multiple instances for different AWS accounts:

```bash
# Production account
skill install aws-skill --instance prod
skill config aws-skill --instance prod

# Staging account
skill install aws-skill --instance staging
skill config aws-skill --instance staging

# Use specific instance
skill run aws-skill --instance prod s3-list my-prod-bucket
skill run aws-skill --instance staging s3-list my-staging-bucket
```

## Available Tools

### s3-list
List objects in an S3 bucket

**Parameters**:
- `bucket` (required): Name of the S3 bucket
- `prefix` (optional): Filter objects by prefix (folder path)
- `max-keys` (optional): Maximum number of objects to return (default: 1000)

**Example**:
```bash
skill run ./aws-skill s3-list bucket=my-bucket
skill run ./aws-skill s3-list bucket=my-bucket prefix=logs/2024/
```

### s3-upload
Upload a file to S3

**Parameters**:
- `bucket` (required): Destination S3 bucket
- `key` (required): Object key (path) in S3
- `file` (required): Local file path to upload

**Example**:
```bash
skill run ./aws-skill s3-upload bucket=my-bucket key=data/file.txt file=./local-file.txt
```

### s3-download
Download a file from S3

**Parameters**:
- `bucket` (required): Source S3 bucket
- `key` (required): Object key (path) in S3
- `output` (required): Local file path to save to

**Example**:
```bash
skill run ./aws-skill s3-download bucket=my-bucket key=data/file.txt output=./downloaded-file.txt
```

### ec2-list
List EC2 instances

**Parameters**:
- `state` (optional): Filter by instance state (running, stopped, terminated)
- `tag` (optional): Filter by tag (format: key=value)

**Example**:
```bash
skill run ./aws-skill ec2-list
skill run ./aws-skill ec2-list state=running
skill run ./aws-skill ec2-list tag=Environment=production
```

### lambda-invoke
Invoke a Lambda function

**Parameters**:
- `function` (required): Lambda function name or ARN
- `payload` (optional): JSON payload to send (default: {})
- `async` (optional): Invoke asynchronously (default: false)

**Example**:
```bash
skill run ./aws-skill lambda-invoke function=my-function
skill run ./aws-skill lambda-invoke function=data-processor payload='{"key":"value"}'
```

## Security & Best Practices

### Credential Security
- Credentials are encrypted in your system keychain
- Never log or print credentials
- Use IAM roles with least-privilege permissions
- Rotate access keys regularly

### IAM Permissions Required

**For S3 operations**:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:ListBucket",
        "s3:GetObject",
        "s3:PutObject"
      ],
      "Resource": [
        "arn:aws:s3:::your-bucket-name",
        "arn:aws:s3:::your-bucket-name/*"
      ]
    }
  ]
}
```

**For EC2 operations**:
```json
{
  "Effect": "Allow",
  "Action": [
    "ec2:DescribeInstances",
    "ec2:DescribeInstanceStatus"
  ],
  "Resource": "*"
}
```

**For Lambda operations**:
```json
{
  "Effect": "Allow",
  "Action": "lambda:InvokeFunction",
  "Resource": "arn:aws:lambda:*:*:function:*"
}
```

## Troubleshooting

### "Access Denied" errors
- Check IAM permissions for your credentials
- Verify the resource (bucket, instance, function) exists
- Ensure you're using the correct region

### "Region not specified" errors
- Set `region` in your config
- Or use AWS_DEFAULT_REGION environment variable

### "Bucket does not exist" errors
- Verify bucket name (no typos)
- Check if bucket is in the configured region
- S3 bucket names are globally unique

## Development

This skill is written in JavaScript and can be modified directly. After changes, the runtime automatically recompiles:

```bash
# Edit skill.js
vim examples/aws-skill/skill.js

# Run immediately - auto-recompiles if changed
skill run ./examples/aws-skill s3-list bucket=test
```

### Adding New AWS Services

To add support for additional AWS services:

1. Add the service client to the implementation
2. Define new tools in `getTools()`
3. Implement handlers in `executeTool()`
4. Update this SKILL.md with documentation

## Resources

- [AWS SDK for JavaScript](https://docs.aws.amazon.com/AWSJavaScriptSDK/v3/latest/)
- [AWS IAM Best Practices](https://docs.aws.amazon.com/IAM/latest/UserGuide/best-practices.html)
- [S3 User Guide](https://docs.aws.amazon.com/s3/)
- [EC2 User Guide](https://docs.aws.amazon.com/ec2/)
- [Lambda Developer Guide](https://docs.aws.amazon.com/lambda/)
