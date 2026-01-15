# PRD: Enhanced RAG System for Skill Discovery and Agent Guidance

## Executive Summary

This PRD outlines a comprehensive plan to significantly improve the Retrieval-Augmented Generation (RAG) quality and vector search capabilities for the skill-engine. The goal is to provide agents with precise, context-aware skill discovery while minimizing context window overhead and enabling seamless integration with optional remote vector databases and embedding models.

## Problem Statement

### Current Limitations

1. **In-Memory Only Indexing**: All tool indexing happens in-memory per search operation, causing redundant embedding computation and inability to scale
2. **No Persistent Vector Store**: No support for remote/persistent vector databases - all embeddings are ephemeral
3. **Limited Embedding Options**: Only FastEmbed (local), OpenAI, and Ollama supported
4. **No Reranking**: Single-stage retrieval without cross-encoder reranking for precision
5. **No Hybrid Search**: Pure semantic search without keyword/BM25 fallback
6. **Context Window Overload**: Full skill documentation returned without smart chunking or summarization
7. **No Query Understanding**: No query expansion, intent classification, or entity extraction
8. **Limited Caching Strategy**: Basic disk cache for embeddings, but no intelligent cache invalidation or warm-up
9. **No Skill Execution Guidance**: Search returns tool metadata but lacks execution interface documentation

### Impact

- Agents receive too much context, wasting tokens and reducing response quality
- Irrelevant skills returned for ambiguous queries
- Cold search latency of 500ms-2s due to on-demand embedding
- No ability to integrate with enterprise vector infrastructure

## Goals

### Primary Goals

1. **Reduce context window usage by 70%** through smart chunking, summarization, and relevance scoring
2. **Improve search precision (NDCG@5) by 50%** through hybrid search + reranking
3. **Enable sub-100ms search latency** with persistent indexing and caching
4. **Provide pluggable vector database backends** (Qdrant, Pinecone, Weaviate, pgvector)
5. **Deliver execution-ready skill interfaces** with minimal context overhead

### Secondary Goals

1. Support multiple embedding providers with hot-swapping
2. Enable offline-first with graceful degradation
3. Provide skill usage analytics for ranking optimization
4. Support multi-tenant/workspace isolation

## Technical Architecture

### Phase 1: Core RAG Infrastructure (Foundation)

#### 1.1 Persistent Vector Store Abstraction

Create a `VectorStore` trait to abstract storage backends:

```rust
#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn upsert(&self, documents: Vec<EmbeddedDocument>) -> Result<UpsertStats>;
    async fn search(&self, query_embedding: Vec<f32>, filter: Option<Filter>, top_k: usize) -> Result<Vec<SearchResult>>;
    async fn delete(&self, ids: Vec<String>) -> Result<DeleteStats>;
    async fn get(&self, ids: Vec<String>) -> Result<Vec<EmbeddedDocument>>;
    async fn count(&self, filter: Option<Filter>) -> Result<usize>;
    async fn health_check(&self) -> Result<HealthStatus>;
}
```

**Backends to implement:**
- `InMemoryVectorStore` - Current behavior, for development/testing
- `QdrantVectorStore` - Via qdrant-client crate (local or cloud)
- `PineconeVectorStore` - Via pinecone-sdk crate (cloud)
- `WeaviateVectorStore` - Via weaviate-community crate
- `PgVectorStore` - Via sqlx with pgvector extension
- `LanceDBVectorStore` - Via lancedb crate (local, columnar)

#### 1.2 Embedding Provider Abstraction

Enhance the existing embedding setup with a unified trait:

```rust
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed_documents(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>>;
    async fn embed_query(&self, text: &str) -> Result<Vec<f32>>;
    fn dimensions(&self) -> usize;
    fn model_name(&self) -> &str;
}
```

**Providers to support:**
- `FastEmbedProvider` - Local ONNX models (current: AllMiniLM, BGE variants)
- `OpenAIEmbedProvider` - text-embedding-3-small, text-embedding-3-large, ada-002
- `OllamaProvider` - nomic-embed-text, mxbai-embed-large
- `VoyageAIProvider` - voyage-3, voyage-code-3 (best for code)
- `CohereProvider` - embed-v4 with input type support
- `HuggingFaceProvider` - TEI server for custom models
- `AnthropicVoyageProvider` - When Claude embeddings become available

#### 1.3 Document Schema Enhancement

