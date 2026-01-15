# RAG Search Pipeline

Skill Engine includes a production-ready RAG (Retrieval-Augmented Generation) pipeline for intelligent tool discovery. This document covers the architecture, components, and configuration options.

## Overview

The RAG pipeline enables:
- **Semantic tool discovery** - Find tools using natural language queries
- **Hybrid search** - Combine dense (vector) and sparse (BM25) retrieval
- **Intelligent reranking** - Improve precision with cross-encoder models
- **Token-efficient output** - Compress context for LLM consumption
- **Persistent indexing** - Incremental updates with change detection

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            Query Pipeline                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  User Query                                                              │
│      │                                                                   │
│      ▼                                                                   │
│  ┌─────────────────┐                                                     │
│  │ QueryProcessor  │ → Intent classification + entity extraction         │
│  └────────┬────────┘                                                     │
│           │                                                              │
│           ▼                                                              │
│  ┌─────────────────────────────────────────────────────────┐             │
│  │              HybridRetriever (optional)                  │             │
│  │  ┌───────────────┐      ┌───────────────┐               │             │
│  │  │ Dense Search  │      │ BM25 Search   │               │             │
│  │  │ (VectorStore) │      │ (Tantivy)     │               │             │
│  │  └───────┬───────┘      └───────┬───────┘               │             │
│  │          │                      │                        │             │
│  │          └──────────┬───────────┘                        │             │
│  │                     ▼                                    │             │
│  │              ┌────────────┐                              │             │
│  │              │ RRF Fusion │                              │             │
│  │              └─────┬──────┘                              │             │
│  └────────────────────┼─────────────────────────────────────┘             │
│                       │                                                  │
│                       ▼                                                  │
│              ┌─────────────────┐                                         │
│              │    Reranker     │ → Cross-encoder scoring (optional)      │
│              └────────┬────────┘                                         │
│                       │                                                  │
│                       ▼                                                  │
│              ┌─────────────────────┐                                     │
│              │ ContextCompressor   │ → Token-aware output formatting     │
│              └────────┬────────────┘                                     │
│                       │                                                  │
│                       ▼                                                  │
│              Final Results                                               │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Components

### 1. VectorStore

Abstract trait for pluggable vector backends.

```rust
use skill_runtime::{VectorStore, InMemoryVectorStore, EmbeddedDocument};

// Create in-memory store
let store = InMemoryVectorStore::new(384); // 384 dimensions

// Upsert documents
let docs = vec![
    EmbeddedDocument {
        id: "tool-1".to_string(),
        content: "List Kubernetes pods".to_string(),
        embedding: vec![0.1, 0.2, ...], // 384-dim vector
        metadata: DocumentMetadata {
            skill_name: Some("kubernetes".to_string()),
            tool_name: Some("get".to_string()),
            ..Default::default()
        },
    }
];
store.upsert(docs).await?;

// Search
let results = store.search(&query_embedding, 10, None).await?;
```

**Backends:**

| Backend | Feature Flag | Use Case |
|---------|--------------|----------|
| `InMemoryVectorStore` | default | Development, small catalogs |
| `QdrantVectorStore` | `qdrant` | Production, large catalogs |

### 2. Embedding Providers

Generate embeddings from text using various providers.

```rust
use skill_runtime::{EmbeddingProvider, FastEmbedProvider, FastEmbedModel};

// Local embeddings (no API key needed)
let provider = FastEmbedProvider::new(FastEmbedModel::AllMiniLML6V2)?;

// Generate embeddings
let texts = vec!["List kubernetes pods", "Deploy application"];
let embeddings = provider.embed(&texts).await?;
```

**Providers:**

| Provider | Feature | Models | Notes |
|----------|---------|--------|-------|
| FastEmbed | default | all-minilm, bge-small, bge-base | Local, offline, fast |
| OpenAI | default | text-embedding-3-small/large, ada-002 | Requires API key |
| Ollama | default | nomic-embed-text, mxbai-embed-large | Self-hosted |

### 3. Hybrid Search (BM25 + Dense)

Combine sparse and dense retrieval for better recall.

```rust
use skill_runtime::{BM25Index, BM25Config, HybridRetriever, HybridConfig};

// Create BM25 index
let bm25 = BM25Index::new(BM25Config::default())?;
bm25.add_documents(documents)?;

// Create hybrid retriever
let config = HybridConfig {
    dense_weight: 0.7,
    sparse_weight: 0.3,
    first_stage_k: 100,
    final_k: 10,
    ..Default::default()
};
let retriever = HybridRetriever::new(vector_store, bm25, config);

// Search
let results = retriever.search(&query, &query_embedding).await?;
```

**Fusion Methods:**

| Method | Description |
|--------|-------------|
| `ReciprocalRank` | RRF with configurable k parameter (default: 60) |
| `WeightedSum` | Normalize and weight scores directly |

