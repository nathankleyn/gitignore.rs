use crate::ignore_file::*;
use failure::Error;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::ruleset::IgnoreResult;

pub struct Repo {
    root: PathBuf,
    ignore_files: HashMap<PathBuf, IgnoreFile>
}

/// Given the path to a Git repository, load up all of the ignore files in the
/// usual Git heirachy and allow checking of ignore status against all of them.
impl Repo {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Repo, Error> {
        let glob = root.as_ref().join("**/.gitignore").to_string_lossy().into_owned();
        let files = glob::glob(&glob)?;

        let ignore_files: HashMap<PathBuf, IgnoreFile> =
            files.flat_map(|glob_result| glob_result.ok())
                .flat_map(|file| IgnoreFile::new(&root, &file).map(|ignore_file| (file, ignore_file)))
                .collect();

        Ok(Repo {
            root: root.as_ref().to_path_buf(),
            ignore_files,
        })
    }

    pub fn is_ignored<P: AsRef<Path>>(&self, path: P, is_dir: bool) -> bool {
        let mut abs_path = path.as_ref().to_path_buf();
        if abs_path.is_relative() {
            abs_path = self.root.join(&path);
        }

        let result = abs_path.parent()// XXX should we catch root here? or add a root guard?
            .unwrap()
            .ancestors()
            .filter_map(|ancestor| {
                if ancestor.starts_with(&self.root) {
                    Some(ancestor.join(".gitignore"))
                } else {
                    None
                }
            })
            .filter_map(|ancestor| self.ignore_files.get(&ancestor))
            .find_map(|ignore_file| {
                let ignore_result = ignore_file.is_ignored_or_negated(&path, is_dir);
                if ignore_result == IgnoreResult::Undefined {
                    None
                } else {
                    Some(ignore_result)
                }
            });

        if let Some(ignore_result) = result {
            ignore_result == IgnoreResult::Ignored
        } else {
            false
        }
    }
}


#[cfg(test)]
mod test {
    use super::Repo;
    use std::path::PathBuf;

    macro_rules! test_repo {
        () => {
            {
                let cargo_root: PathBuf = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
                let root: PathBuf = cargo_root.join("tests/resources/fake_repo").to_path_buf();

                Repo::new(root).unwrap()
            }
        };
    }

    #[test]
    fn is_ignored_is_false_for_all_expected_files() {
        let repo = test_repo!();

        assert!(!repo.is_ignored(".badgitignore", false));
        assert!(!repo.is_ignored(".gitignore", false));
        assert!(!repo.is_ignored("also_include_me", false));
        assert!(!repo.is_ignored("include_me", false));
        assert!(!repo.is_ignored("im_included/hello.greeting", false));
        assert!(!repo.is_ignored("a_dir/a_nested_dir/.gitignore", false));
        assert!(!repo.is_ignored("a_dir/a_nested_dir/deeper_still/bit_now_i_work.no", false));
    }

    #[test]
    fn is_ignored_is_true_for_all_expected_files() {
        let repo = test_repo!();

        assert!(repo.is_ignored("not_me.no", false));
        assert!(repo.is_ignored("or_even_me", false));
        assert!(repo.is_ignored("or_me.no", false));
        assert!(repo.is_ignored("a_dir/a_nested_dir/deeper_still/hello.greeting", false));
        assert!(repo.is_ignored("a_dir/a_nested_dir/deeper_still/hola.greeting", false));
        // FIXME properly handle
        // assert!(repo.is_ignored("not_me_either/not_me.neg", false));
    }

    #[test]
    fn is_ignored_is_false_for_all_expected_directories() {
        let repo = test_repo!();

        assert!(!repo.is_ignored("im_included", true));
        assert!(!repo.is_ignored("a_dir/a_nested_dir", true));
    }

    #[test]
    fn is_ignored_is_true_for_all_expected_directories() {
        let repo = test_repo!();

        assert!(repo.is_ignored("not_me_either", true));
    }
}
