use crate::ruleset::*;
use failure::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct IgnoreFile {
    ruleset: RuleSet
}

/// Given a single specific gitignore style file, allow matching against
/// the rules within that file.
impl IgnoreFile {
    pub fn new<P: AsRef<Path>, P2: Lines>(root: P, lines_source: P2) -> Result<IgnoreFile, Error> {
        let lines: Vec<String> = lines_source.lines()?;
        let rule_set = RuleSet::new(root, lines.as_slice())?;

        Ok(IgnoreFile {
          ruleset: rule_set
        })
    }

    pub fn is_ignored<P: AsRef<Path>>(&self, path: P, is_dir: bool) -> bool {
        self.ruleset.is_ignored(path, is_dir)
    }
}

#[cfg(test)]
mod test {
    use super::{IgnoreFile, RuleSet};
    use std::path::PathBuf;
    macro_rules! ignore_file_from_test_repo {
        ($ignore_path:expr) => {
            {
                let cargo_root: PathBuf = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
                let root: PathBuf = cargo_root.join("tests/resources/fake_repo").to_path_buf();
                let ignore: PathBuf = root.join($ignore_path).to_path_buf();

                IgnoreFile::new(root, ignore).unwrap()
            }
        };
    }

    fn ruleset_from_rules<S: AsRef<str>>(raw_rules: S) -> RuleSet {
        let rules: Vec<String> = raw_rules.as_ref().lines().map(|s| s.to_string()).collect();
        RuleSet::new("foo", rules.iter()).unwrap()
    }

    #[test]
    #[should_panic]
    fn fails_when_file_is_missing() {
        IgnoreFile::new("/i/do/not/exist", "/i/do/not/exist/.gitignore").unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_when_rules_invalid() {
        ignore_file_from_test_repo!(".badgitignore");
    }

    #[test]
    fn returns_correctly_an_ignorefile_from_valid_file() {
        let file = ignore_file_from_test_repo!(".gitignore");

        assert_eq!(file.ruleset.rules, ruleset_from_rules("*.no\nnot_me_either/\n/or_even_me").rules)
    }

    #[test]
    fn allows_passing_rows() {
        let ignore = IgnoreFile::new("/a/b/c/d", vec!["*.rs", "blah", "kuku"]);
        println!("{:?}", ignore);
    }
}


pub trait Lines {
    fn lines(&self) -> Result<Vec<String>, Error>;
}

impl Lines for &str {
    fn lines(&self) -> Result<Vec<String>, Error> {
        lines(self)
    }
}

impl Lines for PathBuf {
    fn lines(&self) -> Result<Vec<String>, Error> {
        lines(self)
    }
}

impl Lines for &PathBuf {
    fn lines(&self) -> Result<Vec<String>, Error> {
        lines(self)
    }
}

pub fn lines<T>(thing: T) -> Result<Vec<String>, Error> where T: AsRef<Path> {
    let file = File::open(thing)?;
    let lines: Vec<String> = BufReader::new(file).lines().flat_map(|line| line.ok()).collect();
    Ok(lines)
}
impl Lines for Vec<String> {
    fn lines(&self) -> Result<Vec<String>, Error> {
        Ok(self.clone())
    }
}

impl Lines for Vec<&str> {
    fn lines(&self) -> Result<Vec<String>, Error> {
        Ok(self.iter().map(|x| x.to_string()).collect())
    }
}
