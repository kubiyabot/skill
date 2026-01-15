# Example Skills

Explore our collection of pre-built skills for common tasks and integrations.

## Featured Skills

### Infrastructure & DevOps

#### [Kubernetes Skill](./kubernetes.md)
Manage Kubernetes clusters with kubectl integration.
```bash
skill run kubernetes get --resource pods --all-namespaces
```
**Tools**: get, apply, delete, logs, exec, port-forward, scale, rollout

#### [Terraform Skill](./terraform.md)
Infrastructure as Code with Terraform.
```bash
skill run terraform plan --dir ./infrastructure
skill run terraform apply --auto-approve
```
**Tools**: init, plan, apply, destroy, show, validate

#### [Docker Skill](./docker.md)
Container management and operations.
```bash
skill run docker ps
skill run docker build --tag myapp:latest
```
**Tools**: ps, build, run, stop, logs, exec, inspect

### Cloud Platforms

#### [AWS Skill](./aws.md)
Amazon Web Services integration.
```bash
skill run aws s3-list --bucket my-bucket
skill run aws ec2-list --region us-east-1
```
**Tools**: s3 operations, EC2 management, Lambda functions, DynamoDB queries

#### [GCP Skill](./gcp.md)
Google Cloud Platform operations.
```bash
skill run gcp compute-list --project my-project
skill run gcp storage-upload --bucket my-bucket --file data.json
```
**Tools**: Compute Engine, Cloud Storage, BigQuery, Cloud Functions

### Development Tools

#### [GitHub Skill](./github.md)
GitHub repository and workflow management.
```bash
skill run github list-repos --org anthropics
skill run github create-issue --repo skill --title "Bug report"
```
**Tools**: repos, issues, PRs, workflows, releases

#### [Git Skill](./git.md)
Local git operations.
```bash
skill run git status
skill run git commit --message "feat: add new feature"
```
**Tools**: status, commit, push, pull, branch, log, diff

### Collaboration

#### [Slack Skill](./slack.md)
Team communication and notifications.
```bash
skill run slack post --channel general --text "Deployment complete"
skill run slack list-channels
```
**Tools**: post message, list channels, get users, file upload

#### [Jira Skill](./jira.md)
Issue tracking and project management.
```bash
skill run jira create-issue --project PROJ --type bug --summary "Fix login"
skill run jira list-issues --jql "assignee = currentUser()"
```
**Tools**: create/update/list issues, transitions, comments

### Databases

#### [PostgreSQL Skill](./postgres.md)
PostgreSQL database operations.
```bash
skill run postgres query --host localhost --query "SELECT * FROM users"
skill run postgres backup --database myapp --output backup.sql
```
**Tools**: query, backup, restore, list tables, execute script

#### [Redis Skill](./redis.md)
Redis cache and data structure operations.
```bash
skill run redis get --key session:user123
skill run redis set --key config:feature --value enabled --ttl 3600
```
**Tools**: get, set, delete, keys, hash operations, pub/sub

### Media Processing (Docker-based)

#### [FFmpeg Skill](./ffmpeg.md)
Video and audio processing.
```bash
skill run ffmpeg -- -i input.mp4 output.webm
skill run ffmpeg -- -i video.mp4 -vn audio.mp3
```
**Features**: Format conversion, compression, thumbnails, GIF creation

#### [ImageMagick Skill](./imagemagick.md)
Image manipulation and conversion.
```bash
skill run imagemagick convert input.png output.jpg
skill run imagemagick resize input.jpg 800x600 output.jpg
```
**Features**: Convert, resize, crop, effects, watermarks

## Skill Catalog

### By Runtime

**WASM Skills** (sandboxed, portable):
- simple-skill - Minimal example
- github-oauth-skill - GitHub OAuth integration
- slack-skill - Slack API integration
- http-skill - HTTP requests
- json-skill - JSON processing

**Native Skills** (CLI wrappers):
- kubernetes - kubectl wrapper
- terraform - Terraform CLI
- docker - Docker CLI
- git - Git operations
- github - GitHub CLI (gh)

**Docker Skills** (containerized tools):
- ffmpeg - Video/audio processing
- imagemagick - Image manipulation
- postgres - PostgreSQL client
- redis - Redis client
- mysql - MySQL client

### By Category

**Infrastructure** (6 skills)
kubernetes, terraform, docker, helm, ansible, vagrant

**Cloud** (8 skills)
aws, gcp, azure, digitalocean, heroku, cloudflare, vercel, netlify

**Development** (10 skills)
git, github, gitlab, bitbucket, npm, cargo, maven, gradle, make, cmake

**Databases** (7 skills)
postgres, mysql, mongodb, redis, elasticsearch, dynamodb, cassandra

**Collaboration** (5 skills)
slack, discord, teams, jira, notion

**Media** (4 skills)
ffmpeg, imagemagick, pandoc, ghostscript

**Monitoring** (4 skills)
prometheus, grafana, datadog, newrelic

**Security** (3 skills)
vault, 1password, bitwarden

## Skill Templates

Start with these templates for common patterns:

### [Simple JavaScript Skill](./templates/javascript.md)
Basic skill with tool definitions.

### [Rust WASM Skill](./templates/rust.md)
High-performance compiled skill.

### [CLI Wrapper Skill](./templates/cli-wrapper.md)
Wrap existing command-line tools.

### [HTTP API Skill](./templates/http-api.md)
Integrate external REST APIs.

## Community Skills

Browse community-contributed skills:

**[Skill Marketplace →](https://marketplace.skill-engine.dev)** (coming soon)

## Contributing Your Skill

Share your skill with the community:

1. Create your skill following the [Development Guide](../guides/developing-skills.md)
2. Add comprehensive documentation (SKILL.md)
3. Submit to the [skill-catalog](https://github.com/kubiyabot/skill-catalog) repository
4. Follow the [contribution guidelines](../guides/contributing.md)

## Skill Quality Checklist

Before publishing:

- [ ] Comprehensive SKILL.md documentation
- [ ] Example usage for each tool
- [ ] Parameter validation
- [ ] Error handling
- [ ] Unit tests
- [ ] Integration tests
- [ ] Security review
- [ ] Performance benchmarks

## Finding More Skills

- **GitHub**: Search for `skill-engine-skill` topic
- **Docker Hub**: Tagged with `skill-engine`
- **npm**: Published with `skill-engine-` prefix
- **Crates.io**: Tagged with `skill-engine`

## Next Steps

- **[Create Your First Skill](../getting-started/first-skill.md)**
- **[Skill Development Guide](../guides/developing-skills.md)**
- **[Publishing Skills](../guides/publishing.md)**
