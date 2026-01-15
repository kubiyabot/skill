# Manual Acceptance Tests for Claude Code Integration

This document provides step-by-step manual testing procedures for validating
Claude Code's integration with the Skill Engine across 10+ real-world usage
scenarios.

## Table of Contents

- [Prerequisites](#prerequisites)
- [User Personas](#user-personas)
- [Test Procedures](#test-procedures)
  - [TC1: Kubernetes Pod Investigation](#tc1-kubernetes-pod-investigation)
  - [TC2: Docker Container Debugging](#tc2-docker-container-debugging)
  - [TC3: Git Repository Analysis](#tc3-git-repository-analysis)
  - [TC4: AWS Infrastructure Review](#tc4-aws-infrastructure-review)
  - [TC5: Terraform Plan Review](#tc5-terraform-plan-review)
  - [TC6: Database Query Execution](#tc6-database-query-execution)
  - [TC7: REST API Health Check](#tc7-rest-api-health-check)
  - [TC8: System Log Analysis](#tc8-system-log-analysis)
  - [TC9: Metrics Query](#tc9-metrics-query)
  - [TC10: CI/CD Pipeline Status](#tc10-cicd-pipeline-status)
- [Success Criteria](#success-criteria)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### General Requirements

- [ ] Claude Code CLI installed (`claude --version` works)
- [ ] Skill Engine configured:
  ```bash
  skill claude generate --force
  ```
- [ ] MCP server accessible:
  ```bash
  skill serve
  ```
- [ ] Claude Code can discover skills:
  ```bash
  # In Claude Code session
  /skills list
  ```

### Scenario-Specific Requirements

- **TC1 (Kubernetes)**: kubectl configured with cluster access
- **TC2 (Docker)**: Docker daemon running
- **TC3 (Git)**: Git repository with commit history
- **TC4 (AWS)**: AWS credentials configured
- **TC5 (Terraform)**: Terraform installed with valid project
- **TC6 (Database)**: PostgreSQL server accessible
- **TC7 (API)**: Test API endpoint available
- **TC8 (Logs)**: System log access (journalctl or syslog)
- **TC9 (Metrics)**: Prometheus server running
- **TC10 (CI/CD)**: GitHub repository with Actions, GITHUB_TOKEN set

## User Personas

### Persona 1: DevOps Engineer (Sarah)
- **Experience**: 5 years in infrastructure
- **Goals**: Quick troubleshooting, infrastructure visibility
- **Skills**: Kubernetes, Docker, Terraform
- **Scenarios**: TC1, TC2, TC5

### Persona 2: SRE (Marcus)
- **Experience**: 8 years in reliability engineering
- **Goals**: Monitor systems, investigate incidents
- **Skills**: Metrics, logs, databases
- **Scenarios**: TC6, TC8, TC9

### Persona 3: Full-Stack Developer (Priya)
- **Experience**: 3 years development
- **Goals**: Debug applications, check deployments
- **Skills**: Git, Docker, API testing
- **Scenarios**: TC2, TC3, TC7, TC10

## Test Procedures

---

### TC1: Kubernetes Pod Investigation

**User Persona**: DevOps Engineer (Sarah)

**Scenario**: Sarah needs to check which pods are running in production

**User Prompt**:
```
Show me all running pods in the cluster
```

**Expected Workflow**:

1. **Skill Discovery**
   - [ ] Claude recognizes this is a Kubernetes operation
   - [ ] Claude searches for relevant skills using semantic search
   - [ ] `search_skills("kubernetes pods")` returns kubernetes skill

2. **Tool Selection**
   - [ ] Claude identifies `kubernetes:get` as the appropriate tool
   - [ ] Claude determines parameters: `resource=pods`

3. **Execution**
   - [ ] Claude executes via MCP: `execute(skill="kubernetes", tool="get", args={"resource": "pods"})`
   - [ ] Output shows pod list with NAME, STATUS, AGE columns

4. **Context Engineering**
   - [ ] Claude applies grep filter to show only Running pods
   - [ ] Final output excludes Pending, Failed, Completed pods
   - [ ] Output is concise and relevant

**Verification Checklist**:
- [ ] Correct skill discovered
- [ ] Appropriate tool selected
- [ ] Tool executed successfully
- [ ] Context engineering applied correctly
- [ ] Output is accurate and useful
- [ ] No errors or warnings shown to user

**Expected Result**:
```
Here are the running pods in your cluster:

NAME                           READY   STATUS    RESTARTS   AGE
nginx-deployment-66b6c48dd5-   1/1     Running   0          2d
api-server-7d9c8f5b4d-x8k9p    1/1     Running   0          1d
...
```

---

### TC2: Docker Container Debugging

**User Persona**: Full-Stack Developer (Priya)

**Scenario**: Priya needs to check application logs for errors

**User Prompt**:
```
Show me the last 50 lines of logs from the nginx container
```

**Expected Workflow**:

1. **Skill Discovery**
   - [ ] Claude identifies docker operation
   - [ ] `search_skills("docker container logs")` returns docker skill

2. **Tool Selection & Parameters**
   - [ ] Claude selects `docker:logs` tool
   - [ ] Claude extracts parameters: `container=nginx`
   - [ ] Claude applies context engineering: `head=50`

3. **Execution**
   - [ ] `execute(skill="docker", tool="logs", args={"container": "nginx"}, head=50)`
   - [ ] Output shows last 50 log lines

**Verification Checklist**:
- [ ] docker skill discovered
- [ ] Correct tool selected
- [ ] Container name extracted correctly
- [ ] Line limit applied (50 lines max)
- [ ] Logs displayed in chronological order

**Expected Result**:
```
Showing the last 50 lines from nginx container logs:

2024-01-04 10:00:01 [info] Server started
2024-01-04 10:00:02 [info] Listening on port 80
...
(exactly 50 lines or fewer if container has less)
```

---

### TC3: Git Repository Analysis

**User Persona**: Full-Stack Developer (Priya)

**Scenario**: Priya wants to understand recent code changes

**User Prompt**:
```
What changed in the last 5 commits?
```

**Expected Workflow**:

1. **Skill Discovery**
   - [ ] Claude recognizes git operation
   - [ ] `search_skills("git commit history")` returns git skill

2. **Tool Selection**
   - [ ] Claude selects `git:log` tool
   - [ ] Claude determines parameter: `n=5`

3. **Execution & Summarization**
   - [ ] `execute(skill="git", tool="log", args={"n": "5"})`
   - [ ] Claude summarizes commits into readable format
   - [ ] Shows commit hashes, authors, dates, messages

**Verification Checklist**:
- [ ] git skill discovered
- [ ] Correct number of commits shown (5)
- [ ] Commit information is complete
- [ ] Summary is helpful and readable
- [ ] Chronological order (newest first)

**Expected Result**:
```
Here are the last 5 commits:

1. 7a4b2a2 - Add Claude Bridge (2024-01-04)
   - Added Claude Bridge for generating skills

2. 104d7f8 - Add Claude Bridge (2024-01-03)
   - Integration tests added

...
```

---

### TC4: AWS Infrastructure Review

**User Persona**: DevOps Engineer (Sarah)

**Scenario**: Sarah needs to audit S3 buckets

**User Prompt**:
```
List all S3 buckets in my AWS account
```

**Expected Workflow**:

1. **Skill Discovery**
   - [ ] Claude identifies AWS S3 operation
   - [ ] `search_skills("aws s3 buckets")` returns aws skill

2. **Tool Selection**
   - [ ] Claude selects `aws:s3_list_buckets` tool
   - [ ] No additional parameters needed

3. **Execution**
   - [ ] `execute(skill="aws", tool="s3_list_buckets")`
   - [ ] Output shows bucket names, creation dates

**Verification Checklist**:
- [ ] aws skill discovered
- [ ] AWS credentials used correctly
- [ ] Bucket list is accurate
- [ ] Error handling if credentials missing
- [ ] Proper region handling

**Expected Result**:
```
Here are your S3 buckets:

- my-app-uploads (created 2023-06-15)
- logs-bucket (created 2023-09-20)
- terraform-state (created 2024-01-01)
...
```

---

### TC5: Terraform Plan Review

**User Persona**: DevOps Engineer (Sarah)

**Scenario**: Sarah wants to review infrastructure changes before applying

**User Prompt**:
```
Run terraform plan and show me what will change
```

**Expected Workflow**:

1. **Skill Discovery**
   - [ ] Claude identifies Terraform operation
   - [ ] `search_skills("terraform plan")` returns terraform skill

2. **Tool Execution**
   - [ ] Claude selects `terraform:plan` tool
   - [ ] Executes in current directory or specified path

3. **Result Presentation**
   - [ ] Shows planned additions, changes, deletions
   - [ ] Highlights important changes
   - [ ] Summarizes resource changes

**Verification Checklist**:
- [ ] terraform skill discovered
- [ ] Plan executes successfully
- [ ] Output shows resource changes clearly
- [ ] No unintended changes flagged
- [ ] Summary is accurate

**Expected Result**:
```
Terraform Plan Results:

Plan: 2 to add, 1 to change, 0 to destroy

Changes:
+ aws_s3_bucket.new_bucket (will be created)
~ aws_instance.web (will be modified in-place)
  ~ instance_type: "t2.micro" -> "t2.small"
```

---

### TC6: Database Query Execution

**User Persona**: SRE (Marcus)

**Scenario**: Marcus needs to check user count for capacity planning

**User Prompt**:
```
Query the database to count how many users we have
```

**Expected Workflow**:

1. **Skill Discovery**
   - [ ] Claude identifies database query operation
   - [ ] `search_skills("postgres database query")` returns postgres skill

2. **Query Construction**
   - [ ] Claude constructs SQL: `SELECT COUNT(*) FROM users`
   - [ ] Uses postgres:query tool

3. **Execution & Formatting**
   - [ ] Query executes against configured database
   - [ ] Result formatted in readable way

**Verification Checklist**:
- [ ] postgres skill discovered
- [ ] SQL query is correct
- [ ] Database connection established
- [ ] Result is accurate
- [ ] Sensitive data protected

**Expected Result**:
```
User count from database:

Total users: 15,234
```

---

### TC7: REST API Health Check

**User Persona**: SRE (Marcus)

**Scenario**: Marcus needs to verify API availability

**User Prompt**:
```
Check if the production API is healthy
```

**Expected Workflow**:

1. **Skill Discovery**
   - [ ] Claude identifies HTTP request operation
   - [ ] `search_skills("http api health")` returns http skill

2. **Request Construction**
   - [ ] Claude determines health endpoint URL
   - [ ] Uses http:get tool

3. **Execution & Interpretation**
   - [ ] GET request to health endpoint
   - [ ] Status code interpreted (200 = healthy)
   - [ ] Response time noted

**Verification Checklist**:
- [ ] http skill discovered
- [ ] Correct endpoint used
- [ ] HTTP method appropriate (GET)
- [ ] Response interpreted correctly
- [ ] Clear health status reported

**Expected Result**:
```
API Health Check:

Status: ✓ Healthy (200 OK)
Response time: 145ms
All systems operational
```

---

### TC8: System Log Analysis

**User Persona**: SRE (Marcus)

**Scenario**: Marcus needs to investigate recent errors

**User Prompt**:
```
Show me errors from the system logs in the last hour
```

**Expected Workflow**:

1. **Skill Discovery**
   - [ ] Claude identifies log analysis operation
   - [ ] Finds journalctl or syslog skill

2. **Query Construction**
   - [ ] Determines time range: last 1 hour
   - [ ] Applies grep filter: "error"

3. **Context Engineering**
   - [ ] Limits output to prevent overwhelm
   - [ ] Highlights critical errors
   - [ ] Groups similar errors

**Verification Checklist**:
- [ ] System log skill discovered
- [ ] Time range correct (last hour)
- [ ] Grep filter applied ("error")
- [ ] Output is manageable size
- [ ] Critical errors highlighted

**Expected Result**:
```
System errors from the last hour:

10:15:03 - ERROR: Failed to connect to database (3 occurrences)
10:22:41 - ERROR: Disk space low on /var
10:45:12 - ERROR: Authentication failed for user 'admin'

Total: 12 error entries
```

---

### TC9: Metrics Query

**User Persona**: SRE (Marcus)

**Scenario**: Marcus needs to check current CPU usage

**User Prompt**:
```
Query Prometheus for current CPU usage across all nodes
```

**Expected Workflow**:

1. **Skill Discovery**
   - [ ] Claude identifies metrics query
   - [ ] `search_skills("prometheus metrics")` returns prometheus skill

2. **Query Construction**
   - [ ] Claude constructs PromQL query
   - [ ] Example: `100 - (avg(irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)`

3. **Execution & Visualization**
   - [ ] Query executes against Prometheus
   - [ ] Results formatted in readable way
   - [ ] Per-node breakdown if available

**Verification Checklist**:
- [ ] prometheus skill discovered
- [ ] PromQL query is valid
- [ ] Prometheus connection established
- [ ] Results are current
- [ ] Data formatted clearly

**Expected Result**:
```
CPU Usage Metrics:

node-1: 23.4%
node-2: 45.7%
node-3: 12.1%

Average: 27.1%
```

---

### TC10: CI/CD Pipeline Status

**User Persona**: Full-Stack Developer (Priya)

**Scenario**: Priya wants to check if her deployment succeeded

**User Prompt**:
```
Show me the status of our GitHub Actions workflows
```

**Expected Workflow**:

1. **Skill Discovery**
   - [ ] Claude identifies GitHub Actions query
   - [ ] `search_skills("github actions workflows")` returns github skill

2. **API Request**
   - [ ] Uses github:workflows_list tool
   - [ ] Authenticates with GITHUB_TOKEN

3. **Status Presentation**
   - [ ] Shows recent workflow runs
   - [ ] Indicates success/failure
   - [ ] Links to workflow details

**Verification Checklist**:
- [ ] github skill discovered
- [ ] GitHub authentication works
- [ ] Workflow statuses accurate
- [ ] Recent runs shown first
- [ ] Failure reasons visible

**Expected Result**:
```
GitHub Actions Workflow Status:

✓ Deploy to Production - Success (2m ago)
✓ Run Tests - Success (15m ago)
✗ Build Docker Image - Failed (1h ago)
  Error: Docker build timeout

Latest deployment: Successful
```

---

## Success Criteria

### Individual Test Success

Each test case is considered successful if:

1. **Skill Discovery**: Correct skill found in top 3 results
2. **Tool Selection**: Appropriate tool selected automatically
3. **Parameter Extraction**: User intent correctly parsed into tool parameters
4. **Execution**: Tool executes without errors
5. **Context Engineering**: Filters applied appropriately
6. **Output Quality**: Result is helpful and answers user's question
7. **User Experience**: Natural conversation flow, no technical jargon

### Overall Success Targets

- **Automated Tests**: 95%+ pass rate
- **Manual Tests**: 90%+ satisfaction across 3+ user personas
- **Performance**: Tool execution < 5s for most operations
- **Accuracy**: Context engineering applies correctly 100% of the time
- **Error Handling**: Graceful failures with helpful messages

## Troubleshooting

### Skill Not Discovered

**Symptom**: Claude says "I don't have a tool for that"

**Solutions**:
1. Verify skill is installed: `skill list`
2. Check skill manifest: `cat ~/.skill-engine.toml`
3. Regenerate Claude skills: `skill claude generate --force`
4. Verify MCP server running: `skill serve`

### Tool Execution Fails

**Symptom**: "Error executing tool"

**Solutions**:
1. Check tool permissions (kubectl access, AWS credentials, etc.)
2. Verify external service is running (Docker daemon, Kubernetes cluster)
3. Test tool directly: `skill run <skill> <tool> --arg value`
4. Check MCP server logs for detailed error

### Context Engineering Not Applied

**Symptom**: Output contains irrelevant information

**Solutions**:
1. Verify MCP server supports context engineering
2. Check server version: `skill --version` (requires >= 1.0.0)
3. Test context engineering directly via MCP tools/call
4. Review skill TOOLS.md for supported parameters

### Performance Issues

**Symptom**: Slow tool execution (> 10s)

**Solutions**:
1. Check external service health (cluster, database, API)
2. Apply stricter context engineering (max_output, head/tail)
3. Use more specific queries to reduce search time
4. Check network latency to external services

### Authentication Failures

**Symptom**: "Unauthorized" or "Permission denied"

**Solutions**:
1. Verify credentials configured: AWS_PROFILE, GITHUB_TOKEN, etc.
2. Check credential permissions match required actions
3. Refresh expired credentials
4. Test credentials directly: `aws sts get-caller-identity`, `gh auth status`

## Feedback Collection

After each manual test session, collect feedback:

1. **Ease of Use** (1-5): How natural was the interaction?
2. **Accuracy** (1-5): Did Claude understand your intent?
3. **Output Quality** (1-5): Was the result helpful?
4. **Performance** (1-5): Was the response time acceptable?
5. **Overall Satisfaction** (1-5): Would you use this regularly?

**Target**: Average score >= 4.0 across all dimensions

## Reporting Results

Document results in this format:

```markdown
## Test Session Report

**Date**: 2024-01-04
**Tester**: Sarah (DevOps Engineer persona)
**Duration**: 45 minutes

### Tests Executed

- [x] TC1: Kubernetes Pod Investigation - PASS
- [x] TC2: Docker Container Debugging - PASS
- [x] TC5: Terraform Plan Review - FAIL (see notes)

### Issues Found

1. **TC5 Failure**: Terraform plan didn't execute
   - **Root cause**: Terraform not in PATH
   - **Fix**: Added terraform to PATH
   - **Retest**: PASS

### Feedback Scores

- Ease of Use: 5/5
- Accuracy: 4/5
- Output Quality: 5/5
- Performance: 4/5
- Overall: 4.5/5

### Comments

"Very intuitive! The grep filtering on pods was especially helpful.
Would like to see better error messages when tools are misconfigured."
```

---

**Document Version**: 1.0
**Last Updated**: 2024-01-04
**Maintainer**: Claude Bridge Testing Team
