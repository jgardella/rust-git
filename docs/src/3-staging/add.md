# Staging
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

The C Git implementation can be found [here](https://github.com/git/git/blob/master/builtin/add.c). There is a lot of functionality here; to keep things simple, we'll probably omit a lot of features.
