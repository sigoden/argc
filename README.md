# Argc

[![CI](https://github.com/sigoden/argc/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/argc/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/argc.svg)](https://crates.io/crates/argc)

A bash cli framework, also a task management & automation tool.

## Bash cli framework

![cli framework](https://user-images.githubusercontent.com/4012553/182050295-8f6f5fe1-b1b1-49ab-afb4-8d81dbb08ee2.gif)

To write a command-line program with Argc, we only need to do two things:

1. Describe the options, parameters, and subcommands in comments
2. Call the following command to entrust Argc to process command line parameters for us


```sh
eval $(argc --argc-eval "$0" "$@")
```

Argc will do the following for us:

1. Extract flag/option/subcommand definitions from comments
2. Parse command line arguments
3. If arguments are invalid, output error message or help information
4. If everything is ok, output parsed variables.
5. If there is a subcommand, call the function related to the subcommand

We can easily access the corresponding flags/options/arguments through their associated variables.

The `@cmd`, `@arg`, `@option` are comment tags, see [docs/comment-tag](docs/comment-tag.md) for more details.

## Task automation tool

![task automation](https://user-images.githubusercontent.com/4012553/183274529-91cfe1ae-ddcf-490f-84d7-b5da01be1877.png)

Write your task as subcommand, name your script file `argcfile`, `argc` runs `argcfile` like `make` runs `makefile`.

Features:

- Use normal shell script, no need to learn another language/syntax.
- Cross-platform, use built-in bash in macos/linux, git-bash in windows.
- GNU tools( `ls`, `rm`, `grep`, `find`, `sed`, `awk` , ..) are available.
- Informative tasks listings and beautiful help printings.
- Completion scripts are available for bash/zsh/powershell.
- `argc` can be invoked from any subdirectory, not just the directory that contains the `argcfile`.
- Tasks can accept flags/options/positional arguments.
- Tasks can have pre and post dependencies.
- Tasks can be aliased.
- Tasks can be grouped.

See [docs/task-automation](docs/task-automation.md) for more details.

## Install

### With cargo

```
cargo install argc
```

### Binaries on macOS, Linux, Windows

Download from [Github Releases](https://github.com/sigoden/argc/releases), unzip and add argc to your $PATH.

### GitHub Actions

[extractions/setup-crate](https://github.com/marketplace/actions/setup-just) can be used to install just in a GitHub Actions workflow.

```yaml
- uses: extractions/setup-crate@v1
  with:
    owner: sigoden
    name: argc
```

## CLI

```
A bash cli framework, also a task management & automation tool - https://github.com/sigoden/argc

USAGE:
    argc --argc-eval SCRIPT [ARGS ...]             Print code snippets for `eval $(argc --argc-eval "$0" "$@")`
    argc --argc-compgen SCRIPT [ARGS ...]          Print commands and options as completion candidates 
    argc --argc-argcfile                           Print argcfile path
    argc --argc-help                               Print help information
    argc --argc-version                            Print version information
```

## License

Copyright (c) 2022 argc-developers.

argc is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.