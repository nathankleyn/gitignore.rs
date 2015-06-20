# gitignore.rs [![Build Status](https://travis-ci.org/nathankleyn/gitignore.rs.svg)](https://travis-ci.org/nathankleyn/gitignore.rs) [![Crates.io Version Of gitignore](https://img.shields.io/crates/v/gitignore.svg)](https://crates.io/crates/gitignore)

This is an implementation of `.gitignore` parsing and matching in Rust. Use this library if you want to check whether a given path would be excluded by a `.gitignore` file.

It currently builds on both nighly and stable versions of Rust.

## Usage

The crate is called `gitignore` and you can it is available via [crates.io](https://crates.io/gitignore):

```toml
[dependencies]
gitignore = "x.y.z"
```

You can also use the Git version directory simply by depending on it via Cargo:

```toml
[dependencies.gitignore]
git = "https://github.com/nathankleyn/gitignore.rs.git"
```

There are some useful [bundled binaries](/src/bin) which you can view to see how you might apply this library. [The Rust
docs are available to view](https://nathankleyn.com/gitignore.rs/gitignore/) as well.

## Examples

A simple example is as follows:

```rust
// Create a file
let file = gitignore::file::File::new(&path_to_gitignore).unwrap();

// This returns a bool as to whether the file matches a rule in the .gitignore file.
file.is_excluded(&path).unwrap();
```
