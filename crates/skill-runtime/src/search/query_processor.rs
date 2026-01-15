//! Query understanding with intent classification and entity extraction
//!
//! Provides intelligent query preprocessing to improve search relevance
//! through rule-based intent detection, entity recognition, and query expansion.

use std::collections::{HashMap, HashSet};

/// Query intent classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryIntent {
    /// User wants to discover what tools can do something
    /// E.g., "what tools can list files", "tools for kubernetes"
    ToolDiscovery,
    /// User wants to execute a specific tool
    /// E.g., "run list_pods", "execute get deployment"
    ToolExecution,
    /// User wants documentation on how a tool works
    /// E.g., "how does list_pods work", "explain kubernetes tool"
    ToolDocumentation,
    /// User wants to compare tools
    /// E.g., "difference between X and Y", "X vs Y"
    Comparison,
    /// User is troubleshooting
    /// E.g., "why is X failing", "error with X"
    Troubleshooting,
    /// General query - no specific intent detected
    General,
}

impl QueryIntent {
    /// Get confidence threshold for this intent
    pub fn confidence_threshold(&self) -> f32 {
        match self {
            QueryIntent::ToolExecution => 0.8,
            QueryIntent::Comparison => 0.7,
            QueryIntent::Troubleshooting => 0.7,
            QueryIntent::ToolDocumentation => 0.6,
            QueryIntent::ToolDiscovery => 0.5,
            QueryIntent::General => 0.0,
        }
    }
}

/// Entity type for extracted entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    /// Skill name (e.g., "kubernetes", "github")
    SkillName,
    /// Tool name (e.g., "list_pods", "create_issue")
    ToolName,
    /// Action verb (e.g., "create", "delete", "list")
    ActionVerb,
    /// Category (e.g., "database", "cloud", "git")
    Category,
    /// Target/object (e.g., "pods", "files", "users")
    Target,
}

/// Extracted entity from query
#[derive(Debug, Clone)]
pub struct ExtractedEntity {
    /// The entity text
    pub text: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Position in the original query
    pub position: usize,
}

/// Query expansion with synonyms and related terms
#[derive(Debug, Clone)]
pub struct QueryExpansion {
    /// Original term
    pub original: String,
    /// Expanded terms (synonyms, related)
    pub expanded: Vec<String>,
    /// Expansion type
    pub expansion_type: ExpansionType,
}

/// Type of query expansion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpansionType {
    /// Synonym expansion (create -> make, generate)
    Synonym,
    /// Acronym expansion (k8s -> kubernetes)
    Acronym,
    /// Pattern expansion (get pods -> list pods)
    Pattern,
}

/// Processed query with intent, entities, and expansions
#[derive(Debug, Clone)]
pub struct ProcessedQuery {
    /// Original query string
    pub original: String,
    /// Normalized/cleaned query
    pub normalized: String,
    /// Detected intent
    pub intent: QueryIntent,
    /// Intent confidence score
    pub intent_confidence: f32,
    /// Extracted entities
    pub entities: Vec<ExtractedEntity>,
    /// Query expansions
    pub expansions: Vec<QueryExpansion>,
    /// Suggested search filters
    pub suggested_filters: Vec<SuggestedFilter>,
}

/// Suggested filter for search
#[derive(Debug, Clone)]
pub struct SuggestedFilter {
    /// Filter field
    pub field: String,
    /// Filter value
    pub value: String,
    /// Confidence
    pub confidence: f32,
}

/// Query processor for intelligent search preprocessing
pub struct QueryProcessor {
    /// Known skill names for entity extraction
    known_skills: HashSet<String>,
    /// Known tool names for entity extraction
    known_tools: HashSet<String>,
    /// Synonyms for query expansion
    synonyms: HashMap<String, Vec<String>>,
    /// Acronyms for expansion
    acronyms: HashMap<String, String>,
    /// Action verb patterns
    action_verbs: HashSet<String>,
    /// Category keywords
    categories: HashMap<String, Vec<String>>,
}

