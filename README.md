# Argc

[![CI](https://github.com/sigoden/argc/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/argc/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/argc.svg)](https://crates.io/crates/argc)

Make beautiful bash cli with comments, also a command runner using bash.

## Usage

### Make beautiful cli with comments

![demo](https://user-images.githubusercontent.com/4012553/181145104-ee9220e2-ecfc-4f6c-8ad9-89c765ebe498.gif)

To write a command-line program with Argc, we only need to do two things:

1. Describe the options, parameters, and subcommands in comments
2. Call the following command to entrust Argc to process command line parameters for us


```sh
eval "$(argc --argc-eval $0 "$@")"
```

Argc will do the following for us:

1. Extract parameter definitions from comments
2. Parse command line arguments
3. If the parameter is abnormal, output error text or help information
4. If everything is normal, output the parsed parameter variable
5. If there is a subcommand, call the subcommand function

We can easily access the corresponding option or parameter through the variable `$argc_<name>`.

Try [examples/demo.sh](examples/demo.sh) your self.


The `@cmd`, `@arg`, `@option` is comment tag(fields marked with `@` in comments), argc generates parsing rules and help documentation based on comment tags .

See [docs/comment-tag.md](docs/comment-tag.md) for more details.


### A command runner using bash

When argc is executed without the `--argc-*` option, it will enter command runner mode. Argc will search for the `argcfile` file in the current project and its parent directory and execute it.

`argcfile` is to `argc` what `makefile` is to `make`．

![argcfile](https://user-images.githubusercontent.com/4012553/181147199-3c56e865-4057-48c6-b9d7-f8d594ffd49e.gif)

> Note: in windows, you need to install git to provide bash for argc

See [docs/command-runner.md](docs/command-runner.md) for more details


### Generate completion script

```
argc --argc-completion demo.sh
```

## Install

### With cargo

```
cargo install argc
```

### Binaries on macOS, Linux, Windows

Download from [Github Releases](https://github.com/sigoden/argc/releases), unzip and add argc to your $PATH.


## CLI

```
Bash cli utility - https://github.com/sigoden/argc

USAGE:
    argc [OPTIONS] [ARGS]

ARGS:
    <SCRIPT>          Specific script file
    <ARGUMENTS>...    Arguments passed to script file

OPTIONS:
        --argc-completion    Print bash completion script
        --argc-eval          Print code snippets for eval
        --argc-help          Print help information
        --argc-version       Print version information
```

## License

Copyright (c) 2022 argc-developers.

argc is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.