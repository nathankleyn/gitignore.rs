extern crate glob;

use error;

use std::path::Path;

#[derive(Debug)]
pub struct Pattern<'c> {
    pub pattern: glob::Pattern,
    pub anchored: bool,
    directory: bool,
    pub negation: bool,
    root: &'c Path
}

impl<'c> Pattern<'c> {
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
            parsed_pattern = parsed_pattern.trim_left().to_string();
        }

        let abs_pattern = Pattern::abs_pattern(&parsed_pattern, root, anchored);
        let pattern = try!(glob::Pattern::new(&abs_pattern));

        Ok(Pattern {
            pattern: pattern,
            anchored: anchored,
            directory: directory,
            negation: negation,
            root: root
        })
    }

    pub fn matches(&self, path: &Path, directory: bool) -> bool {
        if self.directory && !directory {
            return self.negation
        }

        // XOR the two together to calculate the match.
        self.negation ^ self.pattern.matches_path_with(&path, &self.match_options())
    }

    fn abs_pattern(pattern: &str, root: &Path, anchored: bool) -> String {
        if anchored {
            Pattern::abs_pattern_anchored(pattern, root)
        } else if !pattern.starts_with('*') {
            Pattern::abs_pattern_unanchored(pattern)
        } else {
            pattern.to_string()
        }
    }

    fn abs_pattern_anchored(pattern: &str, root: &Path) -> String {
        let mut root_path = root.to_str().unwrap().to_string();

        if root_path.ends_with("/") {
            root_path.pop();
        }

        root_path + pattern
    }

    fn abs_pattern_unanchored(pattern: &str) -> String {
        let wildcard = "*".to_string();
        wildcard + pattern
    }

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
    use test::Bencher;

    #[test]
    fn test_new_gitignore_pattern() {
        let gip = Pattern::new("*.foo", Path::new("/wing")).unwrap();
        assert!(gip.pattern == glob::Pattern::new("*.foo").unwrap());
        assert!(gip.anchored == false);
        assert!(gip.directory == false);
        assert!(gip.negation == false);
    }

    #[test]
    fn test_new_anchored_gitignore_pattern() {
        let gip = Pattern::new("/*.foo", Path::new("/wing")).unwrap();
        assert!(gip.pattern == glob::Pattern::new("/wing/*.foo").unwrap());
        assert!(gip.anchored == true);
        assert!(gip.directory == false);
        assert!(gip.negation == false);
    }

    #[test]
    fn test_new_anchored_gitignore_pattern_with_trailing_slash_on_root() {
        let gip = Pattern::new("/*.foo", Path::new("/wing/")).unwrap();
        assert!(gip.pattern == glob::Pattern::new("/wing/*.foo").unwrap());
        assert!(gip.anchored == true);
        assert!(gip.directory == false);
        assert!(gip.negation == false);
    }

    #[test]
    fn test_new_directory_gitignore_pattern() {
        let gip = Pattern::new("foo/", Path::new("/wing")).unwrap();
        assert!(gip.pattern == glob::Pattern::new("*foo").unwrap());
        assert!(gip.anchored == false);
        assert!(gip.directory == true);
        assert!(gip.negation == false);
    }

    #[test]
    fn test_new_negated_gitignore_pattern() {
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
        assert!(gip.matches(Path::new("foo"), false));
    }

    #[test]
    fn test_matches_unanchored_on_wildcard() {
        // returns true when given a path that matches a pattern of the format "*.something"
        let gip = Pattern::new("*.foo", Path::new("/")).unwrap();
        assert!(gip.matches(Path::new("bar.foo"), false));
    }

    #[test]
    fn test_matches_negated() {
        // returns false when given a path that has a negation pattern
        let gip = Pattern::new("! foo", Path::new("/")).unwrap();
        assert!(!gip.matches(Path::new("foo"), false));
    }

    #[test]
    fn test_matches_unanchored_on_nested_file() {
        // returns true when given a nested path that matches an unanchored wildcard pattern
        let gip = Pattern::new("*.foo", Path::new("/")).unwrap();
        assert!(gip.matches(Path::new("lux/bar.foo"), false));
    }

    #[test]
    fn test_matches_unanchored_on_nested_dir() {
        // returns true when given a nested path that matches an unanchored wildcard pattern
        let gip = Pattern::new("*foo", Path::new("/")).unwrap();
        assert!(gip.matches(Path::new("lux/bar/foo"), false));
    }

    #[test]
    fn test_matches_anchored_on_nested_file() {
        // returns false when given a nested path that matches an anchored wildcard pattern
        let gip = Pattern::new("/*.foo", Path::new("/")).unwrap();
        assert!(!gip.matches(Path::new("lux/bar.foo"), false));
    }

    #[test]
    fn test_matches_anchored_on_nested_dir() {
        // returns false when given a nested path that matches an anchored wildcard pattern
        let gip = Pattern::new("/foo/*", Path::new("/")).unwrap();
        assert!(!gip.matches(Path::new("foo/bar/lux"), false));
    }

    #[test]
    fn test_matches_directory_on_directory() {
        // returns true when given a path that is a directory and it matches a directory only pattern
        let gip = Pattern::new("foo/", Path::new("/")).unwrap();
        assert!(gip.matches(Path::new("foo"), true));
    }

    #[test]
    fn test_matches_directory_on_file() {
        // returns false when given a path that is a file and it matches a directory only pattern
        let gip = Pattern::new("foo/", Path::new("/")).unwrap();
        assert!(!gip.matches(Path::new("foo"), false));
    }

    #[bench]
    fn bench_pattern_new(b: &mut Bencher) {
        b.iter(|| Pattern::new("foo", Path::new("/")))
    }

    #[bench]
    fn bench_pattern_match(b: &mut Bencher) {
        let gip = Pattern::new("foo", Path::new("/")).unwrap();
        b.iter(|| gip.matches(Path::new("foo"), false))
    }
}