impl Default for QueryProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryProcessor {
    /// Create a new query processor with default knowledge
    pub fn new() -> Self {
        let mut processor = Self {
            known_skills: HashSet::new(),
            known_tools: HashSet::new(),
            synonyms: HashMap::new(),
            acronyms: HashMap::new(),
            action_verbs: HashSet::new(),
            categories: HashMap::new(),
        };

        // Initialize with common knowledge
        processor.init_action_verbs();
        processor.init_synonyms();
        processor.init_acronyms();
        processor.init_categories();

        processor
    }

    /// Add known skills for entity extraction
    pub fn with_skills(mut self, skills: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for skill in skills {
            self.known_skills.insert(skill.into().to_lowercase());
        }
        self
    }

    /// Add known tools for entity extraction
    pub fn with_tools(mut self, tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for tool in tools {
            self.known_tools.insert(tool.into().to_lowercase());
        }
        self
    }

    /// Process a query for improved search
    pub fn process(&self, query: &str) -> ProcessedQuery {
        let normalized = self.normalize_query(query);
        let tokens = self.tokenize(&normalized);

        // Classify intent
        let (intent, intent_confidence) = self.classify_intent(&normalized, &tokens);

        // Extract entities
        let entities = self.extract_entities(&normalized, &tokens);

        // Generate expansions
        let expansions = self.generate_expansions(&tokens);

        // Suggest filters based on entities
        let suggested_filters = self.suggest_filters(&entities);

        ProcessedQuery {
            original: query.to_string(),
            normalized,
            intent,
            intent_confidence,
            entities,
            expansions,
            suggested_filters,
        }
    }

    /// Get expanded query terms for search
    pub fn get_expanded_terms(&self, query: &ProcessedQuery) -> Vec<String> {
        let mut terms = vec![query.normalized.clone()];

        // Add expansion terms
        for expansion in &query.expansions {
            for term in &expansion.expanded {
                if !terms.contains(term) {
                    terms.push(term.clone());
                }
            }
        }

        // Add entity text with higher weight potential
        for entity in &query.entities {
            if !terms.contains(&entity.text) {
                terms.push(entity.text.clone());
            }
        }

        terms
    }

    // --- Internal methods ---

    fn init_action_verbs(&mut self) {
        let verbs = [
            "create", "make", "add", "new", "generate",
            "delete", "remove", "destroy", "drop",
            "list", "get", "fetch", "retrieve", "show", "display",
            "update", "modify", "change", "edit", "patch",
            "read", "view", "inspect", "describe",
            "run", "execute", "invoke", "call", "start",
            "stop", "kill", "terminate", "cancel",
            "deploy", "install", "setup", "configure",
            "search", "find", "query", "filter",
            "connect", "disconnect", "link",
            "send", "receive", "push", "pull",
            "upload", "download", "sync",
            "validate", "verify", "check", "test",
        ];
        self.action_verbs = verbs.iter().map(|s| s.to_string()).collect();
    }

    fn init_synonyms(&mut self) {
        let synonym_map = [
            ("create", vec!["make", "generate", "add", "new", "build"]),
            ("delete", vec!["remove", "destroy", "drop", "erase"]),
            ("list", vec!["get", "show", "display", "fetch", "retrieve"]),
            ("update", vec!["modify", "change", "edit", "patch", "alter"]),
            ("run", vec!["execute", "invoke", "call", "start", "launch"]),
            ("find", vec!["search", "query", "lookup", "locate"]),
            ("stop", vec!["kill", "terminate", "cancel", "halt"]),
            ("deploy", vec!["install", "setup", "release", "publish"]),
            ("file", vec!["document", "artifact"]),
            ("folder", vec!["directory", "dir"]),
            ("container", vec!["pod", "instance"]),
        ];

        for (key, synonyms) in synonym_map {
            self.synonyms.insert(key.to_string(), synonyms.iter().map(|s| s.to_string()).collect());
        }
    }

