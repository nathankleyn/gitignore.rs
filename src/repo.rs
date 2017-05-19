use errors::*;

use std::path::PathBuf;

struct Repo {

}

/// Given the path to a Git repository, load up all of the ignore files in the
/// usual Git heirachy and allow checking of ignore status against all of them.
impl Repo {
    fn new(path: PathBuf) -> Result<Repo> {
        unimplemented!();
    }
}
