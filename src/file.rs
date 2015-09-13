use error;
use pattern;

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Represents a `.gitignore` file. Use this to load the `.gitignore` file, parse the patterns,
/// and then check if a given path would be excluded by any rules contained therein.
///
/// # Examples
///
/// ```
/// # use std::env;
/// # let pwd = env::current_dir().unwrap();
/// # let gitignore_path = pwd.join(".gitignore");
/// let file = gitignore::File::new(&gitignore_path).unwrap();
/// # let path_to_test_if_excluded = pwd.join("target");
/// assert!(file.is_excluded(&path_to_test_if_excluded).unwrap())
/// ```
#[derive(Debug)]
pub struct File<'a> {
    patterns: Vec<pattern::Pattern<'a>>,
    root: &'a Path
}

impl<'b> File<'b> {
    /// Parse the given `.gitignore` file for patterns, allowing any arbitrary path to be checked
    /// against the set of rules to test for exclusion.
    ///
    /// The value of `gitignore_path` must be an absolute path.
    pub fn new(gitignore_path: &'b Path) -> Result<File<'b>, error::Error> {
        let root = gitignore_path.parent().unwrap();
        let patterns = try!(File::patterns(gitignore_path, root));

        Ok(File {
            patterns: patterns,
            root: root
        })
    }

    /// Returns true if, after checking against all the patterns found in the `.gitignore` file,
    /// the given path is matched any of the globs (applying negated patterns as expected).
    ///
    /// If the value for `path` is not absolute, it will assumed to be relative to the current
    /// working directory.
    pub fn is_excluded(&self, path: &'b Path) -> Result<bool, error::Error> {
        let abs_path = self.abs_path(path);
        let directory = try!(fs::metadata(&abs_path)).is_dir();

        Ok(self.patterns.iter().fold(false, |acc, pattern| {
            // Save cycles - only run negations if there's anything to actually negate!
            if pattern.negation && !acc {
                return false;
            }

            let matches = pattern.is_excluded(&abs_path, directory);

            if !pattern.negation {
                acc || matches
            } else {
                matches && acc
            }
        }))
    }

    /// Returns a list of files that are not excluded by the rules in the loaded
    /// `.gitignore` file. It recurses through all subdirectories and returns
    /// everything that is not ignored.
    pub fn included_files(&self) -> Result<Vec<PathBuf>, error::Error> {
        let mut files: Vec<PathBuf> = vec![];
        let mut roots = vec![self.root.to_path_buf()];

        while let Some(root) = roots.pop() {
            let entries = try!(fs::read_dir(root));

            for entry in entries {
                let path = try!(entry).path();
                if path.ends_with(".git") {
                    continue;
                }

                let matches = self.is_excluded(&path);
                if matches.is_err() || try!(matches) {
                    continue;
                }

                files.push(path.to_path_buf());

                let metadata = fs::metadata(&path);
                if !metadata.is_err() && try!(metadata).is_dir() {
                    roots.push(path);
                }
            }
        }

        Ok(files)
    }

    /// Given the path to the `.gitignore` file and the root folder within which it resides,
    /// parse out all the patterns and collect them up into a vector of patterns.
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

    /// Given a path, make it absolute if relative by joining it to a given root, otherwise leave
    /// absolute as originally given.
    fn abs_path(&self, path: &'b Path) -> PathBuf {
        if path.is_absolute() {
            path.to_owned()
        } else {
            self.root.join(path)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate glob;
    extern crate tempdir;

    use super::File;

    use std::fs;
    use std::io::Write;
    use std::path::{Path,PathBuf};

    #[cfg(feature = "nightly")]
    use test::Bencher;

    struct TestEnv<'a> {
        gitignore: &'a Path,
        paths: Vec<PathBuf>
    }

    #[test]
    fn test_new_file_with_empty() {
        with_fake_repo("", vec!["bar.foo"], |test_env| {
            let file = File::new(test_env.gitignore).unwrap();
            for path in test_env.paths.iter() {
                assert!(!file.is_excluded(path.as_path()).unwrap());
            }
        })
    }

    #[test]
    fn test_new_file_with_unanchored_wildcard() {
        with_fake_repo("*.foo", vec!["bar.foo"], |test_env| {
            let file = File::new(test_env.gitignore).unwrap();
            for path in test_env.paths.iter() {
                assert!(file.is_excluded(path.as_path()).unwrap());
            }
        })
    }

    #[test]
    fn test_new_file_with_anchored() {
        with_fake_repo("/out", vec!["out"], |test_env| {
            let file = File::new(test_env.gitignore).unwrap();
            for path in test_env.paths.iter() {
                assert!(file.is_excluded(path.as_path()).unwrap());
            }
        })
    }

    #[test]
    fn test_included_files() {
        with_fake_repo("*.foo", vec!["bar.foo", "foo", "bar"], |test_env| {
            let file = File::new(test_env.gitignore).unwrap();
            let files: Vec<String> = file.included_files().unwrap().iter().map(|path|
                path.file_name().unwrap().to_str().unwrap().to_string()
            ).collect();

            // We can't compare the vec directly, as the order can differ
            // depending on underlying platform. Instead, let's break it
            // apart into the respective assertions.
            assert!(files.len() == 3);
            assert!(files.contains(&".gitignore".to_string()));
            assert!(files.contains(&"bar".to_string()));
            assert!(files.contains(&"foo".to_string()));
        })
    }

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_new_file(b: &mut Bencher) {
        let path = Path::new(".gitignore");
        b.iter(|| {
            File::new(path).unwrap();
        })
    }

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_file_match(b: &mut Bencher) {
        let file = File::new(Path::new(".gitignore")).unwrap();
        let path = Path::new("/dev/null");

        b.iter(|| {
            file.is_excluded(path).unwrap();
        })
    }

    fn with_fake_repo<F>(ignore_contents: &str, files: Vec<&str>, callback: F)
        where F: Fn(&TestEnv) {
        let dir = tempdir::TempDir::new("gitignore_tests").unwrap();

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