    fn init_acronyms(&mut self) {
        let acronym_map = [
            ("k8s", "kubernetes"),
            ("gh", "github"),
            ("gl", "gitlab"),
            ("db", "database"),
            ("aws", "amazon web services"),
            ("gcp", "google cloud platform"),
            ("az", "azure"),
            ("tf", "terraform"),
            ("ci", "continuous integration"),
            ("cd", "continuous deployment"),
            ("api", "application programming interface"),
            ("cli", "command line interface"),
            ("env", "environment"),
            ("vars", "variables"),
            ("config", "configuration"),
            ("auth", "authentication"),
            ("repo", "repository"),
        ];

        for (acronym, expanded) in acronym_map {
            self.acronyms.insert(acronym.to_string(), expanded.to_string());
        }
    }

    fn init_categories(&mut self) {
        let category_map = [
            ("kubernetes", vec!["pod", "deployment", "service", "namespace", "ingress", "configmap", "secret", "node", "cluster"]),
            ("git", vec!["commit", "branch", "merge", "pull", "push", "clone", "checkout", "repository", "repo"]),
            ("database", vec!["query", "table", "schema", "index", "migration", "backup", "restore"]),
            ("cloud", vec!["instance", "bucket", "function", "lambda", "storage", "network", "vpc"]),
            ("docker", vec!["container", "image", "volume", "network", "compose"]),
            ("file", vec!["read", "write", "copy", "move", "delete", "list", "directory"]),
        ];

        for (category, keywords) in category_map {
            self.categories.insert(category.to_string(), keywords.iter().map(|s| s.to_string()).collect());
        }
    }

    fn normalize_query(&self, query: &str) -> String {
        let mut normalized = query.to_lowercase();

        // Expand acronyms
        for (acronym, expanded) in &self.acronyms {
            if normalized.contains(acronym) {
                // Only expand if it's a whole word
                let pattern = format!(r"\b{}\b", acronym);
                if let Ok(re) = regex_lite::Regex::new(&pattern) {
                    normalized = re.replace_all(&normalized, expanded.as_str()).to_string();
                }
            }
        }

        // Remove extra whitespace
        normalized.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|s| s.trim_matches(|c: char| c.is_ascii_punctuation()).to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn classify_intent(&self, query: &str, _tokens: &[String]) -> (QueryIntent, f32) {
        let query_lower = query.to_lowercase();

        // Check for execution intent (highest priority)
        let execution_patterns = ["run ", "execute ", "invoke ", "call "];
        for pattern in execution_patterns {
            if query_lower.starts_with(pattern) {
                return (QueryIntent::ToolExecution, 0.9);
            }
        }

        // Check for comparison intent
        if query_lower.contains(" vs ") ||
           query_lower.contains(" versus ") ||
           query_lower.contains("compare ") ||
           query_lower.contains("difference between") {
            return (QueryIntent::Comparison, 0.85);
        }

        // Check for troubleshooting intent
        let trouble_patterns = ["why ", "error", "fail", "not working", "issue", "problem", "debug"];
        for pattern in trouble_patterns {
            if query_lower.contains(pattern) {
                return (QueryIntent::Troubleshooting, 0.8);
            }
        }

        // Check for documentation intent
        let doc_patterns = ["how does", "how to", "what is", "explain", "documentation", "help with"];
        for pattern in doc_patterns {
            if query_lower.contains(pattern) {
                return (QueryIntent::ToolDocumentation, 0.75);
            }
        }

        // Check for discovery intent
        let discovery_patterns = ["what tools", "tools for", "which tool", "find tool", "available"];
        for pattern in discovery_patterns {
            if query_lower.contains(pattern) {
                return (QueryIntent::ToolDiscovery, 0.7);
            }
        }

        // Default to general with lower confidence
        (QueryIntent::General, 0.5)
    }

    fn extract_entities(&self, _query: &str, tokens: &[String]) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();

        for (pos, token) in tokens.iter().enumerate() {
            let token_lower = token.to_lowercase();

            // Check for known skills
            if self.known_skills.contains(&token_lower) {
                entities.push(ExtractedEntity {
                    text: token.clone(),
                    entity_type: EntityType::SkillName,
                    confidence: 0.95,
                    position: pos,
                });
                continue;
            }

            // Check for known tools
            if self.known_tools.contains(&token_lower) {
                entities.push(ExtractedEntity {
                    text: token.clone(),
                    entity_type: EntityType::ToolName,
                    confidence: 0.95,
                    position: pos,
                });
                continue;
            }

            // Check for action verbs
            if self.action_verbs.contains(&token_lower) {
                entities.push(ExtractedEntity {
                    text: token.clone(),
                    entity_type: EntityType::ActionVerb,
                    confidence: 0.85,
                    position: pos,
                });
                continue;
            }

            // Check for category matches
            for (category, keywords) in &self.categories {
                if keywords.iter().any(|k| token_lower.contains(k) || k.contains(&token_lower)) {
                    entities.push(ExtractedEntity {
                        text: category.clone(),
                        entity_type: EntityType::Category,
                        confidence: 0.75,
                        position: pos,
                    });
                    break;
                }
            }
        }

        // Deduplicate entities by (text, type) pair
        let mut seen = HashSet::new();
        entities.retain(|e| seen.insert((e.text.clone(), e.entity_type)));

        entities
    }

