// Git source URL parsing and types
//
// Supports various Git URL formats:
// - HTTPS: https://github.com/user/repo
// - Shorthand: github:user/repo, gitlab:user/repo
// - SSH: git@github.com:user/repo.git
// - With ref: github:user/repo@v1.0.0, github:user/repo@main

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Represents a parsed Git source URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSource {
    /// Repository URL (normalized to HTTPS for cloning)
    pub url: String,
    /// Repository owner/organization
    pub owner: String,
    /// Repository name
    pub repo: String,
    /// Git reference (branch, tag, or commit)
    pub git_ref: GitRef,
    /// Original input string for display
    pub original: String,
}

/// Git reference type
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum GitRef {
    /// Use the default branch (main/master)
    #[default]
    DefaultBranch,
    /// A specific branch
    Branch(String),
    /// A tag (usually versions like v1.0.0)
    Tag(String),
    /// A specific commit SHA
    Commit(String),
}

impl GitRef {
    /// Get the refspec string for checkout
    pub fn as_refspec(&self) -> Option<&str> {
        match self {
            GitRef::DefaultBranch => None,
            GitRef::Branch(b) => Some(b),
            GitRef::Tag(t) => Some(t),
            GitRef::Commit(c) => Some(c),
        }
    }

    /// Check if this is a pinned ref (tag or commit)
    pub fn is_pinned(&self) -> bool {
        matches!(self, GitRef::Tag(_) | GitRef::Commit(_))
    }
}

impl std::fmt::Display for GitRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitRef::DefaultBranch => write!(f, "HEAD"),
            GitRef::Branch(b) => write!(f, "{}", b),
            GitRef::Tag(t) => write!(f, "{}", t),
            GitRef::Commit(c) => write!(f, "{}", &c[..7.min(c.len())]),
        }
    }
}

impl GitSource {
    /// Get a unique identifier for this source (for caching)
    pub fn cache_key(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }

    /// Get display name
    pub fn display_name(&self) -> String {
        match &self.git_ref {
            GitRef::DefaultBranch => format!("{}/{}", self.owner, self.repo),
            ref_type => format!("{}/{}@{}", self.owner, self.repo, ref_type),
        }
    }
}

/// Parse various Git URL formats into a normalized GitSource
///
/// Supported formats:
/// - `https://github.com/user/repo`
/// - `https://github.com/user/repo.git`
/// - `github:user/repo`
/// - `github:user/repo@v1.0.0`
/// - `git@github.com:user/repo.git`
/// - `gitlab:user/repo`
/// - `https://gitlab.com/user/repo`
pub fn parse_git_url(input: &str) -> Result<GitSource> {
    let original = input.to_string();

    // Handle shorthand formats: github:user/repo[@ref]
    if let Some(rest) = input.strip_prefix("github:") {
        return parse_shorthand("github.com", rest, original);
    }
    if let Some(rest) = input.strip_prefix("gitlab:") {
        return parse_shorthand("gitlab.com", rest, original);
    }
    if let Some(rest) = input.strip_prefix("bitbucket:") {
        return parse_shorthand("bitbucket.org", rest, original);
    }

    // Handle SSH format: git@github.com:user/repo.git
    if input.starts_with("git@") {
        return parse_ssh_url(input, original);
    }

    // Handle HTTPS URLs
    if input.starts_with("https://") || input.starts_with("http://") {
        return parse_https_url(input, original);
    }

    anyhow::bail!(
        "Unsupported Git URL format: {}\n\
         Supported formats:\n\
         - github:user/repo\n\
         - github:user/repo@v1.0.0\n\
         - https://github.com/user/repo\n\
         - git@github.com:user/repo.git",
        input
    );
}

fn parse_shorthand(host: &str, rest: &str, original: String) -> Result<GitSource> {
    // Split by @ for ref: user/repo@v1.0.0
    let (path, git_ref) = if let Some(at_pos) = rest.rfind('@') {
        let ref_str = &rest[at_pos + 1..];
        let path = &rest[..at_pos];
        (path, parse_ref(ref_str))
    } else {
        (rest, GitRef::DefaultBranch)
    };

    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 2 {
        anyhow::bail!(
            "Invalid shorthand format '{}'. Expected: user/repo or user/repo@version",
            rest
        );
    }

    let owner = parts[0].to_string();
    let repo = parts[1].trim_end_matches(".git").to_string();

    Ok(GitSource {
        url: format!("https://{}/{}/{}.git", host, owner, repo),
        owner,
        repo,
        git_ref,
        original,
    })
}

fn parse_ssh_url(input: &str, original: String) -> Result<GitSource> {
    // git@github.com:user/repo.git
    let without_prefix = input
        .strip_prefix("git@")
        .context("Invalid SSH URL format")?;

    let colon_pos = without_prefix
        .find(':')
        .context("Invalid SSH URL: missing colon separator")?;

    let host = &without_prefix[..colon_pos];
    let path = &without_prefix[colon_pos + 1..];

    let parts: Vec<&str> = path.trim_end_matches(".git").split('/').collect();
    if parts.len() < 2 {
        anyhow::bail!("Invalid SSH URL: expected user/repo format after host");
    }

    Ok(GitSource {
        url: format!("https://{}/{}", host, path),
        owner: parts[0].to_string(),
        repo: parts[1].trim_end_matches(".git").to_string(),
        git_ref: GitRef::DefaultBranch,
        original,
    })
}

