# Argc

[![CI](https://github.com/sigoden/argc/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/argc/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/argc.svg)](https://crates.io/crates/argc)

A bash cli framework, also a task runner.

## Usage

### Bash cli framework

![cli framework](https://user-images.githubusercontent.com/4012553/182018182-7a91f7b3-ab9e-41fd-89f8-a14e14391a7f.gif)

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

We can easily access the corresponding flags/options/arguements through their associated variables.

The `@cmd`, `@arg`, `@option` are comment tags, see [docs/comment-tag](docs/comment-tag.md) for more details.

### Task runner

![task runner](https://user-images.githubusercontent.com/4012553/182012460-8f4c6cea-1adc-43c9-9a2b-a8a1fff0879a.png)

Argc will enter the task runner mode if you do not activate its other modes with the `--argc-*` option.

What argc does in task runner mode are: locate bash, search for `argcfile` in the current project and its parent directory, then run argcfile with bash.

> `argcfile` is a plain shell script, you can run it via `bash argcfile`.

Argc is written in rust, is cross-platform, is a single executable file less than 1M without any dependencies.

Bash is already builtin in macos/linux. On Windows, most developers already have git installed, argc uses the bash that ships with git.

**Argc/argcfile is a cross-platform task runner solution.**

Use the bash you are most familiar with, no need to learn another language or set of syntax.

GNU tools( `ls`, `rm`, `grep`, `find`, `sed`, `awk` , etc..) are also avaiable, Don't worry about windows incompatibility.

Argc also provides `bash`, `zsh`, `powershell` completion scripts to prompt for tasks and options in `argcfile`, See [completions](completions)

See [docs/task-runner](docs/task-runner.md) for more details.

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
a bash cli framework, also a task runner. - https://github.com/sigoden/argc

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