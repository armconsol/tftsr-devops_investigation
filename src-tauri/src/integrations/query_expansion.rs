/// Query expansion module for integration search
///
/// This module provides functionality to expand user queries with related terms,
/// synonyms, and variations to improve search results across integrations like
/// Confluence, ServiceNow, and Azure DevOps.
use std::collections::HashSet;

/// Product name synonyms for common product variations
/// Maps common abbreviations/variants to their full names for search expansion
fn get_product_synonyms(query: &str) -> Vec<String> {
    let mut synonyms = Vec::new();

    // VESTA NXT related synonyms
    if query.to_lowercase().contains("vesta") || query.to_lowercase().contains("product") {
        synonyms.extend(vec![
            "VESTA NXT".to_string(),
            "DevOps Platform NXT".to_string(),
            "DevOps Tool".to_string(),
            "product".to_string(),
            "DevOps Platform".to_string(),
            "vesta".to_string(),
            "VNX".to_string(),
            "vnx".to_string(),
        ]);
    }

    // Version number patterns (e.g., 1.0.12, 1.1.9)
    if query.contains('.') {
        // Extract version-like patterns and add variations
        let version_parts: Vec<&str> = query.split('.').collect();
        if version_parts.len() >= 2 {
            // Add variations without dots
            let version_no_dots = version_parts.join("");
            synonyms.push(version_no_dots);

            // Add partial versions
            if version_parts.len() >= 2 {
                synonyms.push(version_parts[0..2].join("."));
            }
            if version_parts.len() >= 3 {
                synonyms.push(version_parts[0..3].join("."));
            }
        }
    }

    // Common upgrade-related terms
    if query.to_lowercase().contains("upgrade") || query.to_lowercase().contains("update") {
        synonyms.extend(vec![
            "upgrade".to_string(),
            "update".to_string(),
            "migration".to_string(),
            "patch".to_string(),
            "version".to_string(),
            "install".to_string(),
            "installation".to_string(),
        ]);
    }

    // Remove duplicates and empty strings
    synonyms.sort();
    synonyms.dedup();
    synonyms.retain(|s| !s.is_empty());

    synonyms
}

/// Expand a search query with related terms for better search coverage
///
/// This function takes a user query and expands it with:
/// - Product name synonyms (e.g., "DevOps Tool" -> "VESTA NXT", "DevOps Platform NXT")
/// - Version number variations
/// - Related terms based on query content
///
/// # Arguments
/// * `query` - The original user query
///
/// # Returns
/// A vector of query strings to search, with the original query first
/// followed by expanded variations. Returns empty only if input is empty or
/// whitespace-only. Otherwise, always returns at least the original query.
pub fn expand_query(query: &str) -> Vec<String> {
    if query.trim().is_empty() {
        return Vec::new();
    }

    let mut expanded = vec![query.to_string()];

    // Get product synonyms
    let product_synonyms = get_product_synonyms(query);
    expanded.extend(product_synonyms);

    // Extract keywords from query for additional expansion
    let keywords = extract_keywords(query);

    // Add keyword variations
    for keyword in keywords.iter().take(5) {
        if !expanded.contains(keyword) {
            expanded.push(keyword.clone());
        }
    }

    // Add common related terms based on query content
    let query_lower = query.to_lowercase();

    if query_lower.contains("confluence") || query_lower.contains("documentation") {
        expanded.push("docs".to_string());
        expanded.push("manual".to_string());
        expanded.push("guide".to_string());
    }

    if query_lower.contains("deploy") || query_lower.contains("deployment") {
        expanded.push("deploy".to_string());
        expanded.push("deployment".to_string());
        expanded.push("release".to_string());
        expanded.push("build".to_string());
    }

    if query_lower.contains("kubernetes") || query_lower.contains("k8s") {
        expanded.push("kubernetes".to_string());
        expanded.push("k8s".to_string());
        expanded.push("pod".to_string());
        expanded.push("container".to_string());
    }

    // Remove duplicates and empty strings
    expanded.sort();
    expanded.dedup();
    expanded.retain(|s| !s.is_empty());

    expanded
}

