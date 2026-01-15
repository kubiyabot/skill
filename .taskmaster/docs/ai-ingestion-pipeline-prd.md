# AI-Enhanced Ingestion Pipeline PRD

## Overview

Enhance the skill ingestion pipeline with AI-powered synthetic example generation, self-ask-with-search reasoning, and streaming thought process output. This will improve tool discoverability and provide better execution examples for LLM agents.

## Problem Statement

Currently, the skill ingestion pipeline:
- Relies solely on manually written examples in SKILL.md files
- Has no AI enhancement during indexing
- Cannot generate contextual usage examples from tool schemas
- Provides no streaming feedback during processing

## Goals

1. **Synthetic Example Generation**: Automatically generate high-quality execution examples from tool schemas
2. **Advanced Reasoning**: Implement self-ask-with-search pattern for complex tool discovery queries
3. **Streaming Output**: Real-time SSE streaming of generation/reasoning steps
4. **Offline Support**: Full functionality with local models (Ollama, llama.cpp)
5. **Provider Abstraction**: Use llm-chain's executor abstraction for uniform provider support

## User Choices

Based on requirements gathering:
- **LLM Provider Approach**: llm-chain abstraction for all providers
- **Agent Access Points**: Both MCP + CLI interfaces
- **Streaming Format**: SSE (Server-Sent Events) with JSON payloads

---

## Phase 1: Synthetic Example Generator (P0 - Critical)

### Description

Generate realistic execution examples from tool schemas during skill installation/indexing.

### Requirements

#### 1.1 Schema-to-Example Generation

Input a tool's name, description, and parameter schema. Output realistic, valid execution examples.

```rust
pub struct ExampleGenerator {
    executor: Box<dyn llm_chain::Executor>,
    config: GeneratorConfig,
}

pub struct GeneratedExample {
    /// Full command: skill run k8s:get pods namespace=default
    command: String,
    /// Human explanation: "Lists pods in the default namespace"
    explanation: String,
    /// Model confidence score 0.0-1.0
    confidence: f32,
    /// Schema validation passed
    validated: bool,
}

impl ExampleGenerator {
    pub async fn generate(&self, tool: &ToolDocument) -> Result<Vec<GeneratedExample>>;
    pub fn generate_stream(&self, tool: &ToolDocument) -> impl Stream<Item = GenerationEvent>;
}
```

#### 1.2 Prompt Engineering

Use structured few-shot prompting:

```
You are a CLI tool documentation expert. Generate {count} realistic usage examples.

Tool: {tool_name}
Description: {description}
Parameters:
{parameters_json}

Existing Examples (if any):
{existing_examples}

Generate diverse examples covering:
- Common use cases
- Edge cases with optional parameters
- Real-world scenarios

Output JSON array:
[{"command": "...", "explanation": "..."}]
```

#### 1.3 Example Validation

- Validate generated examples match parameter schema
- Filter invalid/nonsensical examples
- Score diversity using embedding similarity
- Reject duplicates of existing examples

### Acceptance Criteria

- [ ] Generate 3-5 valid examples per tool
- [ ] Examples pass schema validation >95% of time
- [ ] Diverse coverage (not all similar examples)
- [ ] Works with Ollama local models

---

## Phase 2: Self-Ask-With-Search Agent (P1 - High)

### Description

Implement llm-chain's self-ask-with-search pattern for advanced tool discovery queries.

### Requirements

#### 2.1 Agent Pattern Implementation

```rust
pub struct SelfAskSearchAgent {
    executor: Box<dyn llm_chain::Executor>,
    search_pipeline: Arc<SearchPipeline>,
}

pub struct AgentStep {
    pub thought: String,
    pub follow_up_question: Option<String>,
    pub search_results: Option<Vec<ToolDocument>>,
    pub is_final: bool,
    pub final_answer: Option<String>,
}

impl SelfAskSearchAgent {
    pub async fn reason(&self, query: &str) -> Result<AgentResponse>;
    pub fn reason_stream(&self, query: &str) -> impl Stream<Item = AgentStep>;
}
```

#### 2.2 Reasoning Chain