### 4. Cross-encoder Reranking

Improve precision by rescoring candidates with a cross-encoder.

```rust
use skill_runtime::{Reranker, FastEmbedReranker, RerankerModel, RerankerConfig};

let config = RerankerConfig {
    model: RerankerModel::BGERerankerBase,
    top_k: 5,
    ..Default::default()
};
let reranker = FastEmbedReranker::new(config)?;

// Rerank candidates
let reranked = reranker.rerank(&query, candidates, 5)?;
```

**Models:**

| Model | Speed | Quality | Languages |
|-------|-------|---------|-----------|
| `BGERerankerBase` | Fast | Good | English |
| `BGERerankerV2M3` | Medium | Better | Multilingual |
| `JinaRerankerV1TurboEn` | Fast | Good | English |
| `JinaRerankerV2BaseMultilingual` | Slow | Best | Multilingual |

### 5. Context Compression

Generate token-efficient output for LLM consumption.

```rust
use skill_runtime::{ContextCompressor, CompressionConfig, CompressionStrategy};

let config = CompressionConfig {
    max_tokens_per_result: 200,
    max_total_tokens: 800,
    strategy: CompressionStrategy::Template,
    ..Default::default()
};
let compressor = ContextCompressor::new(config)?;

// Compress search results
let compressed = compressor.compress(search_results)?;
println!("Tokens used: {}", compressed.total_tokens);
```

**Strategies:**

| Strategy | Description |
|----------|-------------|
| `Extractive` | First sentence + parameter list |
| `Template` | Structured format with key info |
| `Progressive` | More detail for higher-ranked results |
| `None` | No compression, full content |

### 6. Query Understanding

Classify intent and extract entities for better retrieval.

```rust
use skill_runtime::{QueryProcessor, QueryIntent};

let mut processor = QueryProcessor::new();
processor.add_known_skill("kubernetes");
processor.add_known_tool("get");

let processed = processor.process("how do I list k8s pods?");

println!("Intent: {:?}", processed.intent);      // ToolDiscovery
println!("Confidence: {}", processed.confidence); // 0.85
println!("Entities: {:?}", processed.entities);   // [("kubernetes", SkillName)]
println!("Expanded: {}", processed.expanded_query); // "list kubernetes pods"
```

**Intents:**

| Intent | Description | Example Query |
|--------|-------------|---------------|
| `ToolDiscovery` | Finding tools for a task | "how to deploy to kubernetes" |
| `ToolExecution` | Running a specific tool | "run kubectl get pods" |
| `ToolDocumentation` | Understanding a tool | "what does scale do?" |
| `Comparison` | Comparing tools | "difference between apply and create" |
| `Troubleshooting` | Fixing issues | "why is my pod crashing?" |
| `General` | General questions | "what is kubernetes?" |

### 7. Persistent Index Manager

Manage index lifecycle with incremental updates.

```rust
use skill_runtime::{IndexManager, IndexConfig};

let config = IndexConfig::with_path("~/.skill-engine/index")
    .with_model("all-minilm", 384)
    .with_chunk_size(32);

let mut manager = IndexManager::new(config)?;

// Plan sync (detects changes via content hash)
let plan = manager.plan_sync(&current_skills)?;
println!("To add: {}", plan.to_add.len());
println!("To update: {}", plan.to_update.len());
println!("To remove: {}", plan.to_remove.len());

// Execute sync
let result = manager.sync(plan, &embedding_provider).await?;
```

## Configuration

### TOML Configuration

Create `skill.toml` in your project:

```toml
[search]
# Backend type: "inmemory" or "qdrant"
backend = { type = "inmemory" }

[search.embedding]
# Provider: "fastembed", "openai", "ollama"
provider = "fastembed"
# Model name (provider-specific)
model = "all-minilm"
# Embedding dimensions
dimensions = 384
# Batch size for embedding generation
batch_size = 32
# OpenAI API key (if provider = "openai")
# openai_api_key = "sk-..."
# Ollama host (if provider = "ollama")
# ollama_host = "http://localhost:11434"

[search.retrieval]
# Enable hybrid (dense + sparse) search
enable_hybrid = true
# Weight for dense (vector) search
dense_weight = 0.7
# Weight for sparse (BM25) search
sparse_weight = 0.3
# Number of results for first stage retrieval
first_stage_k = 100
# Number of results to rerank
rerank_k = 20
# Final number of results to return
final_k = 5
# Fusion method: "reciprocal_rank" or "weighted_sum"
fusion_method = "reciprocal_rank"
# RRF k parameter
rrf_k = 60.0

[search.reranker]
# Enable cross-encoder reranking
enabled = false
# Reranker provider: "fastembed" or "cohere"
provider = "fastembed"
# Reranker model
model = "bge-reranker-base"
# Maximum documents to rerank
max_documents = 50

[search.context]
# Maximum tokens per result
max_tokens_per_result = 200
# Maximum total tokens
max_total_tokens = 800
# Include code examples in output
include_examples = false
# Compression strategy: "extractive", "template", "progressive", "none"
compression = "template"

[search.qdrant]
# Qdrant URL
url = "http://localhost:6334"
# API key (for Qdrant Cloud)
# api_key = "..."
# Collection name
collection = "skill-tools"
# Enable TLS
tls = false

[search.index]
# Index directory path (default: ~/.skill-engine/index)
# path = "/custom/path"
# Index on startup
index_on_startup = true
# Watch for skill changes
watch_for_changes = false
```

