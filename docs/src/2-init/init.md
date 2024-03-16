# Initialization
The `init` command should initialize a new git repository. From `git help init`:

```
NAME
       git-init - Create an empty Git repository or reinitialize an existing one

SYNOPSIS
       git init [-q | --quiet] [--bare] [--template=<template-directory>]
                 [--separate-git-dir <git-dir>] [--object-format=<format>]
                 [-b <branch-name> | --initial-branch=<branch-name>]
                 [--shared[=<permissions>]] [<directory>]


DESCRIPTION
       This command creates an empty Git repository - basically a .git directory with subdirectories for objects, refs/heads,
       refs/tags, and template files. An initial branch without any commits will be created (see the --initial-branch option below
       for its name).

       If the $GIT_DIR environment variable is set then it specifies a path to use instead of ./.git for the base of the repository.

       If the object storage directory is specified via the $GIT_OBJECT_DIRECTORY environment variable then the sha1 directories are
       created underneath - otherwise the default $GIT_DIR/objects directory is used.

       Running git init in an existing repository is safe. It will not overwrite things that are already there. The primary reason
       for rerunning git init is to pick up newly added templates (or to move the repository to another place if --separate-git-dir
       is given).
```

Basically, create the Git database in `.git` folder. See the implementation in Git source [here](https://github.com/git/git/blob/master/builtin/init-db.c#L73).
The Git book provides a good summary of the structure of the folder [here](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects).

Note that `init-db` is a synonym for `init`. From `get help init-db`:

```
NAME
       git-init-db - Creates an empty Git repository

SYNOPSIS
       git init-db [-q | --quiet] [--bare] [--template=<template-directory>] [--separate-git-dir <git-dir>] [--shared[=<permissions>]]


DESCRIPTION
       This is a synonym for git-init(1). Please refer to the documentation of that command.

```