Extend `ToolDocument` with richer metadata for filtering:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDocument {
    // Identity
    pub id: String,                          // skill@instance/tool
    pub skill_name: String,
    pub instance_name: String,
    pub tool_name: String,

    // Content (embedding targets)
    pub description: String,
    pub full_documentation: String,
    pub parameters_text: String,
    pub examples_text: String,

    // Structured metadata (for filtering)
    pub category: Option<String>,            // From SKILL.md frontmatter
    pub tags: Vec<String>,
    pub allowed_tools: Vec<String>,          // What tools this skill can use
    pub parameter_names: Vec<String>,        // For keyword matching
    pub action_verbs: Vec<String>,           // Extracted action keywords

    // Execution interface (compact)
    pub execution_signature: ExecutionSignature,

    // Analytics (for ranking)
    pub usage_count: u64,
    pub success_rate: f32,
    pub avg_latency_ms: u32,
    pub last_used: Option<DateTime<Utc>>,

    // Versioning
    pub skill_version: String,
    pub indexed_at: DateTime<Utc>,
    pub content_hash: String,               // For cache invalidation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSignature {
    pub tool_name: String,
    pub parameters: Vec<ParameterSignature>,
    pub returns: String,
    pub streaming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSignature {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,          // One-line summary
    pub default: Option<String>,
}
```

### Phase 2: Hybrid Search Pipeline

#### 2.1 Three-Stage Retrieval Architecture

Implement a configurable multi-stage retrieval pipeline:

```
┌─────────────────────────────────────────────────────────────────┐
│                        Query Processing                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  Query   │→ │  Intent  │→ │  Query   │→ │  Entity  │        │
│  │  Parse   │  │ Classify │  │ Expand   │  │ Extract  │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    Stage 1: Candidate Retrieval                  │
│                        (Recall-focused, 100-500 docs)           │
│  ┌──────────────────┐   ┌──────────────────┐                   │
│  │  Dense Retrieval │   │ Sparse Retrieval │                   │
│  │  (Bi-Encoder)    │   │ (BM25/SPLADE)    │                   │
│  └────────┬─────────┘   └────────┬─────────┘                   │
│           └──────────┬───────────┘                              │
│                      ↓                                          │
│              ┌──────────────┐                                   │
│              │ RRF Fusion   │                                   │
│              └──────────────┘                                   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    Stage 2: Reranking                            │
│                    (Precision-focused, top 50)                   │
│  ┌──────────────────────────────────────────────────────┐      │
│  │              Cross-Encoder Reranker                  │      │
│  │  (ms-marco-MiniLM-L-12-v2 or Cohere Rerank)         │      │
│  └──────────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    Stage 3: Result Processing                    │
│                    (Context optimization, top 5)                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ Context  │→ │ Response │→ │ Metadata │→ │  Cache   │        │
│  │ Compress │  │ Format   │  │ Enrich   │  │ Results  │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

#### 2.2 BM25/Sparse Retrieval Integration

Add sparse retrieval alongside dense:

```rust
pub struct HybridRetriever {
    dense_store: Arc<dyn VectorStore>,
    sparse_index: Arc<BM25Index>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    fusion_method: FusionMethod,
    dense_weight: f32,  // Default 0.7
    sparse_weight: f32, // Default 0.3
}

pub enum FusionMethod {
    ReciprocalRankFusion { k: u32 },  // RRF with constant k (default 60)
    WeightedSum,
    CombMNZ,
}
```

**BM25 Implementation Options:**
- `tantivy` crate for full-text search with BM25
- Custom BM25 implementation for lightweight needs
- SPLADE sparse embeddings via fastembed

#### 2.3 Cross-Encoder Reranking

Implement reranking layer for precision:

```rust
#[async_trait]
pub trait Reranker: Send + Sync {
    async fn rerank(&self, query: &str, documents: Vec<SearchResult>, top_k: usize) -> Result<Vec<RankedResult>>;
}

pub struct CrossEncoderReranker {
    model: CrossEncoderModel,  // Local ONNX or API
}

pub struct CohereReranker {
    client: CohereClient,
    model: String,  // rerank-v3.5
}

pub struct VoyageReranker {
    client: VoyageClient,
    model: String,  // rerank-2
}
```

**Local Cross-Encoder Models (via fastembed-rs):**
- ms-marco-MiniLM-L-6-v2 (fastest)
- ms-marco-MiniLM-L-12-v2 (balanced)
- BAAI/bge-reranker-base
- BAAI/bge-reranker-large (most accurate)

### Phase 3: Context Optimization for Agents

#### 3.1 Smart Context Compression

Reduce context window usage with intelligent summarization:

```rust
pub struct ContextCompressor {
    max_tokens_per_tool: usize,      // Default 200
    max_total_tokens: usize,         // Default 800
    include_examples: bool,          // Default false for search
    include_full_params: bool,       // Default false, only required params
}

pub struct CompressedToolContext {
    pub tool_id: String,
    pub summary: String,              // 1-2 sentences
    pub execution_hint: String,       // How to call it
    pub required_params: Vec<ParamHint>,
    pub relevance_score: f32,
}

pub struct ParamHint {
    pub name: String,
    pub param_type: String,
    pub hint: String,  // One-line description
}
```

**Compression Strategies:**
1. **Extractive**: Key sentence extraction from documentation
2. **Abstractive**: LLM-generated summaries (cached)
3. **Template-based**: Structured format with placeholders
4. **Progressive**: More detail for higher-ranked results

#### 3.2 Execution-Ready Interface Format

Design output format optimized for agent consumption:

```json
{
  "query": "create a kubernetes deployment",
  "results": [
    {
      "relevance": 0.94,
      "tool": {
        "id": "kubernetes@production/create_deployment",
        "name": "create_deployment",
        "skill": "kubernetes",
        "instance": "production"
      },
      "summary": "Creates a Kubernetes deployment with specified replicas, image, and resource limits.",
      "execution": {
        "signature": "create_deployment(name: str, image: str, replicas: int = 1, namespace: str = 'default')",
        "required_params": ["name", "image"],
        "example": "create_deployment(name='nginx', image='nginx:latest', replicas=3)"
      },
      "context_tokens": 85
    }
  ],
  "total_context_tokens": 340,
  "search_latency_ms": 45
}
```

#### 3.3 Query Understanding Layer

Implement intelligent query preprocessing:

```rust
pub struct QueryProcessor {
    intent_classifier: Option<IntentClassifier>,
    entity_extractor: Option<EntityExtractor>,
    query_expander: Option<QueryExpander>,
}

pub enum QueryIntent {
    ToolDiscovery,      // "what tools can do X"
    ToolExecution,      // "run/execute/call X"
    ToolDocumentation,  // "how does X work"
    Comparison,         // "difference between X and Y"
    Troubleshooting,    // "why is X failing"
}

pub struct ProcessedQuery {
    pub original: String,
    pub expanded: Vec<String>,      // Synonym expansion
    pub intent: QueryIntent,
    pub entities: Vec<Entity>,      // Extracted skill/tool names
    pub filters: Vec<Filter>,       // Inferred metadata filters
}
```

**Entity Types to Extract:**
- Skill names (fuzzy matching against known skills)
- Tool names
- Categories (database, kubernetes, git, etc.)
- Actions (create, delete, list, etc.)
- Parameter hints (with name X, using Y)

### Phase 4: Persistent Indexing Pipeline

#### 4.1 Incremental Index Management

Create an index manager for persistent state:

```rust
pub struct IndexManager {
    vector_store: Arc<dyn VectorStore>,
    sparse_index: Arc<BM25Index>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    config: IndexConfig,
}

pub struct IndexConfig {
    pub index_path: PathBuf,            // ~/.skill-engine/index/
    pub embedding_model: String,
    pub embedding_dimensions: usize,
    pub chunk_size: usize,              // For long docs
    pub chunk_overlap: usize,
    pub index_on_startup: bool,
    pub watch_for_changes: bool,
    pub batch_size: usize,              // Embedding batch size
}

impl IndexManager {
    /// Index a single skill (on install/update)
    pub async fn index_skill(&self, skill_path: &Path) -> Result<IndexStats>;

    /// Remove skill from index (on uninstall)
    pub async fn remove_skill(&self, skill_name: &str) -> Result<()>;

    /// Full reindex (on model change or corruption)
    pub async fn reindex_all(&self) -> Result<IndexStats>;

    /// Incremental update (check hashes, update changed)
    pub async fn sync(&self) -> Result<SyncStats>;

    /// Background index optimization
    pub async fn optimize(&self) -> Result<()>;
}
```

#### 4.2 Index Versioning and Migration

Handle embedding model changes gracefully:

```rust
pub struct IndexMetadata {
    pub version: u32,
    pub embedding_model: String,
    pub embedding_dimensions: usize,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub document_count: usize,
    pub skill_checksums: HashMap<String, String>,
}

// On startup, check if index is compatible
// If embedding model changed, trigger full reindex
```

### Phase 5: Configuration and Integration

#### 5.1 Configuration Schema

Extend `.skill-engine.toml` with RAG configuration:

```toml
[search]
# Vector store backend
backend = "qdrant"  # in-memory | qdrant | pinecone | weaviate | pgvector | lancedb

# Embedding configuration
[search.embedding]
provider = "fastembed"  # fastembed | openai | ollama | voyage | cohere
model = "BAAI/bge-small-en-v1.5"
dimensions = 384
batch_size = 32

# Hybrid search settings
[search.retrieval]
enable_hybrid = true
dense_weight = 0.7
sparse_weight = 0.3
first_stage_k = 100
rerank_k = 20
final_k = 5

# Reranking configuration
[search.reranker]
enabled = true
provider = "fastembed"  # fastembed | cohere | voyage
model = "BAAI/bge-reranker-base"

# Context compression
[search.context]
max_tokens_per_result = 200
max_total_tokens = 800
include_examples = false
compression = "extractive"  # none | extractive | abstractive

# Backend-specific configuration
[search.qdrant]
url = "http://localhost:6334"
api_key = "${QDRANT_API_KEY}"
collection = "skill-tools"

[search.pinecone]
api_key = "${PINECONE_API_KEY}"
environment = "us-west1-gcp"
index = "skill-tools"

[search.pgvector]
connection_string = "${DATABASE_URL}"
table = "tool_embeddings"
```

#### 5.2 Environment Variables

Support all configuration via environment:

```bash
# Vector Store Backends
SKILL_SEARCH_BACKEND=qdrant
QDRANT_URL=http://localhost:6334
QDRANT_API_KEY=...
PINECONE_API_KEY=...
WEAVIATE_URL=...
DATABASE_URL=postgres://...

# Embedding Providers
SKILL_EMBEDDING_PROVIDER=fastembed
OPENAI_API_KEY=...
VOYAGE_API_KEY=...
COHERE_API_KEY=...
OLLAMA_URL=http://localhost:11434

# Reranker
SKILL_RERANKER_PROVIDER=cohere
```

#### 5.3 MCP Server Enhancement

Update MCP server with new search capabilities:

```rust
// New MCP tools
pub async fn search_skills_v2(
    query: String,
    options: SearchOptions,
) -> Result<SearchResponse>;

pub struct SearchOptions {
    pub top_k: Option<usize>,
    pub filters: Option<Filters>,
    pub include_execution_hints: Option<bool>,
    pub max_context_tokens: Option<usize>,
    pub expand_query: Option<bool>,
}

pub struct Filters {
    pub categories: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    pub min_relevance: Option<f32>,
}
```

### Phase 6: Analytics and Optimization

#### 6.1 Usage Analytics Collection

Track search and execution patterns:

```rust
pub struct SearchAnalytics {
    pub query: String,
    pub results_returned: Vec<String>,  // tool IDs
    pub result_selected: Option<String>,
    pub execution_success: Option<bool>,
    pub latency_ms: u32,
    pub timestamp: DateTime<Utc>,
}

// Store in SQLite for local analytics
// Optional: aggregate to remote service
```

#### 6.2 Learning-to-Rank Integration

Use analytics to improve ranking:

```rust
pub struct LTRFeatures {
    pub dense_score: f32,
    pub sparse_score: f32,
    pub rerank_score: f32,
    pub usage_count: f32,           // Normalized
    pub success_rate: f32,
    pub recency_score: f32,         // Time decay
    pub query_tool_overlap: f32,    // Keyword match
    pub category_match: f32,        // Query intent vs tool category
}

// Train simple gradient boosted model on click data
// Or use weighted combination with learned weights
```

## Implementation Phases

### Phase 1: Foundation (Weeks 1-3)
**Tasks:**
1. Create `VectorStore` trait and in-memory implementation
2. Create `EmbeddingProvider` trait and unify existing providers
3. Enhance `ToolDocument` schema with new fields
4. Implement `IndexManager` with persistence
5. Add Qdrant backend (local Docker + cloud)
6. Add configuration schema to `.skill-engine.toml`

**Deliverables:**
- Persistent vector index in Qdrant
- Sub-100ms warm search latency
- Configuration-driven backend selection

### Phase 2: Hybrid Search (Weeks 4-5)
**Tasks:**
1. Integrate tantivy for BM25 indexing
2. Implement RRF fusion
3. Add cross-encoder reranking (fastembed)
4. Create `HybridRetriever` orchestration
5. Add Cohere/Voyage reranker options

**Deliverables:**
- Hybrid search with configurable weights
- Reranking pipeline with multiple providers
- 30%+ precision improvement on test queries

### Phase 3: Context Optimization (Weeks 6-7)
**Tasks:**
1. Implement `ContextCompressor`
2. Design execution-ready response format
3. Add query understanding layer
4. Implement progressive context expansion
5. Add context token counting and limits

**Deliverables:**
- 70% reduction in context tokens
- Execution-ready tool interfaces
- Query intent classification

### Phase 4: Additional Backends (Weeks 8-9)
**Tasks:**
1. Add Pinecone backend
2. Add Weaviate backend
3. Add pgvector backend
4. Add LanceDB backend
5. Backend compatibility testing

**Deliverables:**
- 5 vector store backends
- Migration tooling between backends
- Performance benchmarks per backend

### Phase 5: Advanced Features (Weeks 10-12)
**Tasks:**
1. Implement usage analytics collection
2. Add learning-to-rank features
3. Create index versioning and migration
4. Add multi-tenant/workspace support
5. Performance optimization and caching

**Deliverables:**
- Analytics dashboard (CLI)
- Improved ranking from usage data
- Production-ready system

## Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Search Latency (p50) | 800ms | <100ms | Benchmark suite |
| Search Latency (p99) | 2000ms | <300ms | Benchmark suite |
| Precision@5 | ~0.6 | >0.85 | Test query set |
| Context Tokens/Query | ~2000 | <600 | Token counter |
| Index Build Time | N/A (per-query) | <5s for 100 skills | Benchmark |
| Memory Usage | High (in-memory) | <100MB baseline | Profiler |

## Testing Strategy

### Unit Tests
- Vector store trait implementations
- Embedding provider abstractions
- BM25 indexing correctness
- RRF fusion math
- Context compression accuracy

### Integration Tests
- End-to-end search pipeline
- Backend switching
- Index persistence and recovery
- Configuration loading

### Benchmark Suite
- Latency benchmarks (cold/warm)
- Throughput benchmarks (queries/sec)
- Memory usage profiling
- Precision/recall on test queries

### Test Query Set
Create 100+ annotated queries with expected results:
```json
{
  "query": "create a kubernetes deployment with nginx",
  "expected_top_3": ["kubernetes@*/create_deployment", "kubernetes@*/apply_manifest", "docker@*/run"],
  "intent": "tool_execution",
  "category": "infrastructure"
}
```

## Dependencies and Prerequisites

### New Rust Crates
```toml
# Vector Stores
qdrant-client = "1.10"
pinecone-sdk = "0.2"
lancedb = "0.4"
sqlx = { version = "0.7", features = ["postgres"] }

# Search
tantivy = "0.22"

# Reranking
fastembed = "4.0"  # Includes reranker models

# Analytics
rusqlite = "0.31"

# Utilities
tiktoken-rs = "0.5"  # Token counting
```

### External Services (Optional)
- Qdrant Cloud or self-hosted
- Pinecone (cloud only)
- Weaviate Cloud or self-hosted
- PostgreSQL with pgvector extension
- Cohere API (for reranking)
- Voyage AI API (for code embeddings)

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Embedding model quality varies | Search relevance degraded | Default to proven models (BGE, MiniLM), provide benchmarks |
| Remote vector DB latency | Search slower than local | Aggressive caching, local fallback |
| Index corruption | Search unavailable | Checksums, automatic rebuild, backup |
| Breaking config changes | User frustration | Version config schema, migration scripts |
| Memory pressure from large indexes | OOM crashes | Streaming, pagination, disk-backed stores |

## Future Enhancements (Out of Scope)

1. **Multi-modal search**: Image/diagram search for skills
2. **Federated search**: Search across multiple skill registries
3. **Real-time indexing**: WebSocket updates on skill changes
4. **Custom embedding fine-tuning**: Train domain-specific models
5. **Graph-based retrieval**: Knowledge graph for skill relationships
6. **Conversational search**: Multi-turn refinement with memory

## References

- [Vector Database Comparison 2025](https://liquidmetal.ai/casesAndBlogs/vector-comparison/)
- [RAG Optimization with Hybrid Search](https://superlinked.com/vectorhub/articles/optimizing-rag-with-hybrid-search-reranking)
- [FastEmbed for Rig.rs](https://redandgreen.co.uk/fastembed-for-rig-rs/rust-programming/)
- [Qdrant Documentation](https://qdrant.tech/documentation/)
- [Choosing Reranking Models 2025](https://www.zeroentropy.dev/articles/ultimate-guide-to-choosing-the-best-reranking-model-in-2025)
- [IBM RAG Techniques](https://www.ibm.com/think/topics/rag-techniques)
