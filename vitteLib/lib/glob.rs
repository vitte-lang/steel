//! Recursive glob pattern matching
//!
//! Implements glob patterns with support for recursive matching using `**`.
//!
//! # Patterns
//!
//! - `*` — Matches any sequence of characters (except directory separators)
//! - `?` — Matches any single character
//! - `[...]` — Character class
//! - `**` — Matches any sequence of directories (recursive)
//!
//! # Examples
//!
//! ```
//! use vittelib::glob::GlobPattern;
//! use std::path::PathBuf;
//!
//! let pattern = GlobPattern::new("src/**/*.rs").unwrap();
//! assert!(pattern.matches(&PathBuf::from("src/main.rs")));
//! assert!(pattern.matches(&PathBuf::from("src/utils/helpers.rs")));
//! ```

use crate::fnmatch::FnmatchOptions;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Glob pattern for recursive matching
#[derive(Debug, Clone)]
pub struct GlobPattern {
    pattern: String,
    segments: Vec<PatternSegment>,
}

#[derive(Debug, Clone)]
enum PatternSegment {
    Exact(String),
    Wildcard(String),
    DoubleStar,
}

/// Options for glob matching
#[derive(Debug, Clone)]
pub struct GlobOptions {
    /// If true, patterns are case-insensitive
    pub nocase: bool,
    /// If true, hidden files (starting with .) are not matched
    pub no_hidden: bool,
}

impl Default for GlobOptions {
    fn default() -> Self {
        GlobOptions {
            nocase: false,
            no_hidden: false,
        }
    }
}

impl GlobPattern {
    /// Create a new glob pattern
    pub fn new(pattern: &str) -> Result<Self> {
        let segments = Self::parse_pattern(pattern)?;
        Ok(GlobPattern {
            pattern: pattern.to_string(),
            segments,
        })
    }

    fn parse_pattern(pattern: &str) -> Result<Vec<PatternSegment>> {
        let mut segments = Vec::new();

        for part in pattern.split('/') {
            if part == "**" {
                segments.push(PatternSegment::DoubleStar);
            } else if part.contains('*') || part.contains('?') || part.contains('[') {
                segments.push(PatternSegment::Wildcard(part.to_string()));
            } else {
                segments.push(PatternSegment::Exact(part.to_string()));
            }
        }

        Ok(segments)
    }

    /// Check if a path matches the pattern
    pub fn matches(&self, path: &Path) -> bool {
        self.matches_with_options(path, &GlobOptions::default())
    }

    /// Check if a path matches with options
    pub fn matches_with_options(&self, path: &Path, options: &GlobOptions) -> bool {
        let components: Vec<&str> = path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        self.match_segments(&components, 0, 0, options)
    }

    fn match_segments(
        &self,
        path_components: &[&str],
        seg_idx: usize,
        path_idx: usize,
        options: &GlobOptions,
    ) -> bool {
        // End of pattern and path
        if seg_idx >= self.segments.len() && path_idx >= path_components.len() {
            return true;
        }

        // End of pattern but not path
        if seg_idx >= self.segments.len() {
            return false;
        }

        match &self.segments[seg_idx] {
            PatternSegment::DoubleStar => {
                // Try matching zero directories
                if self.match_segments(path_components, seg_idx + 1, path_idx, options) {
                    return true;
                }

                // Try matching one or more directories
                for i in path_idx..=path_components.len() {
                    if self.match_segments(path_components, seg_idx + 1, i, options) {
                        return true;
                    }
                }

                false
            }
            PatternSegment::Exact(exact_part) => {
                if path_idx >= path_components.len() {
                    return false;
                }

                if path_components[path_idx] == exact_part {
                    self.match_segments(path_components, seg_idx + 1, path_idx + 1, options)
                } else {
                    false
                }
            }
            PatternSegment::Wildcard(pattern) => {
                if path_idx >= path_components.len() {
                    return false;
                }

                let component = path_components[path_idx];

                // Check for hidden files
                if options.no_hidden && component.starts_with('.') {
                    return false;
                }

                let fnmatch_opts = FnmatchOptions {
                    pathname: true,
                    period: false,
                    nocase: options.nocase,
                };

                if crate::fnmatch::fnmatch_with_options(component, pattern, &fnmatch_opts).unwrap_or(false)
                {
                    self.match_segments(path_components, seg_idx + 1, path_idx + 1, options)
                } else {
                    false
                }
            }
        }
    }

    /// Get the pattern string
    pub fn as_str(&self) -> &str {
        &self.pattern
    }
}

/// Simple glob matching function
pub fn glob(path: &Path, pattern: &str) -> Result<bool> {
    let glob_pattern = GlobPattern::new(pattern)?;
    Ok(glob_pattern.matches(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let pattern = GlobPattern::new("src/main.rs").unwrap();
        assert!(pattern.matches(&PathBuf::from("src/main.rs")));
        assert!(!pattern.matches(&PathBuf::from("src/lib.rs")));
    }

    #[test]
    fn test_single_star() {
        let pattern = GlobPattern::new("src/*.rs").unwrap();
        assert!(pattern.matches(&PathBuf::from("src/main.rs")));
        assert!(!pattern.matches(&PathBuf::from("src/utils/helpers.rs")));
    }

    #[test]
    fn test_double_star() {
        let pattern = GlobPattern::new("src/**/*.rs").unwrap();
        assert!(pattern.matches(&PathBuf::from("src/main.rs")));
        assert!(pattern.matches(&PathBuf::from("src/utils/helpers.rs")));
        assert!(pattern.matches(&PathBuf::from("src/utils/math/calc.rs")));
        assert!(!pattern.matches(&PathBuf::from("src/main.txt")));
    }

    #[test]
    fn test_wildcard_all() {
        let pattern = GlobPattern::new("**/*.rs").unwrap();
        assert!(pattern.matches(&PathBuf::from("main.rs")));
        assert!(pattern.matches(&PathBuf::from("src/main.rs")));
        assert!(pattern.matches(&PathBuf::from("a/b/c/d.rs")));
    }

    #[test]
    fn test_hidden_files() {
        let mut opts = GlobOptions::default();
        opts.no_hidden = true;

        let pattern = GlobPattern::new("src/**/*.rs").unwrap();
        // .hidden should be skipped when no_hidden is true
        assert!(!pattern.matches_with_options(&PathBuf::from("src/.hidden_dir/main.rs"), &opts));
        assert!(pattern.matches_with_options(&PathBuf::from("src/main.rs"), &opts));
    }

    #[test]
    fn test_case_insensitive() {
        let mut opts = GlobOptions::default();
        opts.nocase = true;

        let pattern = GlobPattern::new("src/*.RS").unwrap();
        assert!(pattern.matches_with_options(&PathBuf::from("src/main.rs"), &opts));
    }
}
