//! Support for loading, parsing and matching paths against the rules in a `.gitignore` file.
//!
//! This create has been specifically crafted to have no dependencies on Git itself - all you need
//! is a directory with a ``.gitignore` file in it, and a path you want to check is excluded by
//! some rule in the file.
//!
//! All of the patterns described in the [man page for the .gitignore format](https://www.kernel.org/pub/software/scm/git/docs/gitignore.html),
//! (specifically, in the ["Pattern Format" section](https://www.kernel.org/pub/software/scm/git/docs/gitignore.html#_pattern_format))
//! are implemented. This crate currently does not support auto-loading patterns from
//! `$GIT_DIR/info/exclude` or from the file specified by the Git configuration variable
//! `core.excludesFile` (the user excludes file); rather, it will only load patterns specified in
//! the `.gitignore` file in the given directory.

// FIXME: Remove this when we're done developing!
#![allow(dead_code)]

#![recursion_limit = "1024"]
#![cfg_attr(all(test, feature = "benchmarks"), feature(test))]

#[macro_use]
extern crate failure_derive;

#[cfg(all(test, feature = "benchmarks"))]
extern crate test;

mod errors {
  #[derive(Fail, Debug)]
  #[fail(display = "An IO error occured: {}", underlying)]
  struct IoError {
      underlying: ::std::io::Error
  }

  #[derive(Fail, Debug)]
  #[fail(display = "A pattern error occured: {}", underlying)]
  struct PatternError {
      underlying: ::globset::Error
  }
}

mod ignore_file;
mod repo;
mod ruleset;
