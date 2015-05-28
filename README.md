# git_ignore.rs

This is an implementation of `.gitignore` parsing and matching in Rust. Use this library if you want to check whether a given path would be excluded by a `.gitignore` file.

## Usage

The crate is called `git_ignore` and you can use it simply by depending on it via Cargo:

```toml
[dependencies.git_ignore]
git = "https://github.com/nathankleyn/git_ignore.git"
```

There is a [simple example binary](/src/bin/git_ignore.rs) which you can view to see how you might apply this library. A simple example is as follows:

```rust
// Create a file
let file = git_ignore::file::File::new(path_to_gitignore, None).unwrap();

// This returns a bool as to whether the file matches a rule in the .gitignore file.
file.matches(&path).unwrap();
```
