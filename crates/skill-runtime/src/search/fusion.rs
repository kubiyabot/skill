//! Fusion algorithms for combining search results
//!
//! Implements Reciprocal Rank Fusion (RRF) and weighted sum fusion
//! for combining results from multiple retrieval systems.

use std::collections::HashMap;

/// Result from fusion containing the ID and combined score
#[derive(Debug, Clone)]
pub struct FusedResult {
    /// Document ID
    pub id: String,
    /// Combined score after fusion
    pub score: f32,
    /// Source scores for debugging/analysis
    pub source_scores: HashMap<String, f32>,
}

/// Fusion method to combine search results
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum FusionMethod {
    /// Reciprocal Rank Fusion (RRF) - default
    #[default]
    ReciprocalRank,
    /// Weighted sum of normalized scores
    WeightedSum,
    /// Take maximum score from any source
    MaxScore,
}

/// Reciprocal Rank Fusion (RRF)
///
/// Combines ranked lists using the formula:
/// score(d) = Σ(1 / (k + rank_i))
///
/// where k is typically 60 (per original paper)
///
/// # Arguments
/// * `ranked_lists` - List of (source_name, rankings) where rankings are (id, original_score)
/// * `k` - RRF constant, default 60
/// * `top_k` - Number of results to return
///
/// # Returns
/// Fused results sorted by combined score descending
pub fn reciprocal_rank_fusion(
    ranked_lists: Vec<(&str, Vec<(String, f32)>)>,
    k: f32,
    top_k: usize,
) -> Vec<FusedResult> {
    let mut scores: HashMap<String, (f32, HashMap<String, f32>)> = HashMap::new();

    for (source_name, rankings) in ranked_lists {
        for (rank, (id, original_score)) in rankings.into_iter().enumerate() {
            let rrf_score = 1.0 / (k + (rank + 1) as f32);

            let entry = scores.entry(id).or_insert_with(|| (0.0, HashMap::new()));
            entry.0 += rrf_score;
            entry.1.insert(source_name.to_string(), original_score);
        }
    }

    let mut results: Vec<FusedResult> = scores
        .into_iter()
        .map(|(id, (score, source_scores))| FusedResult {
            id,
            score,
            source_scores,
        })
        .collect();

    // Sort by score descending
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    results.truncate(top_k);
    results
}

/// Weighted sum fusion
///
/// Normalizes scores from each source to [0,1] and combines with weights
///
/// # Arguments
/// * `weighted_lists` - List of (source_name, weight, rankings) where rankings are (id, original_score)
/// * `top_k` - Number of results to return
///
/// # Returns
/// Fused results sorted by combined score descending
pub fn weighted_sum_fusion(
    weighted_lists: Vec<(&str, f32, Vec<(String, f32)>)>,
    top_k: usize,
) -> Vec<FusedResult> {
    let mut scores: HashMap<String, (f32, HashMap<String, f32>)> = HashMap::new();

    for (source_name, weight, rankings) in weighted_lists {
        // Find min/max for normalization
        let (min_score, max_score) = rankings.iter().fold((f32::MAX, f32::MIN), |(min, max), (_, s)| {
            (min.min(*s), max.max(*s))
        });

        let range = max_score - min_score;

        for (id, original_score) in rankings {
            // Normalize to [0, 1]
            let normalized = if range > 0.0 {
                (original_score - min_score) / range
            } else {
                1.0 // All scores are the same
            };

            let weighted_score = normalized * weight;

            let entry = scores.entry(id).or_insert_with(|| (0.0, HashMap::new()));
            entry.0 += weighted_score;
            entry.1.insert(source_name.to_string(), original_score);
        }
    }

    let mut results: Vec<FusedResult> = scores
        .into_iter()
        .map(|(id, (score, source_scores))| FusedResult {
            id,
            score,
            source_scores,
        })
        .collect();

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    results.truncate(top_k);
    results
}

