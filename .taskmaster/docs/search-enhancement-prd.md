# Search Enhancement & Analytics PRD

## Executive Summary

Enhance the Skill Engine semantic search interface with professional UI/UX, result drill-down capabilities, feedback collection system for ML improvement, and comprehensive search analytics with MCP client tracking.

## Problem Statement

The current search test page is functional but lacks:
1. Professional, polished UI/UX for search results
2. Ability to drill down into search results (view full tool details, parameters, examples)
3. Feedback mechanism to improve vector search quality over time
4. Search history and analytics (especially for MCP client searches)
5. Contextual awareness tracking (which agent/client made which searches)
6. Performance metrics and insights dashboard

## Goals & Success Criteria

### Primary Goals
1. **Enhanced Search UI** - Beautiful, intuitive search interface with drill-down
2. **Feedback System** - Capture user feedback on search result relevance
3. **Search Analytics** - Track all searches with client context and performance metrics
4. **MCP Integration** - Log MCP client searches with proper attribution
5. **Performance Monitoring** - Dashboard showing search quality metrics

### Success Metrics
- Search result click-through rate increases by 40%
- Average time-to-find-tool decreases by 50%
- Feedback collection rate > 20% of searches
- 100% of MCP searches tracked with client info
- Search quality score improves by 30% after 2 weeks of feedback

## User Personas

1. **Direct Users** - Developers using the web UI to search for skills
2. **MCP Clients** - Claude Code, Cline, other agents searching via MCP
3. **System Admins** - Need analytics to understand usage patterns
4. **ML Engineers** - Need feedback data to improve embeddings

## Requirements

### 1. Enhanced Search Results UI

**Must Have:**
- Card-based result layout with clear visual hierarchy
- Skill icon/avatar for each result
- Score visualization (progress bar or badge)
- Expandable sections for tool details
- Syntax-highlighted parameter schemas
- Quick action buttons (copy, open docs, run)
- Empty state with helpful suggestions
- Loading skeletons (not just spinners)

**Nice to Have:**
- Result grouping by skill
- Filtering by score threshold
- Sorting options (relevance, alphabetical, recent)
- Keyboard navigation (arrow keys, enter to expand)
- Dark mode optimized colors

**UI Mockup Structure:**
```
┌─────────────────────────────────────────────┐
│ Search: "kubernetes pods"           [Re-index] │
│ ┌──────────────────────────────────────────┐ │
│ │ [Search input]                      [🔍] │ │
│ └──────────────────────────────────────────┘ │
├─────────────────────────────────────────────┤
│ 127 results in 38ms | Sort: Relevance ▾     │
├─────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────┐ │
│ │ 🎯 kubernetes / get        Score: 0.96  │ │
│ │ Get Kubernetes resources                 │ │
│ │ ⚡ 24 uses | ⏱ avg 250ms | 👍 95%       │ │
│ │ [View Details ▼] [Run] [Copy] [Feedback]│ │
│ └─────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────┐ │
│ │ 🎯 kubernetes / describe   Score: 0.94  │ │
│ │ ...expanded detail view...               │ │
│ │ Parameters:                              │ │
│ │  • resource: string (required)           │ │
│ │  • name: string (required)               │ │
│ │  • namespace: string (optional)          │ │
│ │ Examples: [View 3 examples]              │ │
│ │ [Collapse ▲] [Run] [Copy]  👍 👎        │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

### 2. Result Drill-Down System

**Must Have:**
- Expandable result cards (click to expand inline)
- Full parameter documentation display
- Type information with syntax highlighting
- Example usage code snippets
- Related tools recommendations
- Copy-to-clipboard for parameters/examples

**Nice to Have:**
- Link to full SKILL.md documentation
- Version history if available
- Dependency information
- Performance characteristics
- Similar tools suggestions (vector similarity)

**Data to Display:**
```typescript
{
  skill: "kubernetes",
  tool: "get",
  description: "Get Kubernetes resources",
  score: 0.96,
  parameters: [
    { name: "resource", type: "string", required: true, 
      description: "Resource type (pods, services...)" },
    ...
  ],
  examples: [
    { description: "Get all pods", code: "...", language: "bash" }
  ],
  stats: {
    usage_count: 24,
    avg_duration_ms: 250,
    success_rate: 0.95,
    last_used: "2024-01-05T10:30:00Z"
  }
}
```

### 3. Feedback Collection System

**Must Have:**
- Thumbs up/down buttons on each result
- Feedback reasons (too broad, wrong skill, missing info, perfect)
- Anonymous feedback storage
- Feedback aggregation by query-result pair
- API endpoint: `POST /api/search/feedback`

**Nice to Have:**
- Text comment field for detailed feedback
- "Report incorrect info" option
- Feedback review dashboard for admins
- A/B testing different ranking algorithms
- Feedback-based reranking model

**Database Schema:**
```sql
CREATE TABLE search_feedback (
  id UUID PRIMARY KEY,
  query TEXT NOT NULL,
  result_id TEXT NOT NULL,  -- skill:tool
  score FLOAT NOT NULL,
  rank INTEGER NOT NULL,
  feedback_type TEXT NOT NULL,  -- thumbs_up, thumbs_down
  reason TEXT,  -- optional reason
  comment TEXT,  -- optional detailed comment
  client_type TEXT,  -- web, mcp, api
  client_id TEXT,  -- MCP client identifier
  session_id TEXT,  -- user session
  timestamp TIMESTAMPTZ DEFAULT NOW(),
  context JSONB  -- additional context (user agent, etc.)
);