```
User: "How do I deploy a containerized app to Kubernetes?"

Agent Thought: "I need to break this into steps"

Follow-up: "What tools build Docker containers?"
→ Search → docker:build, docker:compose

Follow-up: "What tools deploy to Kubernetes?"
→ Search → kubernetes:apply, kubernetes:deploy

Follow-up: "What tools manage Kubernetes configs?"
→ Search → kubernetes:get, kubernetes:describe

Final Answer: "Use docker:build to containerize, then kubernetes:apply to deploy.
Suggested workflow:
1. docker:build --context=. --tag=myapp:latest
2. kubernetes:apply --file=deployment.yaml --namespace=production"
```

#### 2.3 Search Integration

- Use existing `SearchPipeline` for intermediate searches
- Track full reasoning trace
- Return citations to source tools

### Acceptance Criteria

- [ ] Handles complex multi-step queries
- [ ] Returns reasoning trace with each step
- [ ] Integrates with existing search pipeline
- [ ] Works offline with Ollama

---

## Phase 3: Streaming Output (P0 - Critical)

### Description

Real-time SSE streaming of generation and reasoning steps.

### Requirements

#### 3.1 SSE Event Types

```rust
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum GenerationEvent {
    #[serde(rename = "started")]
    Started { tool_name: String, total_tools: usize },

    #[serde(rename = "thinking")]
    Thinking { thought: String },

    #[serde(rename = "searching")]
    Searching { query: String },

    #[serde(rename = "search_result")]
    SearchResult { tools: Vec<String>, count: usize },

    #[serde(rename = "example")]
    Example { example: GeneratedExample },

    #[serde(rename = "validation")]
    Validation { valid: bool, errors: Vec<String> },

    #[serde(rename = "progress")]
    Progress { current: usize, total: usize, percent: f32 },

    #[serde(rename = "completed")]
    Completed { examples_generated: usize, duration_ms: u64 },

    #[serde(rename = "error")]
    Error { message: String, recoverable: bool },
}
```

#### 3.2 SSE Format

```
event: thinking
data: {"type":"thinking","thought":"Analyzing kubernetes:apply parameters..."}

event: example
data: {"type":"example","example":{"command":"skill run k8s:apply...","explanation":"..."}}

event: progress
data: {"type":"progress","current":5,"total":15,"percent":33.3}
```

#### 3.3 MCP Streaming Integration

```rust
// In MCP server
#[tool(description = "Generate examples with streaming progress")]
async fn generate_examples_stream(
    &self,
    skill: String,
    #[arg(description = "Stream progress events")] stream: Option<bool>,
) -> Result<impl Stream<Item = GenerationEvent>, McpError>;
```

#### 3.4 CLI Streaming Output

```bash
$ skill enhance kubernetes --stream

🤖 AI-Enhanced Indexing: kubernetes
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

→ kubernetes:apply
  💭 Analyzing deployment manifest application...
  ✓ Generated: skill run k8s:apply --file=deploy.yaml
  ✓ Generated: skill run k8s:apply --file=- < manifest.yaml
  ✓ Generated: skill run k8s:apply --kustomize=./overlays/prod
  ✅ 3 examples validated

→ kubernetes:get [2/15]
  💭 Analyzing resource listing parameters...
  ...

Progress: ████████░░░░░░░░ 33% (5/15 tools)
```

### Acceptance Criteria

- [ ] SSE events stream in real-time
- [ ] CLI shows live progress with spinners
- [ ] MCP supports streaming responses
- [ ] Events are well-typed and documented

---

## Phase 4: Enhanced Ingestion Pipeline (P1 - High)

### Description

Integrate AI generation into the skill installation flow.

### Requirements

#### 4.1 Configuration Schema

```toml
# ~/.skill-engine/search.toml

[ai_ingestion]
# Enable AI example generation
enabled = true

# Number of examples per tool
examples_per_tool = 5

# LLM provider (uses llm-chain abstraction)
provider = "ollama"  # ollama | openai | anthropic | local

# Model configuration
model = "llama3.2:8b"

# Validate examples against schema
validate_examples = true

# Stream progress to terminal
stream_progress = true

# Cache generated examples
cache_examples = true

# Timeout per tool (seconds)
timeout_secs = 30

[ai_ingestion.ollama]
host = "http://localhost:11434"

[ai_ingestion.openai]
# Uses OPENAI_API_KEY env var
model = "gpt-4o-mini"

[ai_ingestion.anthropic]
# Uses ANTHROPIC_API_KEY env var
model = "claude-3-haiku-20240307"

[ai_ingestion.local]
# llm-chain local model
model_path = "~/.skill-engine/models/llama-3.2-8b.gguf"
```

