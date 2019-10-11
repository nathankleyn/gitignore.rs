extern crate glob;

use error;

use std::path::Path;

/// A pattern as found in a `.gitignore` file.
#[derive(Debug)]
pub struct Pattern<'c> {
    /// The glob pattern after being parsed, negation or trailing directory slashes removed,
    /// and the root prepended if anchored.
    pub pattern: glob::Pattern,
    /// Whether the pattern had the root prepended so the matches must be within the root
    /// directory. That is to say, whether the pattern was anchored to the root.
    pub anchored: bool,
    /// Whether the pattern should, if it matches, negate any previously matching patterns. This
    /// flag has no effect if no previous patterns had matched.
    pub negation: bool,
    directory: bool,
    root: &'c Path
}

impl<'c> Pattern<'c> {
    /// Create a new pattern from the raw glob as found in a `.gitignore` file.
    ///
    /// The value of `root` must be an absolute path.
    pub fn new(raw_pattern: &str, root: &'c Path) -> Result<Pattern<'c>, error::Error> {
        let mut parsed_pattern = raw_pattern.to_string();
        let directory = parsed_pattern.ends_with('/');

        if directory {
            parsed_pattern.pop();
        }

        let anchored = parsed_pattern.contains('/');
        let negation = parsed_pattern.starts_with('!');

        if negation {
            parsed_pattern.remove(0);
            parsed_pattern = parsed_pattern.trim_start().to_string();
        }

        let abs_pattern = Pattern::abs_pattern(&parsed_pattern, root, anchored);
        let pattern = glob::Pattern::new(&abs_pattern)?;

        Ok(Pattern { pattern, anchored, negation, directory, root })
    }

    /// Returns true if the given path is matched by the current pattern, and hence would be
    /// excluded if found in a `.gitignore` file. The second argument, `directory`, is a `bool`
    /// representing whether the given path is a directory - if so, it should be set to `true`,
    /// otherwise `false` if not (eg. file, special file, symlink).
    ///
    /// Note that if the glob was negated (ie. of the format `! some/glob/*.here`) then this will
    /// return the opposite value, eg. `false` if the pattern matched, and `true` if the pattern
    /// did not match.
    ///
    /// The value of `path` must be an absolute path.
    pub fn is_excluded(&self, path: &Path, directory: bool) -> bool {
        if self.directory && !directory {
            return self.negation
        }

        // XOR the two together to calculate the match.
        self.negation ^ self.pattern.matches_path_with(&path, self.match_options())
    }

    /// Take the given pattern as a glob, and if anchoring is required, join the given root to the
    /// beginning of the pattern. If the glob is unanchored, instead prepend a wildcard.
    fn abs_pattern(pattern: &str, root: &Path, anchored: bool) -> String {
        if anchored {
            Pattern::abs_pattern_anchored(pattern, root)
        } else if !pattern.starts_with('*') {
            Pattern::abs_pattern_unanchored(pattern)
        } else {
            pattern.to_string()
        }
    }

    /// Given an anchored pattern, join the given root to the beginning of the pattern.
    fn abs_pattern_anchored(pattern: &str, root: &Path) -> String {
        let mut root_path = root.to_str().unwrap().to_string();

        if root_path.ends_with('/') {
            root_path.pop();
        }

        root_path + pattern
    }

    /// Given an an unanchored pattern, prepend a wildcard.
    fn abs_pattern_unanchored(pattern: &str) -> String {
        let wildcard = "*".to_string();
        wildcard + pattern
    }

    /// Return the options that should be given to the glob match function, taking into account
    /// whether the current pattern is anchored or unanchored so as to apply the glob options
    /// described in the ["Pattern Format" section](https://www.kernel.org/pub/software/scm/git/docs/gitignore.html#_pattern_format))
    /// of the man pages on the `.gitignore` format.
    fn match_options(&self) -> glob::MatchOptions {
        glob::MatchOptions {
            case_sensitive: false,
            require_literal_separator: self.anchored,
            require_literal_leading_dot: false
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate glob;

    use super::Pattern;
    use std::path::Path;

    #[cfg(feature = "nightly")]
    use test::Bencher;

    #[test]
    fn test_new_git_ignore_pattern() {
        let gip = Pattern::new("*.foo", Path::new("/wing")).unwrap();
        assert!(gip.pattern == glob::Pattern::new("*.foo").unwrap());
        assert!(gip.anchored == false);
        assert!(gip.directory == false);
        assert!(gip.negation == false);
    }

    #[test]
    fn test_new_anchored_git_ignore_pattern() {
        let gip = Pattern::new("/*.foo", Path::new("/wing")).unwrap();
        assert!(gip.pattern == glob::Pattern::new("/wing/*.foo").unwrap());
        assert!(gip.anchored == true);
        assert!(gip.directory == false);
        assert!(gip.negation == false);
    }

    #[test]
    fn test_new_anchored_git_ignore_pattern_with_trailing_slash_on_root() {
        let gip = Pattern::new("/*.foo", Path::new("/wing/")).unwrap();
        assert!(gip.pattern == glob::Pattern::new("/wing/*.foo").unwrap());
        assert!(gip.anchored == true);
        assert!(gip.directory == false);
        assert!(gip.negation == false);
    }

    #[test]
    fn test_new_directory_git_ignore_pattern() {
        let gip = Pattern::new("foo/", Path::new("/wing")).unwrap();
        assert!(gip.pattern == glob::Pattern::new("*foo").unwrap());
        assert!(gip.anchored == false);
        assert!(gip.directory == true);
        assert!(gip.negation == false);
    }

    #[test]
    fn test_new_negated_git_ignore_pattern() {
        let gip = Pattern::new("! *.foo", Path::new("/wing")).unwrap();
        assert!(gip.pattern == glob::Pattern::new("*.foo").unwrap());
        assert!(gip.anchored == false);
        assert!(gip.directory == false);
        assert!(gip.negation == true);
    }

    #[test]
    fn test_matches_simple() {
        // returns true when given a path that matches a pattern of the format "something"
        let gip = Pattern::new("foo", Path::new("/")).unwrap();
        assert!(gip.is_excluded(Path::new("foo"), false));
    }

    #[test]
    fn test_matches_unanchored_on_wildcard() {
        // returns true when given a path that matches a pattern of the format "*.something"
        let gip = Pattern::new("*.foo", Path::new("/")).unwrap();
        assert!(gip.is_excluded(Path::new("bar.foo"), false));
    }

    #[test]
    fn test_matches_negated() {
        // returns false when given a path that has a negation pattern
        let gip = Pattern::new("! foo", Path::new("/")).unwrap();
        assert!(!gip.is_excluded(Path::new("foo"), false));
    }

    #[test]
    fn test_matches_unanchored_on_nested_file() {
        // returns true when given a nested path that matches an unanchored wildcard pattern
        let gip = Pattern::new("*.foo", Path::new("/")).unwrap();
        assert!(gip.is_excluded(Path::new("lux/bar.foo"), false));
    }

    #[test]
    fn test_matches_unanchored_on_nested_dir() {
        // returns true when given a nested path that matches an unanchored wildcard pattern
        let gip = Pattern::new("*foo", Path::new("/")).unwrap();
        assert!(gip.is_excluded(Path::new("lux/bar/foo"), false));
    }

    #[test]
    fn test_matches_anchored_on_nested_file() {
        // returns false when given a nested path that matches an anchored wildcard pattern
        let gip = Pattern::new("/*.foo", Path::new("/")).unwrap();
        assert!(!gip.is_excluded(Path::new("lux/bar.foo"), false));
    }

    #[test]
    fn test_matches_anchored_on_nested_dir() {
        // returns false when given a nested path that matches an anchored wildcard pattern
        let gip = Pattern::new("/foo/*", Path::new("/")).unwrap();
        assert!(!gip.is_excluded(Path::new("foo/bar/lux"), false));
    }

    #[test]
    fn test_matches_directory_on_directory() {
        // returns true when given a path that is a directory and it matches a directory only pattern
        let gip = Pattern::new("foo/", Path::new("/")).unwrap();
        assert!(gip.is_excluded(Path::new("foo"), true));
    }

    #[test]
    fn test_matches_directory_on_file() {
        // returns false when given a path that is a file and it matches a directory only pattern
        let gip = Pattern::new("foo/", Path::new("/")).unwrap();
        assert!(!gip.is_excluded(Path::new("foo"), false));
    }

    #[test]
    fn test_double_star_matches_nested_dir() {
        let gip = Pattern::new("foo/**", Path::new("/")).unwrap();
        assert!(gip.is_excluded(Path::new("foo/bar"), true));
    }

    #[test]
    fn test_negate_double_star_matches_nested_dir() {
        let gip = Pattern::new("!foo/**", Path::new("/")).unwrap();
        assert!(!gip.is_excluded(Path::new("foo/bar"), true));
    }

    #[test]
    fn test_double_star_matches_nested_file() {
        let gip = Pattern::new("foo/**", Path::new("/")).unwrap();
        assert!(gip.is_excluded(Path::new("foo/bar/index.html"), false));
    }

    #[test]
    fn test_negate_double_star_matches_nested_file() {
        let gip = Pattern::new("!foo/**", Path::new("/")).unwrap();
        assert!(!gip.is_excluded(Path::new("foo/bar/index.html"), false));
    }

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_pattern_new(b: &mut Bencher) {
        b.iter(|| Pattern::new("foo", Path::new("/")))
    }

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_pattern_match(b: &mut Bencher) {
        let gip = Pattern::new("foo", Path::new("/")).unwrap();
        b.iter(|| gip.is_excluded(Path::new("foo"), false))
    }
}
