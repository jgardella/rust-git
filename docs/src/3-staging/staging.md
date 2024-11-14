# Staging
Implement commands for working with the local staging area:
- `add`
- `rm`
- `mv`
- `restore`
- `write-tree`

Running `git help add`:
```
NAME
       git-add - Add file contents to the index

SYNOPSIS
       git add [--verbose | -v] [--dry-run | -n] [--force | -f] [--interactive | -i] [--patch | -p]
                 [--edit | -e] [--[no-]all | --[no-]ignore-removal | [--update | -u]] [--sparse]
                 [--intent-to-add | -N] [--refresh] [--ignore-errors] [--ignore-missing] [--renormalize]
                 [--chmod=(+|-)x] [--pathspec-from-file=<file> [--pathspec-file-nul]]
                 [--] [<pathspec>...]
```

The C Git implementation can be found [here](https://github.com/git/git/blob/master/builtin/add.c). There is a lot of functionality here; to keep things simple, we'll probably omit a lot of features. Rather than doing a 1:1 implementation, it's probably better to implement the functionality from scratch; see the [Git Internals chapter of the Git book](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects) for more info.

There is likely a lot of overlap between the `add` porcelain command and the `hash-object` and `update-index` plumbing commands; it may make sense to implement them all now.
I thought `update-index` is a simpler version of `add`, but actually it's more of a super-powered version. So I will just implement a basic version of `add` for now.

With the index implemented for the `add` command, implementing `rm`, `mv`, and `restore` should be quite straightforward, just making changes to/using that index.

For now, I'll just implement `restore` with very basic functionality, to restore files in the working directory from the index. I will add further functionality once commits and branches are implemented.

The `write-tree` command adds Git tree objects to the Git object store based on the current index. Efficient implementation relies on the Git index being maintained in special sorted order: https://stackoverflow.com/questions/72128688/how-git-write-tree-works-internally