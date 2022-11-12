# Argc

[![CI](https://github.com/sigoden/argc/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/argc/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/argc.svg)](https://crates.io/crates/argc)

Easily parse cli arguments in bash.

- [Argc](#argc)
  - [Install](#install)
    - [With cargo](#with-cargo)
    - [Binaries on macOS, Linux, Windows](#binaries-on-macos-linux-windows)
    - [GitHub Actions](#github-actions)
  - [Usage](#usage)
  - [Comment Tags](#comment-tags)
    - [@cmd](#cmd)
    - [@arg](#arg)
    - [@option](#option)
    - [@flag](#flag)
    - [@alias](#alias)
    - [@help](#help)
    - [Meta](#meta)
  - [Shell Completion](#shell-completion)
  - [Related Projects](#related-projects)
  - [License](#license)

## Install

### With cargo

```
cargo install argc
```

### Binaries on macOS, Linux, Windows

Download from [Github Releases](https://github.com/sigoden/argc/releases), unzip and add argc to your $PATH.

### GitHub Actions

[extractions/setup-crate](https://github.com/marketplace/actions/setup-crate) can be used to install just in a GitHub Actions workflow.

```yaml
- uses: extractions/setup-crate@v1
  with:
    owner: sigoden
    name: argc
```

## Usage

![demo](https://user-images.githubusercontent.com/4012553/192987706-2b07a356-640f-454b-aebe-88ba8fc07016.gif)

To write a command-line program with argc, we only need to do two things:

1. Describe the options, parameters, and subcommands in comments.
2. Call the following command to entrust argc to process command line arguments for us

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

`argc` parses cli definition from comment tags.

### @cmd

```
@cmd [string]
```

Define a subcommand

```sh
# @cmd Upload a file
upload() {
  echo Run upload
}

# @cmd Download a file
download() {
  echo Run download
}
```

```
USAGE: test.sh <COMMAND>

COMMANDS:
  upload    Upload a file
  download  Download a file
```

### @arg

```
@arg <name>[modifier] [help string]
```

Define a positional argument.

```sh
# @arg arg1            A positional argument
# @arg arg2!           A required positional argument
# @arg arg3*           A positional argument support multiple values
# @arg arg4+           A required positional argument support multiple values
# @arg arg5=a          A positional argument with default value
# @arg arg6[a|b]       A positional argument with choices
# @arg arg7[=a|b]      A positional argument with choices and default value
# @arg arg8![a|b]      A required positional argument with choices
```

```
USAGE: test.sh [ARG1] <ARG2> [ARG3]... <ARG4>... [ARG5] [ARG6] [ARG7] <ARG8>

ARGS:
  [ARG1]     A positional argument
  <ARG2>     A required positional argument
  [ARG3]...  A positional argument support multiple values
  <ARG4>...  A required positional argument support multiple values
  [ARG5]     A positional argument with default value [default: a]
  [ARG6]     A positional argument with choices [possible values: a, b]
  [ARG7]     A positional argument with choices and default value [default: a] [possible values: a, b]
  <ARG8>     A required positional argument with choices [possible values: a, b]
```


### @option

```
@option [short] <long>[modifier] [notation] [help string]
```

Define a option.

```sh
# @option    --opt1                 A option
# @option -a --opt2                 A option with short alias
# @option    --opt3 <PATH>          A option with notation
# @option    --opt4!                A required option
# @option    --opt5*                A option with multiple values
# @option    --opt6+                A required option with multiple values
# @option    --opt7=a               A option with default value
# @option    --opt8[a|b]            A option with choices
# @option    --opt9[=a|b]           A option with choices and default value
# @option    --opt10![a|b]          A required option with choices
# @option -b --opt11 <PATH>         A option with short alias and notation
```

```
USAGE: test.sh [OPTIONS] --opt4 <OPT4> --opt6 <OPT6>... --opt10 <OPT10>

OPTIONS:
      --opt1 <OPT1>       A option
  -a, --opt2 <OPT2>       A option with short alias
      --opt3 <PATH>       A option with notation
      --opt4 <OPT4>       A required option
      --opt5 [<OPT5>...]  A option with multiple values
      --opt6 <OPT6>...    A required option with multiple values
      --opt7 <OPT7>       A option with default value [default: a]
      --opt8 <OPT8>       A option with choices [possible values: a, b]
      --opt9 <OPT9>       A option with choices and default value [default: a] [possible values: a, b]
      --opt10 <OPT10>     A required option with choices [possible values: a, b]
  -b, --opt11 <PATH>      A option with short alias and notation
```

### @flag

```
@flag [short] <long> [help string]
```

Define a flag. A flag is an option of boolean type, and is always false by default (e.g. --verbose, --quiet, --all, --long, etc).


```sh
# @flag     --flag1       A flag
# @flag  -f --flag2       A flag with short alias
```

```
USAGE: test.sh [OPTIONS]

OPTIONS:
      --flag1  A flag
  -f, --flag2  A flag with short alias
```

### @alias

```
@alias <name...>
```

Add aliases

```sh
# @cmd Run tests
# @alias t,tst
test() {
  echo Run test
}
```

```
USAGE: test.sh <COMMAND>

COMMANDS:
  test  Run tests [aliases: t, tst]
```

### @help

```
@help string
```

Enable help subcommand.

```sh
# @help Show help

# @cmd Run test
test() {
  echo Run test
}
```

```
USAGE: test.sh <COMMAND>

COMMANDS:
  help  Show help
  foo   Run test
```
### Meta

- @describe: Sets the cli’s description. 
- @version: Sets cli's version.
- @author: Sets cli's author.

```sh
# @describe A demo cli
# @version 2.17.1 
# @author nobody <nobody@example.com>

# @cmd Run test
test() {
  echo Run test
}
```

```
test.sh 2.17.1
nobody <nobody@example.com>
A demo cli

USAGE: test.sh <COMMAND>

COMMANDS:
  test  Run test
```

## Shell Completion

[completion scripts](completions) are available for bash/zsh/powershell.

All argc scripts share the same completion function. To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

## Related Projects

- [runme](https://github.com/sigoden/runme): A task runner using argc.

## License

Copyright (c) 2022 argc-developers.

argc is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.
