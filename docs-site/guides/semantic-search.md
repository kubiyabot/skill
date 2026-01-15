# Semantic Search

Skill Engine includes AI-powered semantic search to help you discover the right tool for your task using natural language queries.

## Quick Start

```bash
# Find tools using natural language
skill find "list kubernetes pods"

# Get top 10 results
skill find "convert video to mp4" --top-k 10

# JSON output for scripting
skill find "backup database" --format json
```

## How It Works

1. **Vectorization**: Tool descriptions are converted to embeddings (vectors)
2. **Query Embedding**: Your search query is converted to the same vector space
3. **Semantic Matching**: Tools are ranked by semantic similarity
4. **Results**: Top matching tools with relevance scores

**Key benefit**: Find tools even if you don't know the exact name or skill.

## Basic Usage

### Natural Language Queries

```bash
# Instead of remembering exact tool names...
skill run kubernetes get --resource pods

# Just describe what you want to do
skill find "show running containers in kubernetes"
```

Output:

```
🔍 Top 5 results for: "show running containers in kubernetes"

1. kubernetes / get (score: 0.92)
   Get Kubernetes resources like pods, services, deployments

   Example:
   skill run kubernetes get --resource pods --namespace default

2. kubernetes / describe (score: 0.78)
   Describe detailed information about a Kubernetes resource

   Example:
   skill run kubernetes describe --resource pod --name nginx-xxx

3. docker / ps (score: 0.65)
   List running Docker containers

   Example:
   skill run docker ps --all false
```

### Run Directly from Search

```bash
# Find and show command to run
skill find "list s3 buckets"

# Output suggests:
# skill run aws s3-list --region us-east-1

# Copy and run
skill run aws s3-list --region us-east-1
```

## Embedding Providers

Skill Engine supports multiple embedding providers:

### FastEmbed (Default)

**Local, no API key needed:**

```bash
# Uses FastEmbed by default
skill find "create github issue"
```

**Pros**:
- No API key required
- Runs locally (private)
- Fast
- Free

**Cons**:
- Slightly less accurate than cloud providers
- Requires ~100MB model download (first run only)

### OpenAI

**High quality, requires API key:**

```bash
# Use OpenAI embeddings
skill find "deploy to kubernetes" --provider openai

# Custom model
skill find "query database" \
  --provider openai \
  --model text-embedding-3-large
```

**Setup:**

```bash
export OPENAI_API_KEY=sk-...
```

**Pros**:
- Highest quality embeddings
- Best semantic understanding

**Cons**:
- Requires API key and internet
- Costs money (tiny per query)
- Data sent to OpenAI

### Ollama

**Local models with good quality:**

```bash
# Use Ollama (must be running)
skill find "backup postgres" --provider ollama

# Specific model
skill find "list files" \
  --provider ollama \
  --model nomic-embed-text
```

**Setup:**

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull embedding model
ollama pull nomic-embed-text

# Start Ollama server
ollama serve
```

**Pros**:
- Local and private
- Good quality
- Free

**Cons**:
- Requires Ollama installation
- Slower than FastEmbed
- Larger disk usage

## Configuration

### Setup Search

First-time setup:

```bash
skill setup
```

This will:
1. Choose embedding provider
2. Download/configure model
3. Build vector index for all skills

### Re-index Skills

After adding new skills:

```bash
skill setup --rebuild
```

### Check Search Status

```bash
skill search --status
```

Output:

```
Search Configuration:
  Provider: fastembed
  Model: BAAI/bge-small-en-v1.5
  Indexed skills: 15
  Indexed tools: 127
  Index size: 2.4 MB
  Last updated: 2024-01-18 14:32:00
```

## Advanced Usage

### Search Options

```bash
# More results
skill find "deploy application" --top-k 20

# JSON output for scripting
skill find "list pods" --format json

# Compact output (one line per result)
skill find "create backup" --format compact
```

### Filter by Skill

```bash
# Search only in kubernetes skill
skill find "get resources" --skill kubernetes

# Search in multiple skills
skill find "deploy" --skill kubernetes,docker
```

### Threshold Filtering

```bash
# Only show results above 0.8 similarity
skill find "list items" --min-score 0.8
```

## Integration with Claude Code

Semantic search works automatically with Claude Code via MCP:

```
You: "I need to list running pods in the production cluster"

Claude: Let me search for the right tool...
[Uses semantic search internally]

I found the kubernetes 'get' tool. Let me list the pods for you.

