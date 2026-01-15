# AWS Skill Testing Guide

This guide explains how to test the AWS skill end-to-end using LocalStack.

## Prerequisites

1. **Docker** - For running LocalStack
2. **Node.js** - For AWS SDK
3. **AWS CLI** (optional) - For manual verification

## Quick Start

### Option 1: Automated Test (Recommended)

Run the comprehensive test script:

```bash
cd examples/aws-skill
./test-localstack.sh
```

This will:
1. Check prerequisites
2. Install dependencies
3. Start LocalStack
4. Run all tests (S3, EC2, Lambda)
5. Clean up automatically

### Option 2: Manual Testing

#### Step 1: Start LocalStack

```bash
cd examples/aws-skill
docker-compose up -d
```

Wait for LocalStack to be ready:
```bash
curl http://localhost:4566/_localstack/health
```

#### Step 2: Set Environment Variables

```bash
export LOCALSTACK=true
export AWS_ENDPOINT_URL="http://localhost:4566"
export SKILL_AWS_ACCESS_KEY_ID="test"
export SKILL_AWS_SECRET_ACCESS_KEY="test"
export SKILL_REGION="us-east-1"
```

#### Step 3: Install Dependencies

```bash
npm install
```

#### Step 4: Create Test Bucket

Using AWS CLI:
```bash
aws --endpoint-url=http://localhost:4566 s3 mb s3://test-bucket
```

Or using the skill itself (after first run):
```bash
# Note: Requires bucket creation tool (not yet implemented)
```

#### Step 5: Run Tests

**Test S3 List (Empty Bucket)**:
```bash
node -e "
import('./skill-real.js').then(async (skill) => {
  const result = await skill.executeTool('s3-list', JSON.stringify({
    bucket: 'test-bucket'
  }));
  console.log(result.output);
});
"
```

**Test S3 Upload**:
```bash
# Create test file
echo 'Test data' > test.txt

# Upload
node -e "
import('./skill-real.js').then(async (skill) => {
  const result = await skill.executeTool('s3-upload', JSON.stringify({
    bucket: 'test-bucket',
    key: 'test-file.txt',
    file: 'test.txt'
  }));
  console.log(result.output);
});
"
```

**Test S3 List (With Objects)**:
```bash
node -e "
import('./skill-real.js').then(async (skill) => {
  const result = await skill.executeTool('s3-list', JSON.stringify({
    bucket: 'test-bucket'
  }));
  console.log(result.output);
});
"
```

**Test S3 Download**:
```bash
node -e "
import('./skill-real.js').then(async (skill) => {
  const result = await skill.executeTool('s3-download', JSON.stringify({
    bucket: 'test-bucket',
    key: 'test-file.txt',
    output: 'downloaded.txt'
  }));
  console.log(result.output);
});
"

# Verify
cat downloaded.txt
```

**Test EC2 List**:
```bash
node -e "
import('./skill-real.js').then(async (skill) => {
  const result = await skill.executeTool('ec2-list', JSON.stringify({}));
  console.log(result.output);
});
"
```

#### Step 6: Clean Up

```bash
docker-compose down
rm -f test.txt downloaded.txt
```

## Testing with Real AWS

To test against real AWS (not LocalStack):

1. **Remove LocalStack environment variables**:
```bash
unset LOCALSTACK
unset AWS_ENDPOINT_URL
```

2. **Set real AWS credentials**:
```bash
export SKILL_AWS_ACCESS_KEY_ID="your-real-key"
export SKILL_AWS_SECRET_ACCESS_KEY="your-real-secret"
export SKILL_REGION="us-east-1"
```

3. **Run tests** (be careful with production!):
```bash
# Test on a safe test bucket
node -e "
import('./skill-real.js').then(async (skill) => {
  const result = await skill.executeTool('s3-list', JSON.stringify({
    bucket: 'your-test-bucket'
  }));
  console.log(result.output);
});
"
```

## Troubleshooting

### LocalStack Not Starting

**Problem**: `curl: (7) Failed to connect to localhost port 4566`

**Solutions**:
1. Check Docker is running: `docker ps`
2. Check LocalStack logs: `docker-compose logs`
3. Wait longer - it takes 10-30 seconds to start
4. Check port 4566 isn't in use: `lsof -i :4566`

### Dependencies Not Installing

**Problem**: `npm install` fails

**Solutions**:
1. Check Node.js version: `node --version` (requires 18+)
2. Clear npm cache: `npm cache clean --force`
3. Delete node_modules: `rm -rf node_modules && npm install`

### AWS SDK Errors

**Problem**: `Cannot find module '@aws-sdk/client-s3'`

**Solutions**:
1. Install dependencies: `npm install`
2. Check package.json exists
3. Verify node_modules has the SDK: `ls node_modules/@aws-sdk/`

### Module Import Errors

**Problem**: `SyntaxError: Cannot use import statement outside a module`

**Solutions**:
1. Check package.json has `"type": "module"`
2. Use `.js` extension (not `.mjs`)
3. Update Node.js to 18+

### LocalStack Connection Refused

**Problem**: `connect ECONNREFUSED 127.0.0.1:4566`

**Solutions**:
1. Verify LocalStack is running: `docker ps | grep localstack`
2. Check health endpoint: `curl http://localhost:4566/_localstack/health`
3. Restart LocalStack: `docker-compose restart`

### Bucket Not Found

**Problem**: `The specified bucket does not exist`

**Solutions**:
1. Create bucket first: `aws --endpoint-url=http://localhost:4566 s3 mb s3://test-bucket`
2. Verify bucket exists: `aws --endpoint-url=http://localhost:4566 s3 ls`
3. Check bucket name spelling

## Expected Test Results

### S3 List (Empty)
```
📦 S3 Bucket: test-bucket
🌍 Region: us-east-1

No objects found.

✓ Found 0 objects
```

### S3 Upload
```
✓ File uploaded successfully

Source: test.txt
Destination: s3://test-bucket/test-file.txt
Region: us-east-1
Size: 10 B

URL: http://localhost:4566/test-bucket/test-file.txt
```

### S3 List (With Object)
```
📦 S3 Bucket: test-bucket
🌍 Region: us-east-1

Key                                                Size         Last Modified
--------------------------------------------------------------------------------
test-file.txt                                      10 B         12/18/2025

✓ Found 1 objects
```

### EC2 List (Empty)
```
💻 EC2 Instances
🌍 Region: us-east-1

No instances found.

✓ Found 0 instances
```

## Performance Benchmarks

Expected performance with LocalStack:

- **S3 List**: <100ms
- **S3 Upload** (10KB file): <200ms
- **S3 Download** (10KB file): <200ms
- **EC2 List**: <150ms
- **Lambda Invoke**: <300ms

Real AWS will be slower due to network latency (typically +100-500ms).

## Next Steps

After successful testing:

1. Replace `skill.js` with `skill-real.js` for production use
2. Test with real AWS credentials (on test account)
3. Implement additional tools (bucket creation, object deletion, etc.)
4. Add error handling improvements based on real-world usage
5. Optimize for performance (connection pooling, caching)

## Security Notes

**LocalStack Testing**:
- Uses fake credentials (`test`/`test`)
- Data stays local
- Safe for testing sensitive operations

**Real AWS Testing**:
- Use dedicated test account
- Never use production credentials
- Use IAM policies to restrict access
- Monitor AWS billing during tests
- Clean up test resources after

## Contributing

Found a bug? Have a suggestion?

1. Document the issue
2. Create a reproducible test case
3. Submit PR with fix and test
