extern crate gitignore;

use std::env;

/// Traverses the directory trees from the current working directory downwards, listing all the
/// files that are _not_ excluded because of the .gitignore rules. The rules are also loaded from
/// the current working directory.
pub fn main() {
    let pwd = env::current_dir().unwrap();
    let gitignore_path = pwd.join(".gitignore");
    let file = gitignore::File::new(&gitignore_path).unwrap();

    for path in file.included_files().unwrap() {
        println!("{}", path.to_str().unwrap());
    }
}