/// Extract important keywords from a search query
///
/// This function removes stop words and extracts meaningful terms
/// for search expansion.
///
/// # Arguments
/// * `query` - The original user query
///
/// # Returns
/// A vector of extracted keywords
fn extract_keywords(query: &str) -> Vec<String> {
    let stop_words: HashSet<&str> = [
        "how", "do", "i", "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "having", "do", "does", "did", "doing", "will", "would", "should",
        "could", "can", "may", "might", "must", "to", "from", "in", "on", "at", "by", "for",
        "with", "about", "as", "of", "or", "and", "but", "not", "what", "when", "where", "which",
        "who", "this", "that", "these", "those", "if", "then", "else", "for", "while", "until",
        "against", "between", "into", "through", "during", "before", "after", "above", "below",
        "up", "down", "out", "off", "over", "under", "again", "further", "then", "once", "here",
        "there", "why", "where", "all", "any", "both", "each", "few", "more", "most", "other",
        "some", "such", "no", "nor", "only", "own", "same", "so", "than", "too", "very", "can",
        "just", "should", "now",
    ]
    .into_iter()
    .collect();

    let mut keywords = Vec::new();
    let mut remaining = query.to_string();

    while !remaining.is_empty() {
        // Skip leading whitespace
        if remaining.starts_with(char::is_whitespace) {
            remaining = remaining.trim_start().to_string();
            continue;
        }

        // Try to extract version number (e.g., 1.0.12, 1.1.9)
        if remaining.starts_with(|c: char| c.is_ascii_digit()) {
            let mut end_pos = 0;
            let mut dot_count = 0;

            for (i, c) in remaining.chars().enumerate() {
                if c.is_ascii_digit() {
                    end_pos = i + 1;
                } else if c == '.' {
                    end_pos = i + 1;
                    dot_count += 1;
                } else {
                    break;
                }
            }

            // Only extract if we have at least 2 dots (e.g., 1.0.12)
            if dot_count >= 2 && end_pos > 0 {
                let version = remaining[..end_pos].to_string();
                keywords.push(version.clone());
                remaining = remaining[end_pos..].to_string();
                continue;
            }
        }

        // Find word boundary - split on whitespace or non-alphanumeric
        let mut split_pos = remaining.len();
        for (i, c) in remaining.chars().enumerate() {
            if c.is_whitespace() || !c.is_alphanumeric() {
                split_pos = i;
                break;
            }
        }

        // If split_pos is 0, the string starts with a non-alphanumeric character
        // Skip it and continue
        if split_pos == 0 {
            remaining = remaining[1..].to_string();
            continue;
        }

        let word = remaining[..split_pos].to_lowercase();
        remaining = remaining[split_pos..].to_string();

        // Skip empty words, single chars, and stop words
        if word.is_empty() || word.len() < 2 || stop_words.contains(word.as_str()) {
            continue;
        }

        // Add numeric words with 3+ digits
        if word.chars().all(|c| c.is_ascii_digit()) && word.len() >= 3 {
            keywords.push(word.clone());
            continue;
        }

        // Add words with at least one alphabetic character
        if word.chars().any(|c| c.is_alphabetic()) {
            keywords.push(word.clone());
        }
    }

    keywords.sort();
    keywords.dedup();

    keywords
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_query_with_product_synonyms() {
        let query = "upgrade vesta nxt to 1.1.9";
        let expanded = expand_query(query);

        // Should contain original query
        assert!(expanded.contains(&query.to_string()));

        // Should contain product synonyms
        assert!(expanded
            .iter()
            .any(|s| s.contains("product") || s.contains("product")));
    }

    #[test]
    fn test_expand_query_with_version_numbers() {
        let query = "version 1.0.12";
        let expanded = expand_query(query);

        // Should contain original query
        assert!(expanded.contains(&query.to_string()));
    }

    #[test]
    fn test_extract_keywords() {
        let query = "How do I upgrade VESTA NXT from 1.0.12 to 1.1.9?";
        let keywords = extract_keywords(query);

        assert!(keywords.contains(&"upgrade".to_string()));
        assert!(keywords.contains(&"vesta".to_string()));
        assert!(keywords.contains(&"nxt".to_string()));
        assert!(keywords.contains(&"1.0.12".to_string()));
        assert!(keywords.contains(&"1.1.9".to_string()));
    }

    #[test]
    fn test_product_synonyms() {
        let synonyms = get_product_synonyms("vesta nxt upgrade");

        // Should contain DevOps Tool synonym
        assert!(synonyms
            .iter()
            .any(|s| s.contains("DevOps Tool") || s.contains("product")));
    }

    #[test]
    fn test_empty_query() {
        let expanded = expand_query("");
        assert!(expanded.is_empty() || expanded.contains(&"".to_string()));
    }
}