CREATE INDEX idx_feedback_query ON search_feedback(query);
CREATE INDEX idx_feedback_result ON search_feedback(result_id);
CREATE INDEX idx_feedback_timestamp ON search_feedback(timestamp);
```

**API Endpoint:**
```rust
POST /api/search/feedback
{
  "query": "kubernetes pods",
  "result_id": "kubernetes:get",
  "score": 0.96,
  "rank": 1,
  "feedback_type": "thumbs_up",
  "reason": "perfect_match",
  "comment": "Exactly what I needed",
  "session_id": "abc123"
}
```

### 4. Search History & Analytics

**Must Have:**
- Search history page showing all queries
- Filtering by date range, client type, query text
- Per-query metrics (results count, avg score, click-through)
- Drill-down into individual search execution
- Export to CSV/JSON
- API endpoint: `GET /api/search/history`

**Nice to Have:**
- Real-time search activity feed
- Popular queries dashboard
- Failed searches (0 results) tracking
- Query performance over time graphs
- Heatmap of search times (when do people search)
- Geographic distribution (if available)

**UI Components:**
```
Search History Page:
┌───────────────────────────────────────────────┐
│ Search History                                 │
│ Filters: [Last 7 days ▾] [All clients ▾]     │
├───────────────────────────────────────────────┤
│ Query                Client     Results  Time  │
├───────────────────────────────────────────────┤
│ kubernetes pods     Web UI      12      38ms  │
│ deploy service      MCP:Claude  8       45ms  │
│ terraform aws       API Client  24      52ms  │
└───────────────────────────────────────────────┘

Analytics Dashboard:
┌─────────────────┬─────────────────┬─────────────┐
│ Total Searches  │ Avg Results     │ Avg Latency │
│     2,847       │      8.3        │    42ms     │
└─────────────────┴─────────────────┴─────────────┘
┌──────────────────────────────────────────────┐
│ Top Queries (Last 7 Days)                    │
│ 1. kubernetes pods          247 searches     │
│ 2. deploy service           183 searches     │
│ 3. terraform infrastructure 142 searches     │
└──────────────────────────────────────────────┘
```

**Database Schema:**
```sql
CREATE TABLE search_history (
  id UUID PRIMARY KEY,
  query TEXT NOT NULL,
  top_k INTEGER NOT NULL,
  filters JSONB,
  results_count INTEGER NOT NULL,
  avg_score FLOAT,
  duration_ms INTEGER NOT NULL,
  client_type TEXT NOT NULL,
  client_id TEXT,
  client_version TEXT,
  session_id TEXT,
  user_agent TEXT,
  ip_address INET,
  timestamp TIMESTAMPTZ DEFAULT NOW(),
  embedding_provider TEXT,
  vector_backend TEXT,
  hybrid_enabled BOOLEAN,
  reranking_enabled BOOLEAN
);