fn parse_https_url(input: &str, original: String) -> Result<GitSource> {
    let url = url::Url::parse(input).context("Invalid URL")?;
    let host = url.host_str().context("Missing host in URL")?;

    let path_segments: Vec<&str> = url
        .path_segments()
        .context("Invalid URL path")?
        .filter(|s| !s.is_empty())
        .collect();

    if path_segments.len() < 2 {
        anyhow::bail!("URL must include owner/repo path: {}", input);
    }

    let owner = path_segments[0].to_string();
    let repo = path_segments[1].trim_end_matches(".git").to_string();

    // Check for ref in URL fragment
    let git_ref = if let Some(fragment) = url.fragment() {
        parse_ref(fragment)
    } else {
        GitRef::DefaultBranch
    };

    Ok(GitSource {
        url: format!("https://{}/{}/{}.git", host, owner, repo),
        owner,
        repo,
        git_ref,
        original,
    })
}

fn parse_ref(ref_str: &str) -> GitRef {
    // Tags typically start with 'v' followed by a number
    if ref_str.starts_with('v')
        && ref_str
            .chars()
            .nth(1)
            .is_some_and(|c| c.is_ascii_digit())
    {
        GitRef::Tag(ref_str.to_string())
    }
    // 40-character hex strings are commit SHAs
    else if ref_str.len() == 40 && ref_str.chars().all(|c| c.is_ascii_hexdigit()) {
        GitRef::Commit(ref_str.to_string())
    }
    // Everything else is treated as a branch name
    else {
        GitRef::Branch(ref_str.to_string())
    }
}

/// Check if a string looks like a Git URL
pub fn is_git_url(input: &str) -> bool {
    input.starts_with("https://github.com")
        || input.starts_with("https://gitlab.com")
        || input.starts_with("https://bitbucket.org")
        || input.starts_with("github:")
        || input.starts_with("gitlab:")
        || input.starts_with("bitbucket:")
        || input.starts_with("git@")
        || (input.starts_with("https://") && input.ends_with(".git"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_shorthand() {
        let source = parse_git_url("github:user/my-skill").unwrap();
        assert_eq!(source.owner, "user");
        assert_eq!(source.repo, "my-skill");
        assert_eq!(source.url, "https://github.com/user/my-skill.git");
        assert_eq!(source.git_ref, GitRef::DefaultBranch);
    }

    #[test]
    fn test_github_shorthand_with_tag() {
        let source = parse_git_url("github:user/my-skill@v1.0.0").unwrap();
        assert_eq!(source.repo, "my-skill");
        assert!(matches!(source.git_ref, GitRef::Tag(ref t) if t == "v1.0.0"));
    }

    #[test]
    fn test_github_shorthand_with_branch() {
        let source = parse_git_url("github:user/repo@main").unwrap();
        assert!(matches!(source.git_ref, GitRef::Branch(ref b) if b == "main"));
    }

    #[test]
    fn test_https_url() {
        let source = parse_git_url("https://github.com/user/repo").unwrap();
        assert_eq!(source.owner, "user");
        assert_eq!(source.repo, "repo");
        assert_eq!(source.url, "https://github.com/user/repo.git");
    }

    #[test]
    fn test_https_url_with_git_suffix() {
        let source = parse_git_url("https://github.com/user/repo.git").unwrap();
        assert_eq!(source.repo, "repo");
    }

    #[test]
    fn test_ssh_url() {
        let source = parse_git_url("git@github.com:user/repo.git").unwrap();
        assert_eq!(source.owner, "user");
        assert_eq!(source.repo, "repo");
    }

    #[test]
    fn test_gitlab_shorthand() {
        let source = parse_git_url("gitlab:org/project").unwrap();
        assert_eq!(source.url, "https://gitlab.com/org/project.git");
    }

    #[test]
    fn test_is_git_url() {
        assert!(is_git_url("github:user/repo"));
        assert!(is_git_url("https://github.com/user/repo"));
        assert!(is_git_url("git@github.com:user/repo.git"));
        assert!(!is_git_url("./local/path"));
        assert!(!is_git_url("/absolute/path"));
        assert!(!is_git_url("my-skill"));
    }

    #[test]
    fn test_cache_key() {
        let source = parse_git_url("github:user/repo@v1.0.0").unwrap();
        assert_eq!(source.cache_key(), "user/repo");
    }

    #[test]
    fn test_display_name() {
        let source = parse_git_url("github:user/repo").unwrap();
        assert_eq!(source.display_name(), "user/repo");

        let source_with_tag = parse_git_url("github:user/repo@v1.0.0").unwrap();
        assert_eq!(source_with_tag.display_name(), "user/repo@v1.0.0");
    }

    #[test]
    fn test_commit_sha() {
        let sha = "abc123def456789012345678901234567890abcd";
        let source = parse_git_url(&format!("github:user/repo@{}", sha)).unwrap();
        assert!(matches!(source.git_ref, GitRef::Commit(ref c) if c == sha));
    }
}
