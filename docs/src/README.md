# Introduction
**Goal:** re-implement git in Rust for fun and learning.

## References
Useful references related to Git, rust, etc.

### git
- [git-scm docs](https://git-scm.com/doc)
- [git source mirror](https://github.com/git/git)

### rust
- [Rust book](https://doc.rust-lang.org/stable/book/)

## High-level Plan
Goal: re-implement git in rust for fun and learning

We can run `git` to get an overview of what we have to implement. We can split it up into several general feature sets. Overall, a decent approach seems to be to implement these in the order that `git` presents them:

1. **CLI:** stub out all the commands for the CLI 
2. **Initialization:** command(s) for creating a git repo locally (`init`, etc)
3. **Staging:** command(s) for working with local staging area (`add`, `mv`, `restore`, `rm`, etc)
4. **Saving:** command(s) for saving changes (`commit`, etc)
5. **Branching:** command(s) for working with branches (`branch`, `switch`, `tag`, `reset`, etc)
6. **Inspect:** command(s) for inspecting local history and state (`status`, `log`, `diff`, etc)
7. **Merging:** command(s) for combining branches (`merge`, `rebase`, `reset`, etc)
8. **Collaboration:** command(s) for inspecting local history and state (`clone`, `fetch`, `pull`, `push`, `remote`, etc)
