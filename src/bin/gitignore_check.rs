extern crate gitignore;

use std::env;

/// Given a list of files, check the status of these files and whether they are excluded because
/// of the .gitignore rules in the current working directory.
pub fn main() {
    let pwd = env::current_dir().unwrap();
    let gitignore_path = pwd.join(".gitignore");
    let file = gitignore::File::new(&gitignore_path).unwrap();

    for arg in env::args().skip(1) {
        let path = pwd.join(&arg);
        let matches = file.is_excluded(&path).unwrap();
        println!("File: {}, Excluded: {}", arg, matches);
    }
}
