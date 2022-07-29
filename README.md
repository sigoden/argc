# Argc

[![CI](https://github.com/sigoden/argc/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/argc/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/argc.svg)](https://crates.io/crates/argc)

Make beautiful bash cli with comments, also is a cross-platform bash command runner.

## Usage

### Make beautiful cli with comments

![cli](https://user-images.githubusercontent.com/4012553/181766283-8c91e672-30ff-40e5-b03a-a910e8923958.gif)

To write a command-line program with Argc, we only need to do two things:

1. Describe the options, parameters, and subcommands in comments
2. Call the following command to entrust Argc to process command line parameters for us


```sh
eval $(argc --argc-eval "$0" "$@")
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

### command runner

Argc will enter the command runner mode if you do not activate its other modes with the `--argc-*` option.

What argc does in command runner mode is: locate bash, search for `argcfile` in the current project and its parent directory, then run argcfile with bash.

> `argcfile` is a plain shell script, you can run it through `bash argcfile`.

Argc is written in rust, It is cross-platform. It is a single executable file less than 1M without any dependencies. you just download it and put it into $PATH directory to install it.

Bash is already builtin in macos/linux. On Windows, most developers already have git installed, argc uses the bash that ships with git.

So argc/argcfile is a cross-platform command runner solution.  

Use the bash you are most familiar with, no need to learn another language or set of syntax.

You can also freely use GNU tools like `ls`, `rm`, `grep`, `find`, `sed`, `awk`, etc. Don't worry about windows incompatibility.

![command runner](https://user-images.githubusercontent.com/4012553/181766750-c18e5aab-5308-4bd0-8c42-865d48519371.png)

See [docs/command-runner.md](docs/command-runner.md) for more details


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
Bash cli utility - https://github.com/sigoden/argc

USAGE:
    argc [OPTIONS]

OPTIONS:
        --argc-complete <shell>    Print complete script [possible values: bash, zsh, powershell]
        --argc-eval                Print code snippets for `eval $(argc --argc-eval "$0" "$@")`
        --argc-help                Print help information
        --argc-version             Print version information
```

### Print argc help

```
argc --argc-help
```

### Generate bash completion for a script

```
argc --argc-completion demo.sh
```

## License

Copyright (c) 2022 argc-developers.

argc is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.