/// Max score fusion
///
/// Takes the maximum score from any source for each document
#[allow(dead_code)]
pub fn max_score_fusion(
    ranked_lists: Vec<(&str, Vec<(String, f32)>)>,
    top_k: usize,
) -> Vec<FusedResult> {
    let mut scores: HashMap<String, (f32, HashMap<String, f32>)> = HashMap::new();

    for (source_name, rankings) in ranked_lists {
        for (id, original_score) in rankings {
            let entry = scores.entry(id).or_insert_with(|| (f32::MIN, HashMap::new()));
            if original_score > entry.0 {
                entry.0 = original_score;
            }
            entry.1.insert(source_name.to_string(), original_score);
        }
    }

    let mut results: Vec<FusedResult> = scores
        .into_iter()
        .map(|(id, (score, source_scores))| FusedResult {
            id,
            score,
            source_scores,
        })
        .collect();

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    results.truncate(top_k);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_fusion_basic() {
        let dense_results = vec![
            ("doc1".to_string(), 0.95),
            ("doc2".to_string(), 0.85),
            ("doc3".to_string(), 0.75),
        ];

        let sparse_results = vec![
            ("doc2".to_string(), 10.5), // doc2 ranks higher in BM25
            ("doc1".to_string(), 8.3),
            ("doc4".to_string(), 7.1),
        ];

        let results = reciprocal_rank_fusion(
            vec![("dense", dense_results), ("sparse", sparse_results)],
            60.0,
            5,
        );

        // doc1 and doc2 should have the highest scores (appear in both lists)
        assert!(results.len() <= 5);

        // Both doc1 and doc2 should be in results
        let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"doc1"));
        assert!(ids.contains(&"doc2"));
    }

    #[test]
    fn test_rrf_k_parameter() {
        let list = vec![
            ("doc1".to_string(), 1.0),
            ("doc2".to_string(), 0.9),
        ];

        // With k=60, rank 1 gets score 1/(60+1) = 0.0164
        let results = reciprocal_rank_fusion(vec![("test", list.clone())], 60.0, 5);
        assert!((results[0].score - 1.0 / 61.0).abs() < 0.001);
        assert!((results[1].score - 1.0 / 62.0).abs() < 0.001);
    }

    #[test]
    fn test_weighted_sum_fusion() {
        let dense_results = vec![
            ("doc1".to_string(), 0.9),
            ("doc2".to_string(), 0.7),
        ];

        let sparse_results = vec![
            ("doc1".to_string(), 5.0),
            ("doc2".to_string(), 10.0), // Higher sparse score
        ];

        let results = weighted_sum_fusion(
            vec![("dense", 0.7, dense_results), ("sparse", 0.3, sparse_results)],
            5,
        );

        assert!(!results.is_empty());
        // Both should appear
        let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"doc1"));
        assert!(ids.contains(&"doc2"));
    }

    #[test]
    fn test_fusion_with_empty_list() {
        let results = reciprocal_rank_fusion(vec![], 60.0, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_fusion_source_scores_preserved() {
        let dense_results = vec![("doc1".to_string(), 0.95)];
        let sparse_results = vec![("doc1".to_string(), 8.5)];

        let results = reciprocal_rank_fusion(
            vec![("dense", dense_results), ("sparse", sparse_results)],
            60.0,
            5,
        );

        assert_eq!(results[0].id, "doc1");
        assert_eq!(results[0].source_scores.get("dense"), Some(&0.95));
        assert_eq!(results[0].source_scores.get("sparse"), Some(&8.5));
    }

    #[test]
    fn test_max_score_fusion() {
        let list1 = vec![
            ("doc1".to_string(), 0.5),
            ("doc2".to_string(), 0.8),
        ];

        let list2 = vec![
            ("doc1".to_string(), 0.9), // Higher
            ("doc2".to_string(), 0.3),
        ];

        let results = max_score_fusion(vec![("a", list1), ("b", list2)], 5);

        // doc1 should have score 0.9 (max), doc2 should have 0.8
        let doc1 = results.iter().find(|r| r.id == "doc1").unwrap();
        let doc2 = results.iter().find(|r| r.id == "doc2").unwrap();
        assert!((doc1.score - 0.9).abs() < 0.001);
        assert!((doc2.score - 0.8).abs() < 0.001);
    }
}