    fn generate_expansions(&self, tokens: &[String]) -> Vec<QueryExpansion> {
        let mut expansions = Vec::new();

        for token in tokens {
            let token_lower = token.to_lowercase();

            // Check for synonym expansion
            if let Some(synonyms) = self.synonyms.get(&token_lower) {
                expansions.push(QueryExpansion {
                    original: token.clone(),
                    expanded: synonyms.clone(),
                    expansion_type: ExpansionType::Synonym,
                });
            }

            // Note: Acronym expansion is handled in normalize_query
        }

        expansions
    }

    fn suggest_filters(&self, entities: &[ExtractedEntity]) -> Vec<SuggestedFilter> {
        let mut filters = Vec::new();

        for entity in entities {
            match entity.entity_type {
                EntityType::SkillName => {
                    filters.push(SuggestedFilter {
                        field: "skill_name".to_string(),
                        value: entity.text.clone(),
                        confidence: entity.confidence,
                    });
                }
                EntityType::Category => {
                    filters.push(SuggestedFilter {
                        field: "category".to_string(),
                        value: entity.text.clone(),
                        confidence: entity.confidence,
                    });
                }
                _ => {}
            }
        }

        filters
    }
}

// Note: Using regex-lite instead of full regex for lighter dependency
mod regex_lite {
    pub struct Regex(String);

    impl Regex {
        pub fn new(pattern: &str) -> Result<Self, ()> {
            Ok(Regex(pattern.to_string()))
        }

