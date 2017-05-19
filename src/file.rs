
use errors::*;
use ruleset::*;

use std::path::Path;

struct File {
    rules: RuleSet
}

/// Given a single specific gitignore style file, allow matching against
/// the rules within that file.
impl File {
    fn new<P: AsRef<Path>>(path: P) -> Result<File> {
        unimplemented!();
    }
}

//
