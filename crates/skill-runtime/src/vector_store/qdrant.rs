//! Qdrant vector store backend implementation
//!
//! Provides integration with Qdrant vector database for production-ready
//! vector search with support for both local Docker and cloud deployments.
//!
//! # Configuration
//!
//! Set environment variables:
//! - `QDRANT_URL`: Qdrant server URL (default: http://localhost:6334)
//! - `QDRANT_API_KEY`: API key for cloud deployments (optional)
//!
//! Or configure in `.skill-engine.toml`:
//! ```toml
//! [search.qdrant]
//! url = "http://localhost:6334"
//! api_key = "your-api-key"  # optional
//! collection = "skill-tools"
//! ```

use super::{
    DeleteStats, DistanceMetric, DocumentMetadata, EmbeddedDocument, Filter, HealthStatus,
    SearchResult, UpsertStats, VectorStore,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use qdrant_client::qdrant::{
    condition::ConditionOneOf, points_selector::PointsSelectorOneOf,
    Condition, CreateCollectionBuilder, Distance, Filter as QdrantFilter,
    GetPointsBuilder, PointId, PointStruct, PointsIdsList, PointsSelector,
    SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
};
use qdrant_client::Qdrant;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Default Qdrant server URL
pub const DEFAULT_QDRANT_URL: &str = "http://localhost:6334";

/// Default collection name for skill tools
pub const DEFAULT_COLLECTION_NAME: &str = "skill-tools";

/// Configuration for Qdrant vector store
#[derive(Debug, Clone)]
pub struct QdrantConfig {
    /// Qdrant server URL
    pub url: String,
    /// API key for authentication (required for cloud)
    pub api_key: Option<String>,
    /// Collection name
    pub collection_name: String,
    /// Vector dimensions
    pub dimensions: usize,
    /// Distance metric
    pub distance: DistanceMetric,
    /// Create collection if it doesn't exist
    pub auto_create_collection: bool,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("QDRANT_URL").unwrap_or_else(|_| DEFAULT_QDRANT_URL.to_string()),
            api_key: std::env::var("QDRANT_API_KEY").ok(),
            collection_name: DEFAULT_COLLECTION_NAME.to_string(),
            dimensions: 384, // Default for BGE-small / all-minilm
            distance: DistanceMetric::Cosine,
            auto_create_collection: true,
        }
    }
}

impl QdrantConfig {
    /// Create config for local development
    pub fn local() -> Self {
        Self::default()
    }

    /// Create config for cloud deployment
    pub fn cloud(url: &str, api_key: &str) -> Self {
        Self {
            url: url.to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        }
    }

    /// Set vector dimensions
    pub fn with_dimensions(mut self, dims: usize) -> Self {
        self.dimensions = dims;
        self
    }

    /// Set collection name
    pub fn with_collection(mut self, name: &str) -> Self {
        self.collection_name = name.to_string();
        self
    }

    /// Set distance metric
    pub fn with_distance(mut self, metric: DistanceMetric) -> Self {
        self.distance = metric;
        self
    }
}

/// Qdrant vector store backend
///
/// Provides production-ready vector search using Qdrant.
/// Supports both local Docker deployment and Qdrant Cloud.
pub struct QdrantVectorStore {
    client: Arc<Qdrant>,
    config: QdrantConfig,
}

