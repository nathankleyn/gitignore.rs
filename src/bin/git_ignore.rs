extern crate git_ignore;

use git_ignore::file::File;
use std::env;

pub fn main() {
    let pwd = env::current_dir().unwrap();
    let gitignore_path = pwd.join(".gitignore");
    let file = File::new(&gitignore_path, None).unwrap();

    for arg in env::args().skip(1) {
        let path = pwd.join(&arg);
        let matches = file.matches(&path).unwrap();
        println!("File: {}, Excluded: {}", arg, matches);
    }
}
