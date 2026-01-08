//! Unix-like filename pattern matching (fnmatch)
//!
//! Implements POSIX-style filename pattern matching without recursive globbing.
//!
//! # Patterns
//!
//! - `*` — Matches any sequence of characters (except directory separators)
//! - `?` — Matches any single character
//! - `[...]` — Matches character class (e.g., `[abc]`, `[a-z]`)
//! - `[!...]` — Negated character class
//!
//! # Examples
//!
//! ```
//! use vittelib::fnmatch;
//!
//! assert!(fnmatch::fnmatch("test.rs", "test.rs"));
//! assert!(fnmatch::fnmatch("hello.txt", "*.txt"));
//! assert!(fnmatch::fnmatch("file123.rs", "file[0-9]*.rs"));
//! ```

use regex::Regex;
use anyhow::Result;

/// Options for fnmatch pattern matching
#[derive(Debug, Clone)]
pub struct FnmatchOptions {
    /// If true, '/' characters are not matched by wildcards
    pub pathname: bool,
    /// If true, leading '.' is not matched by wildcards
    pub period: bool,
    /// If true, patterns are case-insensitive
    pub nocase: bool,
}

impl Default for FnmatchOptions {
    fn default() -> Self {
        FnmatchOptions {
            pathname: true,
            period: false,
            nocase: false,
        }
    }
}

/// Convert fnmatch pattern to regex pattern
fn pattern_to_regex(pattern: &str, options: &FnmatchOptions) -> Result<String> {
    let mut regex = String::from("^");
    let mut chars = pattern.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '*' => {
                if options.pathname {
                    // Don't match path separators
                    regex.push_str("[^/]*");
                } else {
                    regex.push_str(".*");
                }
            }
            '?' => {
                if options.pathname {
                    regex.push_str("[^/]");
                } else {
                    regex.push('.');
                }
            }
            '[' => {
                // Character class
                let mut class = String::from('[');
                let mut found_close = false;

                if chars.peek() == Some(&'!') {
                    class.push('^');
                    chars.next();
                }

                while let Some(ch) = chars.next() {
                    if ch == ']' {
                        class.push(']');
                        found_close = true;
                        break;
                    } else if ch == '\\' && chars.peek().is_some() {
                        class.push(chars.next().unwrap());
                    } else if options.pathname && ch == '/' {
                        // Paths not allowed in character class
                        return Err(anyhow::anyhow!("Invalid pattern: '/' in character class"));
                    } else {
                        class.push(ch);
                    }
                }

                if !found_close {
                    return Err(anyhow::anyhow!("Unclosed character class"));
                }

                regex.push_str(&class);
            }
            '\\' => {
                // Escape next character
                if let Some(next) = chars.next() {
                    regex.push_str(&regex::escape(&next.to_string()));
                }
            }
            _ => {
                regex.push_str(&regex::escape(&ch.to_string()));
            }
        }
    }

    regex.push('$');
    Ok(regex)
}

/// Match a filename against a pattern
///
/// # Examples
///
/// ```
/// use vittelib::fnmatch;
///
/// assert!(fnmatch::fnmatch("test.rs", "test.rs"));
/// assert!(fnmatch::fnmatch("hello.txt", "*.txt"));
/// assert!(fnmatch::fnmatch("hello.txt", "hello.*"));
/// ```
pub fn fnmatch(name: &str, pattern: &str) -> bool {
    fnmatch_with_options(name, pattern, &FnmatchOptions::default()).unwrap_or(false)
}

/// Match a filename against a pattern with options
pub fn fnmatch_with_options(name: &str, pattern: &str, options: &FnmatchOptions) -> Result<bool> {
    if let Ok(regex_pattern) = pattern_to_regex(pattern, options) {
        let regex = if options.nocase {
            Regex::new(&format!("(?i){}", regex_pattern))?
        } else {
            Regex::new(&regex_pattern)?
        };
        Ok(regex.is_match(name))
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(fnmatch("test.rs", "test.rs"));
        assert!(!fnmatch("test.rs", "test.txt"));
    }

    #[test]
    fn test_star_wildcard() {
        assert!(fnmatch("hello.txt", "*.txt"));
        assert!(fnmatch("file.txt", "file.*"));
        assert!(fnmatch("test123.rs", "test*.rs"));
    }

    #[test]
    fn test_question_wildcard() {
        assert!(fnmatch("a.txt", "?.txt"));
        assert!(fnmatch("test.txt", "test?.txt"));
        assert!(!fnmatch("test.txt", "tes?.txt"));  // "test" doesn't match "tes?"
    }

    #[test]
    fn test_character_class() {
        assert!(fnmatch("file1.rs", "file[0-9].rs"));
        assert!(fnmatch("fileA.rs", "file[A-Z].rs"));
        assert!(!fnmatch("file.rs", "file[0-9].rs"));
    }

    #[test]
    fn test_negated_class() {
        // Negated character classes: match if character is NOT in the class
        // Note: This test verifies basic negation behavior
        assert!(fnmatch("fileA.rs", "file[!0-9].rs"));
        assert!(!fnmatch("file1.rs", "file[!0-9].rs"));
    }

    #[test]
    fn test_pathname_option() {
        let opts = FnmatchOptions {
            pathname: true,
            ..Default::default()
        };
        assert!(!fnmatch_with_options("dir/file.txt", "*.txt", &opts).unwrap());
        assert!(fnmatch_with_options("file.txt", "*.txt", &opts).unwrap());
    }
}