[Executes: skill run kubernetes@prod get --resource pods]
```

## Output Formats

### Rich (Default)

Human-readable with colors and formatting:

```bash
skill find "convert video"
```

### JSON

Machine-readable for scripting:

```bash
skill find "backup database" --format json
```

Output:

```json
{
  "query": "backup database",
  "results": [
    {
      "skill": "postgres",
      "tool": "backup",
      "score": 0.95,
      "description": "Create database backup using pg_dump",
      "example": "skill run postgres backup --database mydb --output backup.sql"
    }
  ]
}
```

### Compact

One line per result:

```bash
skill find "list files" --format compact
```

Output:

```
kubernetes/get (0.92): Get Kubernetes resources
docker/ps (0.78): List Docker containers
```

## Search Quality Tips

### 1. Be Specific

```bash
# Vague
skill find "do something with kubernetes"

# Better
skill find "list kubernetes pods in default namespace"
```

### 2. Use Action Verbs

```bash
# Good verbs: list, create, delete, deploy, backup, restore, convert
skill find "backup postgres database"
skill find "convert mp4 to webm"
skill find "deploy to production"
```

### 3. Include Context

```bash
# Without context
skill find "get logs"

# With context
skill find "get kubernetes pod logs from production cluster"
```

### 4. Use Domain Terms

```bash
# Generic
skill find "container things"

# Domain-specific
skill find "kubernetes pod deployment status"
```

## Performance

### Indexing Time

- **Initial index**: ~1-2 seconds for 100 tools
- **Re-index**: Only indexes new/changed skills
- **Automatic**: Runs in background after skill installation

### Search Time

- **FastEmbed**: ~50ms per query
- **OpenAI**: ~200ms per query (network latency)
- **Ollama**: ~100ms per query

### Index Size

- **Small project** (10 skills, 50 tools): ~500 KB
- **Medium project** (50 skills, 300 tools): ~2.5 MB
- **Large project** (200 skills, 1500 tools): ~12 MB

## Troubleshooting

### Search Not Working

```bash
# Check if index exists
skill search --status

# Rebuild index
skill setup --rebuild
```

### Ollama Connection Failed

```bash
# Check if Ollama is running
curl http://localhost:11434/api/version

# Start Ollama
ollama serve
```

### OpenAI API Error

```bash
# Check API key
echo $OPENAI_API_KEY

# Set API key
export OPENAI_API_KEY=sk-...
```

### Poor Search Results

```bash
# Try different provider
skill find "your query" --provider openai

# Increase result count
skill find "your query" --top-k 20

# Check indexed tools
skill search --status
```

### Index Out of Date

```bash
# Rebuild after adding new skills
skill setup --rebuild

# Auto-rebuild on skill changes (future feature)
skill config set auto_reindex true
```

## API Usage

### Via REST API

```bash
curl -X POST http://localhost:3000/api/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "list kubernetes pods",
    "top_k": 5
  }'
```

### Via MCP

Semantic search is available through MCP protocol:

```json
{
  "method": "tools/call",
  "params": {
    "name": "skill-engine/search",
    "arguments": {
      "query": "backup database",
      "top_k": 5
    }
  }
}
```

## Examples

### Finding Deployment Tools

```bash
$ skill find "deploy application to kubernetes"

1. kubernetes/apply (0.94)
   Deploy resources using kubectl apply
   skill run kubernetes apply --file deployment.yaml

2. helm/install (0.87)
   Install Helm chart
   skill run helm install --chart ./mychart --name myapp

3. kubectl/create (0.79)
   Create Kubernetes resource
   skill run kubectl create --resource deployment --name app
```

### Database Operations

```bash
$ skill find "backup mysql database"

1. mysql/backup (0.96)
   Create MySQL database backup using mysqldump
   skill run mysql backup --database prod --output backup.sql

2. postgres/backup (0.81)
   Create PostgreSQL backup (shown for similarity)
   skill run postgres backup --database prod --output backup.sql
```

### Video Processing

```bash
$ skill find "convert video format"

1. ffmpeg/convert (0.93)
   Convert video between formats
   skill run ffmpeg convert --input video.mp4 --output video.webm

2. ffmpeg/transcode (0.89)
   Transcode video with quality settings
   skill run ffmpeg transcode --input input.mp4 --output output.mp4 --quality high
```

## Best Practices

1. **Keep descriptions clear** - Tools with good descriptions rank better
2. **Use semantic search for discovery** - Find tools you didn't know existed
3. **Rebuild index after changes** - Run `skill setup --rebuild` after adding skills
4. **Choose provider based on needs**:
   - **Local development**: FastEmbed (default)
   - **Best quality**: OpenAI
   - **Privacy + quality**: Ollama

## Next Steps

- **[Web Interface](./web-interface.md)** - Visual search interface
- **[Skill Development](./developing-skills.md)** - Optimize tool descriptions for search
- **[RAG Search Pipeline](./advanced/rag-search.md)** - Technical deep dive
- **[MCP Protocol](./mcp.md)** - Programmatic access to search