        pub fn replace_all<'a>(&self, text: &'a str, replacement: &str) -> std::borrow::Cow<'a, str> {
            // Simple word boundary replacement
            let word = self.0.trim_start_matches(r"\b").trim_end_matches(r"\b");
            let words: Vec<&str> = text.split_whitespace().collect();
            let replaced: Vec<&str> = words.iter()
                .map(|w| if w.to_lowercase() == word { replacement } else { *w })
                .collect();
            std::borrow::Cow::Owned(replaced.join(" "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_classification_execution() {
        let processor = QueryProcessor::new();

        let query = processor.process("run list_pods");
        assert_eq!(query.intent, QueryIntent::ToolExecution);
        assert!(query.intent_confidence > 0.8);

        let query = processor.process("execute get deployment");
        assert_eq!(query.intent, QueryIntent::ToolExecution);
    }

    #[test]
    fn test_intent_classification_comparison() {
        let processor = QueryProcessor::new();

        let query = processor.process("kubernetes vs docker");
        assert_eq!(query.intent, QueryIntent::Comparison);

        let query = processor.process("difference between list and get");
        assert_eq!(query.intent, QueryIntent::Comparison);
    }

    #[test]
    fn test_intent_classification_troubleshooting() {
        let processor = QueryProcessor::new();

        let query = processor.process("why is the pod failing");
        assert_eq!(query.intent, QueryIntent::Troubleshooting);

        let query = processor.process("error connecting to database");
        assert_eq!(query.intent, QueryIntent::Troubleshooting);
    }

    #[test]
    fn test_intent_classification_documentation() {
        let processor = QueryProcessor::new();

        let query = processor.process("how does list_pods work");
        assert_eq!(query.intent, QueryIntent::ToolDocumentation);

        let query = processor.process("explain kubernetes deployment");
        assert_eq!(query.intent, QueryIntent::ToolDocumentation);
    }

    #[test]
    fn test_entity_extraction_with_known_skills() {
        let processor = QueryProcessor::new()
            .with_skills(["kubernetes", "github", "docker"]);

        let query = processor.process("list pods in kubernetes");
        let skill_entities: Vec<_> = query.entities.iter()
            .filter(|e| e.entity_type == EntityType::SkillName)
            .collect();

        assert_eq!(skill_entities.len(), 1);
        assert_eq!(skill_entities[0].text, "kubernetes");
    }

    #[test]
    fn test_entity_extraction_action_verbs() {
        let processor = QueryProcessor::new();

        let query = processor.process("create a new deployment");
        let verb_entities: Vec<_> = query.entities.iter()
            .filter(|e| e.entity_type == EntityType::ActionVerb)
            .collect();

        assert!(verb_entities.iter().any(|e| e.text == "create"));
    }

    #[test]
    fn test_query_expansion_synonyms() {
        let processor = QueryProcessor::new();

        let query = processor.process("create pod");
        let create_expansion = query.expansions.iter()
            .find(|e| e.original.to_lowercase() == "create");

        assert!(create_expansion.is_some());
        let expansion = create_expansion.unwrap();
        assert!(expansion.expanded.contains(&"make".to_string()));
        assert!(expansion.expanded.contains(&"generate".to_string()));
    }

    #[test]
    fn test_acronym_expansion() {
        let processor = QueryProcessor::new();

        let query = processor.process("list pods in k8s");
        assert!(query.normalized.contains("kubernetes"));
    }

    #[test]
    fn test_category_detection() {
        let processor = QueryProcessor::new();

        let query = processor.process("get deployment information");
        let category_entities: Vec<_> = query.entities.iter()
            .filter(|e| e.entity_type == EntityType::Category)
            .collect();

        // "deployment" should trigger kubernetes category
        assert!(category_entities.iter().any(|e| e.text == "kubernetes"));
    }

    #[test]
    fn test_suggested_filters() {
        let processor = QueryProcessor::new()
            .with_skills(["kubernetes"]);

        let query = processor.process("kubernetes pod list");
        let skill_filters: Vec<_> = query.suggested_filters.iter()
            .filter(|f| f.field == "skill_name")
            .collect();

        assert_eq!(skill_filters.len(), 1);
        assert_eq!(skill_filters[0].value, "kubernetes");
    }

    #[test]
    fn test_get_expanded_terms() {
        let processor = QueryProcessor::new();

        let query = processor.process("create deployment");
        let terms = processor.get_expanded_terms(&query);

        // Should include original and expansions
        assert!(terms.iter().any(|t| t.contains("create") || t.contains("deployment")));
        assert!(terms.len() > 1); // Should have expansions
    }

    #[test]
    fn test_normalize_query() {
        let processor = QueryProcessor::new();

        // Test whitespace normalization
        let query = processor.process("  list    pods  ");
        assert_eq!(query.normalized, "list pods");
    }
}