CREATE INDEX idx_history_timestamp ON search_history(timestamp);
CREATE INDEX idx_history_query ON search_history(query);
CREATE INDEX idx_history_client ON search_history(client_type, client_id);
CREATE INDEX idx_history_session ON search_history(session_id);
```

### 5. MCP Client Integration

**Must Have:**
- MCP search tool includes client identification
- Log every MCP search with client metadata
- Client analytics page showing per-client usage
- API endpoint to query searches by client
- Client performance comparison

**MCP Tool Enhancement:**
```typescript
// Add client context to MCP tool
{
  name: "search_skills",
  description: "Search for skills semantically",
  inputSchema: {
    type: "object",
    properties: {
      query: { type: "string", description: "Search query" },
      top_k: { type: "number", default: 10 },
      // New fields for tracking
      client_id: { type: "string", description: "MCP client identifier" },
      client_version: { type: "string" },
      session_id: { type: "string" },
      context: { type: "object", description: "Additional context" }
    },
    required: ["query"]
  }
}
```

**Client Analytics Page:**
```
MCP Client Analytics:
┌──────────────────────────────────────────────┐
│ Client: claude-code-desktop                   │
│ Version: 1.5.0                                │
│ Total Searches: 1,247                         │
│ Avg Results: 9.2                              │
│ Success Rate: 94% (results > 0)              │
│ Avg Latency: 41ms                             │
├──────────────────────────────────────────────┤
│ Top Queries:                                  │
│ 1. kubernetes deployment     87 searches     │
│ 2. terraform aws             65 searches     │
│ 3. docker compose            43 searches     │
└──────────────────────────────────────────────┘
```

**Nice to Have:**
- Client usage trends over time
- Client-specific search quality metrics
- Client feature usage (filters, top_k, etc.)
- Client error rate tracking
- Per-client feedback aggregation

### 6. Performance Monitoring Dashboard

**Must Have:**
- Real-time metrics display
- Search latency percentiles (p50, p95, p99)
- Index size and document count
- Cache hit rates
- Error rate tracking

**Nice to Have:**
- Alert system for degraded performance
- Comparison to baseline metrics
- A/B test results visualization
- Cost tracking (API calls for OpenAI, etc.)
- Capacity planning recommendations

**Metrics to Track:**
```typescript
{
  search_performance: {
    total_searches: 2847,
    avg_latency_ms: 42,
    p50_latency_ms: 38,
    p95_latency_ms: 95,
    p99_latency_ms: 180,
    cache_hit_rate: 0.45,
    error_rate: 0.002
  },
  index_stats: {
    total_documents: 129,
    index_size_bytes: 2457600,
    last_indexed: "2024-01-05T10:00:00Z",
    stale_documents: 0
  },
  feedback_metrics: {
    total_feedback: 247,
    thumbs_up_rate: 0.82,
    thumbs_down_rate: 0.18,
    avg_score_thumbs_up: 0.91,
    avg_score_thumbs_down: 0.54
  }
}
```

## Technical Architecture

### Backend Components

1. **Database Layer**
   - SQLite for development (file-based)
   - PostgreSQL for production (optional)
   - Schema migrations via `sqlx` or `diesel`
   - Connection pooling

2. **API Endpoints**
   ```
   POST   /api/search/feedback        - Submit feedback
   GET    /api/search/history          - Get search history
   GET    /api/search/history/:id      - Get specific search
   GET    /api/search/analytics        - Get analytics dashboard
   GET    /api/search/clients          - List MCP clients
   GET    /api/search/clients/:id      - Get client analytics
   POST   /api/search/export           - Export data to CSV/JSON
   ```

3. **Search Logging Middleware**
   - Intercept all search requests
   - Log to database asynchronously
   - Include request metadata
   - Performance tracking

4. **Feedback Processing**
   - Store feedback immediately
   - Aggregate feedback for ML pipeline
   - Calculate quality scores
   - Trigger reranking updates

5. **MCP Server Updates**
   - Accept client metadata in search requests
   - Validate and sanitize client data
   - Log client information
   - Return client-specific responses

### Frontend Components

1. **Enhanced Search Results Component**
   - File: `src/pages/search_test.rs`
   - Card-based layout with Tailwind CSS
   - Expandable sections using Yew state
   - Feedback buttons with API calls

2. **Search History Page**
   - File: `src/pages/search_history.rs`
   - Table view with pagination
   - Filtering and sorting
   - Export functionality

3. **Analytics Dashboard**
   - File: `src/pages/search_analytics.rs`
   - Charts using charting library
   - Real-time metrics
   - Client breakdown view

4. **Feedback Modal/Toast**
   - Quick feedback collection
   - Optional detailed feedback form
   - Thank you message
   - Analytics tracking

### Data Flow

```
Search Request
    ↓
[Search Handler]
    ↓
[Logging Middleware] ──→ [Database: search_history]
    ↓
[SearchPipeline]
    ↓
[Results + Metadata]
    ↓
[Frontend Display]
    ↓
[User Feedback] ──→ [Database: search_feedback]
    ↓
