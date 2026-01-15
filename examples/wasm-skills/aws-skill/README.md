# AWS Skill

A comprehensive AWS integration skill for Skill Engine, providing access to S3, EC2, and Lambda services.

## Quick Start

```bash
# Run directly from directory (no build needed!)
skill run ./examples/aws-skill s3-list bucket=my-bucket

# List running EC2 instances
skill run ./examples/aws-skill ec2-list state=running

# Invoke a Lambda function
skill run ./examples/aws-skill lambda-invoke function=my-function payload='{"key":"value"}'
```

## Features

- **Zero Configuration**: Just write JavaScript and run
- **Auto-Compilation**: Runtime compiles to WASM on first use (~3 seconds)
- **Cached Execution**: Subsequent runs use cached WASM (<100ms startup)
- **Secure Credentials**: AWS keys stored in system keychain
- **Multi-Account**: Support for multiple AWS accounts via instances

## Available Tools

### S3 Operations
- `s3-list` - List objects in a bucket
- `s3-upload` - Upload files to S3
- `s3-download` - Download files from S3

### EC2 Operations
- `ec2-list` - List and filter EC2 instances

### Lambda Operations
- `lambda-invoke` - Invoke Lambda functions synchronously or asynchronously

## Configuration

### Method 1: Config File

Create `skill.config.toml`:

```toml
[config]
aws_access_key_id = "AKIAIOSFODNN7EXAMPLE"
aws_secret_access_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
region = "us-east-1"
```

### Method 2: Interactive Wizard

```bash
# Install skill first
skill install ./examples/aws-skill --instance prod

# Configure interactively
skill config aws-skill --instance prod
# Follow prompts to enter credentials
```

### Method 3: Environment Variables

```bash
export SKILL_AWS_ACCESS_KEY_ID="AKIA..."
export SKILL_AWS_SECRET_ACCESS_KEY="..."
export SKILL_REGION="us-east-1"

skill run ./examples/aws-skill s3-list bucket=my-bucket
```

## Multi-Account Usage

Support for multiple AWS accounts:

```bash
# Install for production account
skill install ./examples/aws-skill --instance prod
skill config aws-skill --instance prod
# Enter production credentials

# Install for staging account
skill install ./examples/aws-skill --instance staging
skill config aws-skill --instance staging
# Enter staging credentials

# Use specific account
skill run aws-skill --instance prod s3-list bucket=prod-data
skill run aws-skill --instance staging s3-list bucket=staging-data
```

## Examples

### S3 Examples

```bash
# List all objects in a bucket
skill run ./examples/aws-skill s3-list bucket=my-bucket

# List objects with prefix (like a folder)
skill run ./examples/aws-skill s3-list bucket=my-bucket prefix=logs/2024/

# Upload a file
skill run ./examples/aws-skill s3-upload \
  bucket=my-bucket \
  key=uploads/file.txt \
  file=./local-file.txt

# Download a file
skill run ./examples/aws-skill s3-download \
  bucket=my-bucket \
  key=data/report.pdf \
  output=./report.pdf
```

### EC2 Examples

```bash
# List all instances
skill run ./examples/aws-skill ec2-list

# List only running instances
skill run ./examples/aws-skill ec2-list state=running

# Filter by tag
skill run ./examples/aws-skill ec2-list tag=Environment=production
```

### Lambda Examples

```bash
# Invoke function with no payload
skill run ./examples/aws-skill lambda-invoke function=my-function

# Invoke with JSON payload
skill run ./examples/aws-skill lambda-invoke \
  function=data-processor \
  payload='{"action":"process","items":["a","b","c"]}'

# Invoke asynchronously
skill run ./examples/aws-skill lambda-invoke \
  function=email-sender \
  payload='{"to":"user@example.com","subject":"Hello"}' \
  async=true
```

## Security

### Credential Storage

- Credentials are stored in your system's secure keychain:
  - **macOS**: Keychain Access
  - **Windows**: Credential Manager
  - **Linux**: Secret Service API (gnome-keyring, KWallet)

- Secrets are encrypted at rest
- Never logged or printed to console
- Cleared from memory after use

### IAM Best Practices

1. **Use IAM Users, not Root**: Create dedicated IAM users for CLI access
2. **Least Privilege**: Grant only the permissions you need
3. **Rotate Keys**: Change access keys regularly
4. **Enable MFA**: Add multi-factor authentication to your IAM user

### Required IAM Permissions

See [SKILL.md](./SKILL.md) for detailed IAM policy examples.

## Development

This skill is written in pure JavaScript and can be modified directly:

```bash
# Edit the skill
vim examples/aws-skill/skill.js

# Run immediately - automatically recompiles if changed
skill run ./examples/aws-skill s3-list bucket=test
```

### Adding New AWS Services

1. Import the AWS SDK client (if using real AWS SDK)
2. Add tool definition to `getTools()`
3. Implement handler function
4. Add to switch statement in `executeTool()`
5. Update SKILL.md documentation

## Real AWS SDK Implementation

This example uses simulated responses for demonstration. To connect to real AWS:

1. Uncomment the AWS SDK imports at the top of `skill.js`
2. Ensure AWS SDK packages are available during compilation
3. Replace simulated responses with real SDK calls

The skill structure is already set up for real AWS integration.

## Troubleshooting

### "Credentials not configured"
- Run `skill config aws-skill` to set up credentials
- Or create a `skill.config.toml` file with your credentials

### "Region not specified"
- Set `region` in your config file
- Or use `SKILL_REGION` environment variable

### "Access Denied" errors
- Check IAM permissions for your credentials
- Verify the resource (bucket, instance, function) exists
- Ensure you're using the correct region

## Documentation

See [SKILL.md](./SKILL.md) for comprehensive documentation including:
- What is AWS and when to use this skill
- Detailed tool reference
- Security best practices
- IAM permission requirements
- Troubleshooting guide

## License

MIT License - Part of Skill Engine project
