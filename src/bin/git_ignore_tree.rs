extern crate git_ignore;

use git_ignore::file::File;
use std::env;
use std::fs;

/// Traverses the directory trees from the current working directory downwards, listing all the
/// files that are _not_ excluded because of the .gitignore rules. The rules are also loaded from
/// the current working directory.
pub fn main() {
    let pwd = env::current_dir().unwrap();
    let gitignore_path = pwd.join(".gitignore");
    let file = File::new(&gitignore_path, None).unwrap();

    let mut roots = vec![pwd];
    while let Some(root) = roots.pop() {
        let entries = fs::read_dir(root);

        for entry in entries.unwrap() {
            let path = entry.unwrap().path();
            if path.ends_with(".git") {
                continue;
            }

            let matches = file.matches(&path);
            if matches.is_err() || matches.unwrap() {
                continue;
            }

            println!("{}", path.to_str().unwrap());

            let metadata = fs::metadata(&path);
            if !metadata.is_err() && metadata.unwrap().is_dir() {
                roots.push(path);
            }
        }
    }
}