#### 4.2 Installation Flow Integration

```rust
// In install.rs
pub async fn install_skill(name: &str, config: &Config) -> Result<()> {
    // 1. Download skill
    let skill = download_skill(name).await?;

    // 2. Parse SKILL.md
    let tools = parse_skill_md(&skill)?;

    // 3. AI Enhancement (if enabled)
    if config.ai_ingestion.enabled {
        let generator = ExampleGenerator::new(&config)?;

        for tool in &mut tools {
            let stream = generator.generate_stream(tool);

            pin_mut!(stream);
            while let Some(event) = stream.next().await {
                display_event(&event)?;

                if let GenerationEvent::Example { example } = event {
                    tool.examples.push(example);
                }
            }
        }
    }

    // 4. Index with enhanced examples
    search_pipeline.index_tools(&tools).await?;
}
```

#### 4.3 Incremental Updates

- Track content hash of tool schemas
- Only regenerate examples when schema changes
- Cache generated examples with checksums
- Respect API rate limits

#### 4.4 CLI Commands

```bash
# Enhance existing skills with AI examples
skill enhance kubernetes
skill enhance --all

# Enhance during install
skill install kubernetes --enhance

# Configure AI ingestion
skill setup --ai-ingestion

# View enhancement status
skill status --enhanced
```

### Acceptance Criteria

- [ ] Seamless integration with install flow
- [ ] Incremental updates work correctly
- [ ] CLI commands are intuitive
- [ ] Configuration is flexible

---

## Phase 5: llm-chain Integration (P2 - Medium)

### Description

Leverage llm-chain library for robust LLM operations.

### Requirements

#### 5.1 Dependencies

```toml
[dependencies]
llm-chain = { version = "0.13", features = ["llama"] }
llm-chain-openai = "0.13"
llm-chain-ollama = "0.13"
```

#### 5.2 Executor Abstraction

```rust
use llm_chain::executor;
use llm_chain::prompt;
use llm_chain::chains::Chain;

pub struct LLMProvider {
    executor: Box<dyn executor::Executor>,
}

impl LLMProvider {
    pub fn ollama(host: &str, model: &str) -> Result<Self>;
    pub fn openai(api_key: &str, model: &str) -> Result<Self>;
    pub fn anthropic(api_key: &str, model: &str) -> Result<Self>;
    pub fn local(model_path: &Path) -> Result<Self>;
}
```

#### 5.3 Prompt Templates

```rust
let example_prompt = prompt!(
    "You are a CLI tool documentation expert.",
    "Generate {{count}} realistic usage examples for this tool:",
    "",
    "Tool Name: {{tool_name}}",
    "Description: {{description}}",
    "Parameters:",
    "{{parameters}}",
    "",
    "Requirements:",
    "- Examples must use valid parameter values",
    "- Cover common and edge cases",
    "- Include brief explanations",
    "",
    "Output as JSON array: [{\"command\": \"...\", \"explanation\": \"...\"}]"
);
```

#### 5.4 Chain Composition

```rust
// Example generation chain
let gen_chain = Chain::new(example_prompt)
    .with_output_parser(JsonArrayParser::new())
    .with_retry(3);

// Validation chain
let validate_chain = Chain::new(validation_prompt)
    .with_output_parser(BoolParser::new());

// Combined pipeline
let pipeline = gen_chain
    .then(validate_chain)
    .with_error_handler(fallback_handler);
```

### Acceptance Criteria

