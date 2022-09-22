# Argc

[![CI](https://github.com/sigoden/argc/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/argc/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/argc.svg)](https://crates.io/crates/argc)

A handy way to handle sh/bash cli parameters.

- [Argc](#argc)
  - [Install](#install)
    - [With cargo](#with-cargo)
    - [Binaries on macOS, Linux, Windows](#binaries-on-macos-linux-windows)
    - [GitHub Actions](#github-actions)
  - [Usage](#usage)
  - [Comment Tags](#comment-tags)
    - [@cmd](#cmd)
    - [@alias](#alias)
    - [@option](#option)
    - [@flag](#flag)
    - [@arg](#arg)
    - [@help](#help)
    - [Meta Tag](#meta-tag)
  - [License](#license)

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

## Usage

![cli framework](https://user-images.githubusercontent.com/4012553/182050295-8f6f5fe1-b1b1-49ab-afb4-8d81dbb08ee2.gif)

To write a command-line program with argc, we only need to do two things:

1. Describe the options, parameters, and subcommands in comment.
2. Call the following command to entrust argc to process command line parameters for us

```sh
eval $(argc "$0" "$@")
```

Argc will do the following for us:

1. Extract flag/option/subcommand definitions from comments.
2. Parse command line arguments according to the definition.
3. If arguments are invalid, output error message or help information.
4. If everything is ok, output parsed variables.
5. If there is a subcommand, call the subcommand function.

We can directly use variables corresponding to flags/options/positional parameters.

## Comment Tags

`argc` loads cli definition from comment tags.

### @cmd

```
@cmd [string]
```

Define a subcommand

```sh
# @cmd Upload a file
upload() {
}

# @cmd Download a file
download() {
}
```

### @alias

```
@alias <name...>
```

Add aliases

```sh
# @cmd
# @alias t,tst
test() {
}
```

### @option

```
@option [short] <long>[modifier] [notation] [help string]
```

Add a option.

```sh
 # @option    --foo                A option
 # @option -f --foo                A option with short alias
 # @option    --foo <PATH>         A option with notation
 # @option    --foo!               A required option
 # @option    --foo*               A option with multiple values
 # @option    --foo+               A required option with multiple values
 # @option    --foo=a              A option with default value
 # @option    --foo[a|b]           A option with choices
 # @option    --foo[=a|b]          A option with choices and default value
 # @option    --foo![a|b]          A required option with choices
 # @option -f --foo <PATH>         A option with short alias and notation
```

### @flag

```
@flag [short] <long> [help string]
```

Adds a flag.

```sh
# @flag     --foo       A flag
# @flag  -f --foo       A flag with short alias
```

### @arg

```
@arg <name>[modifier] [help string]
```

Adds a positional argument.

```sh
# @arg value            A positional argument
# @arg value!           A required positional argument
# @arg value*           A positional argument support multiple values
# @arg value+           A required positional argument support multiple values
# @arg value=a          A positional argument with default value
# @arg value[a|b]       A positional argument with choices
# @arg value[=a|b]      A positional argument with choices and default value
# @arg value![a|b]      A required positional argument with choices
```

### @help

```
@help string
```

Define help subcommand.

```sh
# @help Print help information
```
### Meta Tag

- @describe: Sets the cli’s description. 
- @version: Sets cli's version.
- @author: Sets cli's author.

```sh
# @describe A demo cli
# @version 2.17.1 
# @author nobody <nobody@example.com>
```

## License

Copyright (c) 2022 argc-developers.

argc is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.