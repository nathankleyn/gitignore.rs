use error;
use pattern;

use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Debug)]
pub struct File<'a> {
    patterns: Vec<pattern::Pattern<'a>>
}

impl<'b> File<'b> {
    pub fn new(path: &'b Path, root: Option<&'b Path>) -> Result<File<'b>, error::Error> {
        let root = root.unwrap_or(path.parent().unwrap());
        let patterns = try!(File::patterns(path, root));

        Ok(File {
            patterns: patterns
        })
    }

    fn patterns(path: &'b Path, root: &'b Path) -> Result<Vec<pattern::Pattern<'b>>, error::Error> {
        let mut file = try!(fs::File::open(path));
        let mut s = String::new();
        try!(file.read_to_string(&mut s));

        Ok(s.lines().filter_map(|line| {
            if !line.trim().is_empty() {
                pattern::Pattern::new(line, root).ok()
            } else {
                None
            }
        }).collect())
    }

    pub fn matches(&self, path: &'b Path) -> Result<bool, error::Error> {
        self.patterns.iter().fold(Ok(false), |acc_wrapped, pattern| {
            let acc = try!(acc_wrapped);

            // Save cycles - only run negations if there's anything to actually negate!
            if pattern.negation && !acc {
                return Ok(false)
            }

            let metadata = try!(fs::metadata(path));
            let matches = pattern.matches(&path, metadata.is_dir());

            let result = if !pattern.negation {
                acc || matches
            } else if matches {
                acc
            } else {
                false
            };

            Ok(result)
        })
    }
}

#[cfg(test)]
mod test {
    extern crate glob;
    extern crate tempdir;

    use super::File;

    use std::fs;
    use std::io::Write;
    use std::path::{Path,PathBuf};

    struct TestEnv<'a> {
        gitignore: &'a Path,
        paths: Vec<PathBuf>
    }

    #[test]
    fn test_new_file_with_unanchored_wildcard() {
        with_fake_repo("*.foo", vec!["bar.foo"], |test_env| {
            let file = File::new(test_env.gitignore, None).unwrap();
            for path in test_env.paths.iter() {
                assert!(file.matches(path.as_path()).unwrap());
            }
        })
    }

    #[test]
    fn test_new_file_with_anchored() {
        with_fake_repo("/out", vec!["out"], |test_env| {
            let file = File::new(test_env.gitignore, None).unwrap();
            for path in test_env.paths.iter() {
                assert!(file.matches(path.as_path()).unwrap());
            }
        })
    }

    fn with_fake_repo<F>(ignore_contents: &str, files: Vec<&str>, callback: F)
        where F: Fn(&TestEnv) {
        let dir = tempdir::TempDir::new("git_ignore_tests").unwrap();

        let paths = files.iter().map(|file| {
            let path = dir.path().join(file);
            write_to_file(&path, "");
            path
        }).collect();

        let gitignore= dir.path().join(".gitignore");
        write_to_file(gitignore.as_path(), ignore_contents);
        let test_env = TestEnv {
            gitignore: gitignore.as_path(),
            paths: paths
        };

        callback(&test_env);
        dir.close().unwrap();
    }

    fn write_to_file(path: &Path, contents: &str) {
        let mut file = fs::File::create(path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }
}