impl QdrantVectorStore {
    /// Create a new Qdrant vector store with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(QdrantConfig::default()).await
    }

    /// Create a new Qdrant vector store with custom configuration
    pub async fn with_config(config: QdrantConfig) -> Result<Self> {
        let client = if let Some(ref api_key) = config.api_key {
            Qdrant::from_url(&config.url)
                .api_key(Some(api_key.as_str()))
                .build()
                .context("Failed to create Qdrant client with API key")?
        } else {
            Qdrant::from_url(&config.url)
                .build()
                .context("Failed to create Qdrant client")?
        };

        let store = Self {
            client: Arc::new(client),
            config,
        };

        // Auto-create collection if configured
        if store.config.auto_create_collection {
            store.ensure_collection().await?;
        }

        Ok(store)
    }

    /// Create for local development (Docker)
    pub async fn local() -> Result<Self> {
        Self::with_config(QdrantConfig::local()).await
    }

    /// Create for cloud deployment
    pub async fn cloud(url: &str, api_key: &str) -> Result<Self> {
        Self::with_config(QdrantConfig::cloud(url, api_key)).await
    }

    /// Ensure the collection exists, creating if necessary
    async fn ensure_collection(&self) -> Result<()> {
        let collections = self
            .client
            .list_collections()
            .await
            .context("Failed to list Qdrant collections")?;

        let exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.config.collection_name);

        if !exists {
            let distance = match self.config.distance {
                DistanceMetric::Cosine => Distance::Cosine,
                DistanceMetric::Euclidean => Distance::Euclid,
                DistanceMetric::DotProduct => Distance::Dot,
            };

            self.client
                .create_collection(
                    CreateCollectionBuilder::new(&self.config.collection_name).vectors_config(
                        VectorParamsBuilder::new(self.config.dimensions as u64, distance),
                    ),
                )
                .await
                .context("Failed to create Qdrant collection")?;

            tracing::info!(
                "Created Qdrant collection '{}' with {} dimensions",
                self.config.collection_name,
                self.config.dimensions
            );
        }

        Ok(())
    }

    /// Convert our Filter to Qdrant Filter
    fn convert_filter(&self, filter: &Filter) -> QdrantFilter {
        let mut conditions = Vec::new();

        // Exact match conditions from standard fields
        if let Some(ref skill_name) = filter.skill_name {
            conditions.push(Self::make_keyword_condition("skill_name", skill_name));
        }
        if let Some(ref instance_name) = filter.instance_name {
            conditions.push(Self::make_keyword_condition("instance_name", instance_name));
        }
        if let Some(ref tool_name) = filter.tool_name {
            conditions.push(Self::make_keyword_condition("tool_name", tool_name));
        }
        if let Some(ref category) = filter.category {
            conditions.push(Self::make_keyword_condition("category", category));
        }

        // Tag conditions (all must match)
        for tag in &filter.tags {
            conditions.push(Self::make_keyword_condition("tags", tag));
        }

        // Custom field conditions
        for (key, value) in &filter.custom {
            conditions.push(Self::make_keyword_condition(key, value));
        }

        QdrantFilter {
            must: conditions,
            ..Default::default()
        }
    }

    /// Create a keyword match condition
    fn make_keyword_condition(key: &str, value: &str) -> Condition {
        Condition {
            condition_one_of: Some(ConditionOneOf::Field(
                qdrant_client::qdrant::FieldCondition {
                    key: key.to_string(),
                    r#match: Some(qdrant_client::qdrant::Match {
                        match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Keyword(
                            value.to_string(),
                        )),
                    }),
                    ..Default::default()
                },
            )),
        }
    }

    /// Convert metadata to Qdrant payload
    fn metadata_to_payload(metadata: &DocumentMetadata) -> HashMap<String, qdrant_client::qdrant::Value> {
        let mut payload = HashMap::new();

        // Add standard fields
        if let Some(ref skill_name) = metadata.skill_name {
            payload.insert("skill_name".to_string(), Self::string_value(skill_name));
        }
        if let Some(ref instance_name) = metadata.instance_name {
            payload.insert("instance_name".to_string(), Self::string_value(instance_name));
        }
        if let Some(ref tool_name) = metadata.tool_name {
            payload.insert("tool_name".to_string(), Self::string_value(tool_name));
        }
        if let Some(ref category) = metadata.category {
            payload.insert("category".to_string(), Self::string_value(category));
        }

        if !metadata.tags.is_empty() {
            let tags_list: Vec<qdrant_client::qdrant::Value> = metadata
                .tags
                .iter()
                .map(|t| Self::string_value(t))
                .collect();

            payload.insert(
                "tags".to_string(),
                qdrant_client::qdrant::Value {
                    kind: Some(qdrant_client::qdrant::value::Kind::ListValue(
                        qdrant_client::qdrant::ListValue { values: tags_list },
                    )),
                },
            );
        }

        // Add custom fields
        for (key, value) in &metadata.custom {
            if let Ok(json_val) = serde_json::from_str::<JsonValue>(value) {
                payload.insert(key.clone(), json_to_qdrant_value(&json_val));
            } else {
                payload.insert(key.clone(), Self::string_value(value));
            }
        }

        payload
    }

    /// Create a string value for payload
    fn string_value(s: &str) -> qdrant_client::qdrant::Value {
        qdrant_client::qdrant::Value {
            kind: Some(qdrant_client::qdrant::value::Kind::StringValue(s.to_string())),
        }
    }

    /// Convert Qdrant payload back to DocumentMetadata
    fn payload_to_metadata(payload: &HashMap<String, qdrant_client::qdrant::Value>) -> DocumentMetadata {
        let mut metadata = DocumentMetadata::default();

        for (key, value) in payload {
            match key.as_str() {
                "skill_name" => {
                    if let Some(qdrant_client::qdrant::value::Kind::StringValue(s)) = &value.kind {
                        metadata.skill_name = Some(s.clone());
                    }
                }
                "instance_name" => {
                    if let Some(qdrant_client::qdrant::value::Kind::StringValue(s)) = &value.kind {
                        metadata.instance_name = Some(s.clone());
                    }
                }
                "tool_name" => {
                    if let Some(qdrant_client::qdrant::value::Kind::StringValue(s)) = &value.kind {
                        metadata.tool_name = Some(s.clone());
                    }
                }
                "category" => {
                    if let Some(qdrant_client::qdrant::value::Kind::StringValue(s)) = &value.kind {
                        metadata.category = Some(s.clone());
                    }
                }
                "tags" => {
                    if let Some(qdrant_client::qdrant::value::Kind::ListValue(list)) = &value.kind {
                        metadata.tags = list
                            .values
                            .iter()
                            .filter_map(|v| {
                                if let Some(qdrant_client::qdrant::value::Kind::StringValue(s)) = &v.kind {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }
                }
                _ => {
                    // Store in custom as string
                    metadata.custom.insert(key.clone(), qdrant_value_to_string(value));
                }
            }
        }

        metadata
    }

    /// Get the collection name
    pub fn collection_name(&self) -> &str {
        &self.config.collection_name
    }

    /// Get the Qdrant URL
    pub fn url(&self) -> &str {
        &self.config.url
    }
}

/// Convert JSON value to Qdrant value
fn json_to_qdrant_value(json: &JsonValue) -> qdrant_client::qdrant::Value {
    match json {
        JsonValue::Null => qdrant_client::qdrant::Value {
            kind: Some(qdrant_client::qdrant::value::Kind::NullValue(0)),
        },
        JsonValue::Bool(b) => qdrant_client::qdrant::Value {
            kind: Some(qdrant_client::qdrant::value::Kind::BoolValue(*b)),
        },
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                qdrant_client::qdrant::Value {
                    kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)),
                }
            } else if let Some(f) = n.as_f64() {
                qdrant_client::qdrant::Value {
                    kind: Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)),
                }
            } else {
                qdrant_client::qdrant::Value {
                    kind: Some(qdrant_client::qdrant::value::Kind::StringValue(n.to_string())),
                }
            }
        }
        JsonValue::String(s) => qdrant_client::qdrant::Value {
            kind: Some(qdrant_client::qdrant::value::Kind::StringValue(s.clone())),
        },
        JsonValue::Array(arr) => {
            let values: Vec<qdrant_client::qdrant::Value> = arr.iter().map(json_to_qdrant_value).collect();
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::ListValue(
                    qdrant_client::qdrant::ListValue { values },
                )),
            }
        }
        JsonValue::Object(obj) => {
            let fields: HashMap<String, qdrant_client::qdrant::Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_qdrant_value(v)))
                .collect();
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StructValue(
                    qdrant_client::qdrant::Struct { fields },
                )),
            }
        }
    }
}

