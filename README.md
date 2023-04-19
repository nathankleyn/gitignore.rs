# gitignore.rs

> ⛔️ **Important: This project is archived. Please don't build new software using this crate!**

## Project Status

I started this project to scratch a very specific itch I had to quickly check if some files were ignored — and didn't focus too much on correctness, just getting my specific issue solved at the time.

In 2017, when I was still working spuriously on this crate, I received some great feedback on issues that needed fixing for this to be useful in a wider set of scenarios. At the time in response, I committed to a fair bit of work to rework some of how this crate works internally, as I had made some poor-in-hindsight decisions on how to structure it — knowing what I know now about how it is used in the wild and the fun-fair of edge cases I didn't think about when I embarked on this journey, things would have been different, but you know what they say about hindsight!

I started working on this but the work got slower and further apart over time till basically I stopped doing much at all on this crate. Unfortunately, I had massively overestimated the time I would have available to maintain OSS stuff once I became a dad!

Now, 5 years later, I'm still seeing the odd issue crop up here and I feel it my duty to return one last time to say I am archiving this crate for the good of the community and will support you to transition to crates that actually work! It's really hard to let go of a problem, especially since it's still one I'd love to work on — but have to be honest about the time I have available to do so.

I am really sorry to those who depended on this project for their work and now have to change,  but thanks for trusting me and for providing awesome feedback — it was a privilege. To those who tread this path after me, thanks for working on something fun and challenging for us all and good luck!

I wish you all the best — and let me know if I can assist you in any way to migrate elsewhere by [dropping me a message using the contact details on my website](https://nathankleyn.com/).

## Alternatives

### The `ignore` Crate

The most plausible alternative to this project is of course [@BurntSushi's `ignore` crate](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore). This crate is what implements the ignore functionality for [ripgrep](https://github.com/BurntSushi/ripgrep), so as you can imagine Andrew has made it blazingly fast and it's been battle tested — you should use it! It also implements a lot of things correctly that this crate never did!

It doesn't yet serve neatly one of the main use-cases of this crate, which is to check whether a single file is ignored or not, as [it's more focussed on walking a file tree and returning unignored files in bulk](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore/examples). However you could glue the raw parts together that it exposes to do this yourself, depending on your use-case:

* [`Gitignore`](https://docs.rs/ignore/latest/ignore/gitignore/struct.Gitignore.html) — this struct has a [`matched` fn](https://docs.rs/ignore/latest/ignore/gitignore/struct.Gitignore.html#method.matched) and a [`matched_path_or_any_parents` fn](https://docs.rs/ignore/latest/ignore/gitignore/struct.Gitignore.html#method.matched_path_or_any_parents) which could be used to check files against a single ignore file, say at the root of a repo or provided dynamically.
* [`WalkBuilder`](https://docs.rs/ignore/latest/ignore/struct.WalkBuilder.html) — this struct can be used to build a FS walker which respects all of the `.gitignore` files. Most will will likely want this so that you ignore things based on the ignore files not just at the root of your repo, but in nested folders too, whilst also respecting any global ignore rules as well.

If anybody is interested in picking up the baton from here, I strongly encourage you to see if this functionality could either be added to the `ignore` crate via PRs or to create a simple wrapper around `ignore` as another crate.

### The `git2` Crate

You could also consider using the actual Rust wrapper around `libgit2` to achieve the same ends via the [`is_path_ignored` fn](https://docs.rs/git2/latest/git2/struct.Repository.html#method.is_path_ignored), although be mindful that this performs FS operations and thus expects the files to actually exist in a well structured actual Git repository — this may not fit your use case.


## Original README

If for some reason you need it, you can find [the original README for this repo still available](ORIGINAL-README.md).

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
