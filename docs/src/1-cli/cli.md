# CLI
**Goal:** build out a CLI matching the `git` tool, with underlying commands stubbed out

The CLI for the `git` program:

```bash
usage: git [-v | --version] [-h | --help] [-C <path>] [-c <name>=<value>]
           [--exec-path[=<path>]] [--html-path] [--man-path] [--info-path]
           [-p | --paginate | -P | --no-pager] [--no-replace-objects] [--bare]
           [--git-dir=<path>] [--work-tree=<path>] [--namespace=<name>]
           [--super-prefix=<path>] [--config-env=<name>=<envvar>]
           <command> [<args>]
```

Checking crates.io, [clap](https://github.com/clap-rs/clap) seems to be the most popular CLI library. It has two main APIs, and [it's recommended is to use the "Derive" API](https://docs.rs/clap/latest/clap/_faq/index.html#when-should-i-use-the-builder-vs-derive-apis).

We can see the implementation in `git` [here](https://github.com/git/git/blob/master/git.c). At a high-level, it seems to construct commands based on the user input and then dispatch them to various underlying handlers; it may make sense to implement something like that at this point as well.
