# Argc

[![CI](https://github.com/sigoden/argc/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/argc/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/argc.svg)](https://crates.io/crates/argc)

An elegant command-line options, arguments and sub-commands parser for bash.

![demo](https://user-images.githubusercontent.com/4012553/228990851-fee5649f-aa24-4297-a924-0d392e0a7400.gif)

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

To write a command-line program with argc, we only need to do two things:

1. Describe options, flags, positional parameters and subcommands in comments.
2. Insert `eval "$(argc --argc-eval "$0" "$@")"` into script to let argc to parse command line arguments.

Write `example.sh`

```sh
# @flag   --foo   A flag
# @option --bar   A option
# @option --baz*  A option with multiple values 

eval "$(argc --argc-eval "$0" "$@")"
echo foo: $argc_foo
echo bar: $argc_bar
echo baz: ${argc_baz[@]}
```

Run `./example.sh --foo --bar=value --baz a b c`, you can see argc successfully parses arguments and generate variables with `argc_` prefix.

```
foo: 1
bar: value
baz: a b c
```

Run `./example.sh -h`, argc wll print help information for you.

```
USAGE: example.sh [OPTIONS]

OPTIONS:
      --foo             A flag
      --bar <BAR>       A option
      --baz [<BAZ>...]  A option with multiple values
  -h, --help            Print help information
```

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
@arg <name>[modifier|default|modifer+choices] [zero-or-one value notation] [help-string]
```

Define a positional argument.

```sh
# @arg arg1            A positional argument
# @arg arg2!           A required positional argument
# @arg arg3 <PATH>     A positional argument with value notation
# @arg arg4*           A positional argument with multiple values
# @arg arg5+           A required positional argument with multiple values
# @arg arg6=a          A positional argument with default value
# @arg arg7=`_fn`      A positional argument with default value fn
# @arg arg8[a|b]       A positional argument with choices
# @arg arg9[`_fn`]     A positional argument with choices fn
# @arg arg10[=a|b]     A positional argument with choices and default value
# @arg arg11*[a|b]     A positional argument with choices and multiple values
```

### @option

```
@option [short] <long>[modifier|default|modifier+choices] [value-notation]... [help-string]
```

Define a option.

```sh
# @option    --opt1                 A option
# @option -a --opt2                 A option with short alias
# @option    --opt3 <PATH>          A option with value notation
# @option    --opt4!                A required option
# @option    --opt5*                A option with multiple values
# @option    --opt6+                A required option with multiple values
# @option    --opt7=a               A option with default value
# @option    --opt8=`_fn`           A option with default value fn
# @option    --opt9[a|b]            A option with choices
# @option    --opt10[`_fn`]         A option with choices fn
# @option    --opt11[=a|b]          A option with choices and default value
# @option    --opt12*[a|b]          A option with choices and multiple values
# @option    --opt13 <FILE> <FILE>  A option with value notation
```

### @flag

```
@flag [short] <long>[*] [help string]
```

Define a flag. A flag is an option of boolean type, and is always false by default (e.g. --verbose, --quiet, --all, --long, etc).


```sh
# @flag     --flag1       A flag
# @flag  -f --flag2       A flag with short alias
# @flag  -f --flag3*      A flag can occure multiple times
```

### @alias

```
@alias <name...>
```

Add aliases for subcommand.

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

### Value Notation

Value notation is used to describe value type of options and positional parameters.

```
# @option --target <FILE>
# @arg target <FILE>
```

Here are some value notation that will affect the shell completion.

- `<FILE>`: complete files in current directory
- `<DIR>`: complete directories in current directory
- `<PATH>`: complete files and directories in current directory

## Shell Completion

Ensure `argc` is added to [$PATH](https://en.wikipedia.org/wiki/PATH_(variable)). Then register the completion

```
# bash (~/.bashrc)
source <(argc --argc-completions bash mycmd1 mycmd2)

# fish (~/.config/fish/config.fish)
argc --argc-completions fish mycmd1 mycmd2 | source

# powershell ($env:USERPROFILE\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1)
argc --argc-completions powershell mycmd1 mycmd2 | Out-String | Invoke-Expression

# zsh (~/.zshrc)
source <(argc --argc-completions zsh mycmd1 mycmd2)
```

**Replace `mycmd1 mycmd2` with your argc script names**.

Argc can be used as multiple shell completion engine. see [argc-compteltions](https://github.com/sigoden/argc-completions)

## Argcscript

We can create a script called `Argcscript.sh`. If argc is run without the `--argc-*` options, argc will locate the file and run it.  

what is the benefit?

- Can enjoy a handy shell completion.
- Can be invoked in arbitrarily subdirectory, no need to locate script file each time.
- As a centralized entrypoint for executing the project's bash scripts.
- serves as a script for a task runner similar to how Makefile acts as make.

You can use `argc --argc-create` to quickly create boilerplate Argcscripts.

## Migrate

To migrate from v0 to v1, the only thing you need to do is:

Replace `eval "$(argc "$0" "$@")"` with `eval "$(argc --argc-eval "$0" "$@")"` in your script.

Otherwise you may encounter an error message like this when running the script:
```
Not found argcscript, try `argc --argc-help` to get help.
```

## Windows Only

Argc requires bash to run scripts. [git](https://git-scm.com/)'s built-in bash is good enough for argc.

If you want to use another bash, please specify it via `ARGC_SHELL` environment variable.

If you want to run the bash script directly, you can add the following configuration to Windows Registry.

```
New-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Classes\sh_auto_file\shell\open\command' -Name '(default)' -Value '"C:\Program Files\Git\bin\bash.exe" "%1" %*' -PropertyType String -Force
```

## License

Copyright (c) 2022 argc-developers.

argc is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.