/// Convert Qdrant value to string representation
fn qdrant_value_to_string(value: &qdrant_client::qdrant::Value) -> String {
    match &value.kind {
        Some(qdrant_client::qdrant::value::Kind::StringValue(s)) => s.clone(),
        Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)) => i.to_string(),
        Some(qdrant_client::qdrant::value::Kind::DoubleValue(d)) => d.to_string(),
        Some(qdrant_client::qdrant::value::Kind::BoolValue(b)) => b.to_string(),
        _ => String::new(),
    }
}

#[async_trait]
impl VectorStore for QdrantVectorStore {
    async fn upsert(&self, documents: Vec<EmbeddedDocument>) -> Result<UpsertStats> {
        let start = Instant::now();

        if documents.is_empty() {
            return Ok(UpsertStats::default());
        }

        let points: Vec<PointStruct> = documents
            .iter()
            .map(|doc| {
                let payload = Self::metadata_to_payload(&doc.metadata);
                PointStruct::new(doc.id.clone(), doc.embedding.clone(), payload)
            })
            .collect();

        let count = points.len();

        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.config.collection_name, points).wait(true))
            .await
            .context("Failed to upsert points to Qdrant")?;

        Ok(UpsertStats::new(count, 0, start.elapsed().as_millis() as u64))
    }

    async fn search(
        &self,
        query_embedding: Vec<f32>,
        filter: Option<Filter>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut request =
            SearchPointsBuilder::new(&self.config.collection_name, query_embedding, top_k as u64)
                .with_payload(true);

        if let Some(ref f) = filter {
            if !f.is_empty() {
                let qdrant_filter = self.convert_filter(f);
                request = request.filter(qdrant_filter);
            }
        }

        let results = self
            .client
            .search_points(request)
            .await
            .context("Failed to search Qdrant")?;

        let search_results: Vec<SearchResult> = results
            .result
            .into_iter()
            .filter_map(|point| {
                let id = match point.id? {
                    PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid)) } => uuid,
                    PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) } => num.to_string(),
                    _ => return None,
                };

                let metadata = Self::payload_to_metadata(&point.payload);

                Some(SearchResult::new(id, point.score, metadata))
            })
            .collect();

        Ok(search_results)
    }

    async fn delete(&self, ids: Vec<String>) -> Result<DeleteStats> {
        use qdrant_client::qdrant::DeletePointsBuilder;

        let start = Instant::now();

        if ids.is_empty() {
            return Ok(DeleteStats::default());
        }

        let count = ids.len();
        let point_ids: Vec<PointId> = ids
            .into_iter()
            .map(|id| PointId {
                point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(id)),
            })
            .collect();

        self.client
            .delete_points(
                DeletePointsBuilder::new(&self.config.collection_name)
                    .points(PointsSelectorOneOf::Points(PointsIdsList {
                        ids: point_ids,
                    }))
                    .wait(true),
            )
            .await
            .context("Failed to delete points from Qdrant")?;

        Ok(DeleteStats::new(count, 0, start.elapsed().as_millis() as u64))
    }

    async fn get(&self, ids: Vec<String>) -> Result<Vec<EmbeddedDocument>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let point_ids: Vec<PointId> = ids
            .into_iter()
            .map(|id| PointId {
                point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(id)),
            })
            .collect();

        let response = self
            .client
            .get_points(
                GetPointsBuilder::new(&self.config.collection_name, point_ids)
                    .with_payload(true)
                    .with_vectors(true),
            )
            .await
            .context("Failed to get points from Qdrant")?;

        let documents: Vec<EmbeddedDocument> = response
            .result
            .into_iter()
            .filter_map(|point| {
                let id = match point.id? {
                    PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid)) } => uuid,
                    PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) } => num.to_string(),
                    _ => return None,
                };

                // Extract vector from VectorsOutput
                let embedding = point.vectors.and_then(|v| {
                    match v.vectors_options? {
                        qdrant_client::qdrant::vectors_output::VectorsOptions::Vector(vec) => Some(vec.data),
                        _ => None,
                    }
                })?;

                let metadata = Self::payload_to_metadata(&point.payload);

                Some(EmbeddedDocument {
                    id,
                    embedding,
                    metadata,
                    content: None,
                })
            })
            .collect();

        Ok(documents)
    }

    async fn count(&self, _filter: Option<Filter>) -> Result<usize> {
        // Get collection info for total count
        let info = self
            .client
            .collection_info(&self.config.collection_name)
            .await
            .context("Failed to get collection info")?;

        Ok(info.result.map(|r| r.points_count.unwrap_or(0) as usize).unwrap_or(0))
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let start = Instant::now();

        match self.client.health_check().await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;

                // Get collection info for additional details
                let document_count = self
                    .client
                    .collection_info(&self.config.collection_name)
                    .await
                    .ok()
                    .and_then(|i| i.result)
                    .and_then(|r| r.points_count)
                    .map(|c| c as usize);

                let mut status = HealthStatus::healthy(self.backend_name(), latency);
                if let Some(count) = document_count {
                    status = status.with_document_count(count);
                }
                Ok(status)
            }
            Err(e) => {
                let latency = start.elapsed().as_millis() as u64;
                Ok(HealthStatus::unhealthy(
                    self.backend_name(),
                    format!("Qdrant health check failed: {}", e),
                    latency,
                ))
            }
        }
    }

    fn backend_name(&self) -> &'static str {
        "qdrant"
    }

    fn dimensions(&self) -> Option<usize> {
        Some(self.config.dimensions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = QdrantConfig::default();
        assert_eq!(config.url, DEFAULT_QDRANT_URL);
        assert_eq!(config.collection_name, DEFAULT_COLLECTION_NAME);
        assert_eq!(config.dimensions, 384);
    }

    #[test]
    fn test_config_cloud() {
        let config = QdrantConfig::cloud("https://cloud.qdrant.io", "api-key-123");
        assert_eq!(config.url, "https://cloud.qdrant.io");
        assert_eq!(config.api_key, Some("api-key-123".to_string()));
    }

    #[test]
    fn test_config_builder() {
        let config = QdrantConfig::default()
            .with_dimensions(1536)
            .with_collection("custom-collection")
            .with_distance(DistanceMetric::Euclidean);

        assert_eq!(config.dimensions, 1536);
        assert_eq!(config.collection_name, "custom-collection");
        assert_eq!(config.distance, DistanceMetric::Euclidean);
    }

    #[test]
    fn test_json_to_qdrant_value() {
        let json = serde_json::json!({
            "string": "hello",
            "number": 42,
            "bool": true
        });

        if let JsonValue::Object(obj) = json {
            for (_, value) in obj {
                let _ = json_to_qdrant_value(&value);
            }
        }
    }

    #[test]
    fn test_metadata_to_payload() {
        let metadata = DocumentMetadata {
            skill_name: Some("test-skill".to_string()),
            instance_name: Some("default".to_string()),
            tool_name: Some("hello".to_string()),
            category: Some("testing".to_string()),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            custom: HashMap::new(),
        };

        let payload = QdrantVectorStore::metadata_to_payload(&metadata);
        assert!(payload.contains_key("skill_name"));
        assert!(payload.contains_key("tags"));
    }

    // Integration tests require a running Qdrant instance
    #[tokio::test]
    #[ignore = "requires running Qdrant server"]
    async fn test_qdrant_operations() {
        let store = QdrantVectorStore::new().await.unwrap();

        // Test upsert
        let doc = EmbeddedDocument::new("test-doc-1", vec![0.1; 384])
            .with_skill_name("test-skill")
            .with_tool_name("test-tool");

        let stats = store.upsert(vec![doc]).await.unwrap();
        assert_eq!(stats.inserted, 1);

        // Test search
        let results = store.search(vec![0.1; 384], None, 5).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "test-doc-1");

        // Test delete
        let delete_stats = store.delete(vec!["test-doc-1".to_string()]).await.unwrap();
        assert_eq!(delete_stats.deleted, 1);
    }
}