### Environment Variables

Override any configuration at runtime:

```bash
# Backend
SKILL_SEARCH_BACKEND=qdrant

# Embedding
SKILL_EMBEDDING_PROVIDER=openai
SKILL_EMBEDDING_MODEL=text-embedding-3-small
SKILL_EMBEDDING_DIMENSIONS=1536

# Retrieval
SKILL_SEARCH_ENABLE_HYBRID=true
SKILL_SEARCH_DENSE_WEIGHT=0.8
SKILL_SEARCH_TOP_K=10

# Reranker
SKILL_RERANKER_ENABLED=true
SKILL_RERANKER_MODEL=bge-reranker-large

# Context
SKILL_CONTEXT_MAX_TOKENS=1000

# Qdrant
QDRANT_URL=http://localhost:6334
QDRANT_API_KEY=your-api-key
```

## Feature Flags

Enable optional features during compilation:

```bash
# Enable all features
cargo build -p skill-runtime --features hybrid-search,reranker,context-compression,qdrant

# Enable specific features
cargo build -p skill-runtime --features hybrid-search
cargo build -p skill-runtime --features reranker
cargo build -p skill-runtime --features qdrant
```

| Feature | Dependencies | Size Impact | Description |
|---------|--------------|-------------|-------------|
| `hybrid-search` | tantivy (~10MB) | Medium | BM25 + RRF fusion |
| `reranker` | fastembed (~50MB models) | Large | Cross-encoder reranking |
| `context-compression` | tiktoken-rs (~5MB) | Small | Token-aware compression |
| `qdrant` | qdrant-client (~2MB) | Small | Production vector DB |

## Performance Characteristics

| Operation | Latency | Notes |
|-----------|---------|-------|
| Embedding generation | 5-20ms | FastEmbed, batch of 32 |
| Vector search (in-memory) | 1-5ms | 10k documents |
| Vector search (Qdrant) | 10-50ms | Network dependent |
| BM25 search | 5-15ms | Tantivy, 10k documents |
| Cross-encoder rerank | 50-200ms | BGE base, 20 candidates |
| Context compression | 1-5ms | 10 results |

## Best Practices

### 1. Choose the Right Embedding Model

| Use Case | Recommended Model |
|----------|------------------|
| General English | `all-minilm` (384-dim) |
| Technical docs | `bge-small-en` (384-dim) |
| Multilingual | `bge-base-multilingual` (768-dim) |
| High accuracy | `text-embedding-3-large` (3072-dim) |

### 2. Tune Hybrid Search Weights

```toml
# More semantic matching (default)
dense_weight = 0.7
sparse_weight = 0.3

# More keyword matching
dense_weight = 0.5
sparse_weight = 0.5

# Pure vector search
enable_hybrid = false
```

### 3. Enable Reranking for Precision

Only enable reranking when:
- You have < 100ms latency budget
- Precision is more important than recall
- Results from first stage need refinement

### 4. Compress Context for Production

```toml
[search.context]
# Be aggressive for large catalogs
max_total_tokens = 500
compression = "extractive"

# Be generous for small catalogs
max_total_tokens = 2000
compression = "template"
```

## Troubleshooting

### High Memory Usage

- Reduce `batch_size` in embedding config
- Use smaller embedding model (384 vs 768 dimensions)
- Enable Qdrant backend for large catalogs

### Slow Search

- Reduce `first_stage_k` in retrieval config
- Disable hybrid search if not needed
- Use lighter reranker model

### Poor Search Quality

- Ensure SKILL.md files have good descriptions
- Add more synonyms to QueryProcessor
- Increase `first_stage_k` for better recall
- Enable reranking for precision

## API Reference

See the Rust documentation:

```bash
cargo doc -p skill-runtime --open
```

Key modules:
- `skill_runtime::vector_store` - VectorStore trait and backends
- `skill_runtime::embeddings` - Embedding providers
- `skill_runtime::search` - Search pipeline components
- `skill_runtime::search_config` - Configuration schema
