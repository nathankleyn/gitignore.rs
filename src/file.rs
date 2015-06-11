use error;
use pattern;

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct File<'a> {
    patterns: Vec<pattern::Pattern<'a>>,
    root: &'a Path
}

impl<'b> File<'b> {
    pub fn new(path: &'b Path, root: Option<&'b Path>) -> Result<File<'b>, error::Error> {
        let root = root.unwrap_or(path.parent().unwrap());
        let patterns = try!(File::patterns(path, root));

        Ok(File {
            patterns: patterns,
            root: root
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
        let abs_path = self.abs_path(path);
        let directory = try!(fs::metadata(&abs_path)).is_dir();

        Ok(self.patterns.iter().fold(false, |acc, pattern| {
            // Save cycles - only run negations if there's anything to actually negate!
            if pattern.negation && !acc {
                return false;
            }

            let matches = pattern.matches(&abs_path, directory);

            if !pattern.negation {
                acc || matches
            } else {
                matches && acc
            }
        }))
    }

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
    use test::Bencher;

    struct TestEnv<'a> {
        gitignore: &'a Path,
        paths: Vec<PathBuf>
    }

    #[test]
    fn test_new_file_with_empty() {
        with_fake_repo("", vec!["bar.foo"], |test_env| {
            let file = File::new(test_env.gitignore, None).unwrap();
            for path in test_env.paths.iter() {
                assert!(!file.matches(path.as_path()).unwrap());
            }
        })
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

    #[bench]
    fn bench_new_file(b: &mut Bencher) {
        let path = Path::new(".gitignore");
        b.iter(|| {
            File::new(path, None).unwrap();
        })
    }

    #[bench]
    fn bench_file_match(b: &mut Bencher) {
        let file = File::new(Path::new(".gitignore"), None).unwrap();
        let path = Path::new("/dev/null");

        b.iter(|| {
            file.matches(path).unwrap();
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