- [ ] llm-chain integrated and working
- [ ] Multiple providers supported uniformly
- [ ] Chains are composable and testable
- [ ] Error handling is robust

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Skill Installation                          │
│                  skill install kubernetes                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SKILL.md Parser                              │
│  - Frontmatter extraction                                       │
│  - Tool section parsing                                         │
│  - Parameter extraction                                         │
│  - Manual example extraction                                    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              AI Example Generator (NEW)                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ LLM Provider    │  │ Prompt Builder  │  │ Validator       │ │
│  │ (llm-chain)     │  │ - Schema→Prompt │  │ - Type check    │ │
│  │ - Ollama        │  │ - Few-shot      │  │ - Diversity     │ │
│  │ - OpenAI        │  │ - Templates     │  │ - Quality score │ │
│  │ - Anthropic     │  └─────────────────┘  └─────────────────┘ │
│  │ - Local         │                                           │
│  └─────────────────┘                                           │
│                              │                                  │
│                    Stream<GenerationEvent> (SSE)                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Enhanced ToolDocument                        │
│  - Original metadata from SKILL.md                              │
│  - AI-generated examples with confidence scores                 │
│  - Validated against schema                                     │
│  - Weighted embedding text                                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SearchPipeline                               │
│  - Vector embeddings (FastEmbed)                                │
│  - BM25 keyword index (Tantivy)                                 │
│  - Hybrid retrieval with RRF                                    │
│  - Self-ask agent for complex queries                           │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│      MCP Server         │     │      CLI                │
│  - search_skills        │     │  - skill find           │
│  - generate_examples    │     │  - skill enhance        │
│  - reason_query         │     │  - skill install        │
│  - SSE streaming        │     │  - Progress spinners    │
└─────────────────────────┘     └─────────────────────────┘
```

---

## Files to Create

| File | Description |
|------|-------------|
| `crates/skill-runtime/src/generation/mod.rs` | Generation module exports |
| `crates/skill-runtime/src/generation/example_generator.rs` | Core example generator |
| `crates/skill-runtime/src/generation/llm_provider.rs` | llm-chain provider wrapper |
| `crates/skill-runtime/src/generation/validator.rs` | Example validation |
| `crates/skill-runtime/src/generation/streaming.rs` | SSE event types and streams |
| `crates/skill-runtime/src/agents/mod.rs` | Agent module exports |
| `crates/skill-runtime/src/agents/self_ask.rs` | Self-ask-with-search agent |
| `crates/skill-cli/src/commands/enhance.rs` | CLI enhance command |

## Files to Modify

| File | Changes |
|------|---------|
| `crates/skill-runtime/src/lib.rs` | Export generation and agents modules |
| `crates/skill-runtime/src/search/pipeline.rs` | Integrate example generator |
| `crates/skill-runtime/src/search_config.rs` | Add ai_ingestion config section |
| `crates/skill-cli/src/commands/install.rs` | Add --enhance flag |
| `crates/skill-cli/src/commands/mod.rs` | Add enhance command |
| `crates/skill-mcp/src/server.rs` | Add generate_examples, reason_query tools |
| `Cargo.toml` | Add llm-chain dependencies |

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Example schema validation pass rate | >95% |
| Example diversity score (unique clusters) | >0.7 |
| Generation latency per tool (Ollama) | <5 seconds |
| Streaming first-event latency | <500ms |
| Offline functionality | 100% with Ollama |
| Search relevance improvement | >20% MRR increase |

---

## Dependencies

### Required Crates

```toml
# LLM
llm-chain = { version = "0.13", features = ["llama"] }
llm-chain-openai = "0.13"
llm-chain-ollama = "0.13"

# Streaming
tokio-stream = "0.1"
async-stream = "0.3"
futures-util = "0.3"

# SSE
axum = { version = "0.7", features = ["sse"] }
```

### External Services

- **Ollama** (recommended): Local LLM inference
- **OpenAI API** (optional): Cloud LLM
- **Anthropic API** (optional): Cloud LLM

---

## Rollout Plan

1. **Phase 1**: Example Generator core (2-3 tasks)
2. **Phase 2**: Streaming infrastructure (2 tasks)
3. **Phase 3**: CLI integration (2 tasks)
4. **Phase 4**: MCP integration (2 tasks)
5. **Phase 5**: Self-ask agent (2-3 tasks)
6. **Phase 6**: llm-chain deep integration (2 tasks)

Total estimated tasks: 12-15

---

## Open Questions

1. Should we support batch generation across multiple skills?
2. Rate limiting strategy for cloud providers?
3. Example quality feedback loop from users?

---

## References

- [llm-chain](https://github.com/sobelio/llm-chain) - Rust LLM library
- [llm-chain self_ask_with_search](https://docs.rs/llm-chain/latest/llm_chain/agents/self_ask_with_search/)
- [OpenAI Synthetic Data Cookbook](https://cookbook.openai.com/examples/sdg1)
- [Anthropic Context Engineering](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)
- [tokio-stream](https://docs.rs/tokio-stream)