[ML Pipeline] (future: retrain embeddings)
```

## Implementation Phases

### Phase 1: Database & Logging (Week 1)
- Set up SQLite database
- Create schema for search_history and search_feedback
- Implement logging middleware
- Add search history API endpoint
- Basic history page in frontend

### Phase 2: Enhanced UI (Week 1-2)
- Redesign search results cards
- Add expandable sections
- Implement drill-down views
- Add loading states and skeletons
- Polish with animations

### Phase 3: Feedback System (Week 2)
- Add feedback buttons to results
- Implement feedback API endpoint
- Store feedback in database
- Create feedback review page (admin)
- Add feedback metrics to analytics

### Phase 4: MCP Integration (Week 2-3)
- Update MCP tool schema with client fields
- Modify search handler to accept client metadata
- Log client information
- Create client analytics page
- Test with Claude Code MCP client

### Phase 5: Analytics Dashboard (Week 3)
- Build analytics page with metrics
- Add charts for trends
- Implement real-time updates
- Export functionality
- Performance monitoring

### Phase 6: Polish & Optimization (Week 4)
- Performance tuning
- UI polish
- Documentation
- Testing (unit, integration, E2E)
- Deployment guide

## Testing Strategy

1. **Unit Tests**
   - Database operations (CRUD)
   - API endpoint handlers
   - Feedback aggregation logic
   - Logging middleware

2. **Integration Tests**
   - End-to-end search flow with logging
   - Feedback submission and retrieval
   - MCP client search tracking
   - Analytics calculations

3. **UI Tests**
   - Search result rendering
   - Expand/collapse functionality
   - Feedback button interactions
   - History page filtering

4. **Performance Tests**
   - Search latency under load
   - Database query performance
   - Frontend rendering speed
   - Concurrent user handling

## Security & Privacy

1. **Data Privacy**
   - No PII collection by default
   - Anonymized search queries
   - Optional user identification
   - GDPR-compliant data export

2. **Access Control**
   - Admin-only analytics pages
   - API authentication for MCP clients
   - Rate limiting on feedback endpoints
   - SQL injection prevention

3. **Data Retention**
   - Configurable retention period
   - Automatic cleanup of old data
   - Export before deletion
   - Audit logs

## Success Metrics & KPIs

1. **User Engagement**
   - Search frequency (daily/weekly active users)
   - Average searches per session
   - Result drill-down rate
   - Feedback submission rate

2. **Search Quality**
   - Click-through rate (CTR) on top result
   - Average result relevance score
   - Zero-result queries rate
   - Feedback positive rate

3. **Performance**
   - Search latency p95
   - Index freshness (time since last index)
   - System uptime
   - Error rate

4. **MCP Adoption**
   - Number of active MCP clients
   - MCP search volume vs web searches
   - Client satisfaction (feedback)
   - Integration adoption rate

## Future Enhancements

1. **ML Improvements**
   - Fine-tune embeddings with feedback data
   - Learn from click patterns
   - Personalized search ranking
   - Query understanding improvements

2. **Advanced Features**
   - Natural language query expansion
   - Multi-modal search (code + docs)
   - Federated search across skills
   - Real-time collaboration (shared searches)

3. **Integration**
   - Slack bot for search
   - VS Code extension
   - CLI search command
   - API clients for popular languages

## Dependencies

- `sqlx` or `diesel` - Database ORM
- `chrono` - Timestamp handling
- `uuid` - ID generation
- `serde_json` - JSON handling
- Chart library for frontend (consider `plotters` or JS charting via trunk)

## Risks & Mitigation

1. **Database Performance** - Risk: slow queries with large history
   - Mitigation: Proper indexing, pagination, archival strategy

2. **Feedback Spam** - Risk: fake feedback skewing results
   - Mitigation: Rate limiting, session validation, anomaly detection

3. **MCP Client Adoption** - Risk: clients don't send metadata
   - Mitigation: Backward compatibility, documentation, incentives

4. **UI Complexity** - Risk: overwhelming users with features
   - Mitigation: Progressive disclosure, optional features, user testing

## Open Questions

1. Should feedback be used immediately for reranking or batch processed?
2. What's the retention period for search history?
3. Should we support user accounts for personalized search?
4. How to handle multi-tenant scenarios (multiple organizations)?
5. What level of analytics should be exposed to MCP clients?

## Appendix

### A. Database Migrations

Migration files in `migrations/`:
- `001_create_search_history.sql`
- `002_create_search_feedback.sql`
- `003_add_indexes.sql`

### B. API Documentation

OpenAPI/Swagger spec for all new endpoints

### C. UI Component Library

Reusable components:
- SearchResultCard
- FeedbackButton
- ExpandableSection
- MetricsCard
- HistoryTable
- AnalyticsChart

### D. MCP Integration Guide

Step-by-step guide for MCP client developers to:
1. Add client metadata to search requests
2. Handle search responses
3. Submit feedback
4. View analytics

---

**Document Version:** 1.0  
**Last Updated:** 2025-01-05  
**Author:** Product Team  
**Status:** Ready for Implementation
