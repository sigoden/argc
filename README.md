# Argc

[![CI](https://github.com/sigoden/argc/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/argc/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/argc.svg)](https://crates.io/crates/argc)

A bash cli framework, also a task management & automation tool.

- [Argc](#argc)
  - [Install](#install)
    - [With cargo](#with-cargo)
    - [Binaries on macOS, Linux, Windows](#binaries-on-macos-linux-windows)
    - [GitHub Actions](#github-actions)
  - [Bash cli framework](#bash-cli-framework)
    - [@cmd](#cmd)
    - [@alias](#alias)
    - [@option](#option)
    - [@flag](#flag)
    - [@arg](#arg)
    - [@help](#help)
    - [Meta Tag](#meta-tag)
  - [Task automation tool](#task-automation-tool)
    - [Linux, MacOS, and Windows are supported](#linux-macos-and-windows-are-supported)
    - [Task is just function](#task-is-just-function)
    - [Task accepts flags, options and positional arguments](#task-accepts-flags-options-and-positional-arguments)
    - [Task can have aliases](#task-can-have-aliases)
    - [Task can have pre and post dependencies](#task-can-have-pre-and-post-dependencies)
    - [Task can be semantically grouped](#task-can-be-semantically-grouped)
    - [The default task](#the-default-task)
    - [Informative tasks listings and beautiful help printings](#informative-tasks-listings-and-beautiful-help-printings)
    - [Customize shell](#customize-shell)
    - [Customize script name](#customize-script-name)
  - [Argc CLI Usage](#argc-cli-usage)
  - [Shell Completion Scripts](#shell-completion-scripts)
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

## Bash cli framework

![cli framework](https://user-images.githubusercontent.com/4012553/182050295-8f6f5fe1-b1b1-49ab-afb4-8d81dbb08ee2.gif)

To write a command-line program with argc, we only need to do two things:

1. Describe the options, parameters, and subcommands in comment.
2. Call the following command to entrust argc to process command line parameters for us

```sh
eval $(argc --argc-eval "$0" "$@")
```

Argc will do the following for us:

1. Extract flag/option/subcommand definitions from comments.
2. Parse command line arguments according to the definition.
3. If arguments are invalid, output error message or help information.
4. If everything is ok, output parsed variables.
5. If there is a subcommand, call the subcommand function.

We can directly use variables corresponding to flags/options/positional parameters.

`# @<name> [args...]` are comment tags. `argc` parses comment tags to get the definition of cli.

### @cmd

Adds a subcommand with help message.

```sh
@cmd [string]

# @cmd Upload a file
upload() {
}

# @cmd Download a file
download() {
}
```

### @alias

Sets multiple aliases to the subcommand.

```sh
@alias <name...>

# @cmd
# @alias t,tst
test() {
}
```

### @option

Add a option.

```sh
 @option [short] <long>[modifier] [notation] [help string]

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

Adds a flag.

```sh
@flag [short] <long> [help string]

# @flag     --foo       A flag
# @flag  -f --foo       A flag with short alias
```

### @arg

Adds a positional argument.

```sh
@arg <name>[modifier] [help string]

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

Generate help subcommand.

```sh
@help string

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

## Task automation tool

![task automation](https://user-images.githubusercontent.com/4012553/183369248-a898021b-bf5b-414b-b353-786522d85f13.png)

The argc script is often used for task automation. But using argc script for task automation has the following disadvantages:

- Not work in some shell such as powershell.
- No shell completions.
- Need to locate script file manually e.g. `../../script.sh`

The new version of argc has been optimized for task automation. It will automatically search for the `argcfile` file in the current project or its parent directory, then run it with `bash`.

`argc` runs `argcfile` like `make` runs `makefile`.

### Linux, MacOS, and Windows are supported

`argc` binaries are available in linux, macos, and windows.

`argc` require `bash` which already builtin in macos/linux. In windows, most developers already have git installed, `argc` automatically locate and use git bash.

GNU tools like `ls`, `rm`, `grep`, `find`, `sed`, `awk`... are also available, use them freely and confidently.

### Task is just function

Adds a task by putting `@cmd` above a function.

```sh
# @cmd Build project
build() {
  echo Build...
}

# @cmd Run tests
test() {
  echo Test...
}

helper() {
  :;
}

eval $(argc --argc-eval "$0" "$@")
```

```
$ argc
argcfile 

USAGE:
    argcfile <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    build    Build project
    test     Run tests 

$ argc build
Build...
```

### Task accepts flags, options and positional arguments

```sh
# @cmd     A simple task
# @flag    -f --flag      A flag
# @option  --opt          A option
# @arg     arg            A positional argument
cmd() {
  echo "flag: $argc_flag"
  echo "opt:  $argc_opt"
  echo "arg:  $argc_arg"
}
```

```
$ argc cmd -h
argcfile
A simple task

USAGE:
    argcfile cmd [OPTIONS] [ARG]

ARGS:
    <ARG>    A positional argument

OPTIONS:
    -f, --flag         A flag
    -h, --help         Print help information
        --opt <OPT>    A option

$ argc cmd -f --opt foo README.md
flag: 1
opt:  foo
arg:  README.md
```

*Shell variables are also available.*

```sh
# @cmd
build() {
  echo $2 $1
}
```

```
$ argc build foo bar
bar foo
```

### Task can have aliases

```sh
# @cmd
# @alias t,tst
test() {
  echo "Test..."
}
```

```
$ argc t
Test...
```

### Task can have pre and post dependencies

Tasks can depend on other tasks. Dependencies are resolved by calling functions.

```sh
# @cmd
bar() { foo;
  echo bar
baz; }

# @cmd
foo() {
  echo foo
}

# @cmd
baz() { 
  echo baz
}
```

```
$ argc bar
foo
bar
baz
```

### Task can be semantically grouped

Tasks can be grouped with `_`, `-`, `@`, `.`, `:`.

```sh
# @cmd
test@unit() {}
# @cmd
test@bin() {}

# @cmd
app.build() {}
# @cmd
app.test() {}
```

### The default task

If the `main` function exists, calling `argc` without any subcommands will call the function, otherwise print a help message and exit.

```sh
# @cmd
foo() {
  echo foo
}
# @cmd
bar() {
  echo baz
}
main() {
  foo
  bar
}
```

```
$ argc
foo
bar
```

### Informative tasks listings and beautiful help printings

See snippets above, `argc` prints a beautiful help message listing all tasks along with their descriptions and aliases.

You can also use `argc <task> -h` to print a help message containing the description of task flags, options and positional arguments.

### Customize shell

Argc uses built-in bash in macos/linux, uses git bash in windows.

You can use environment variable `ARGC_SHELL` to customize shell.

```
ARGC_SHELL=/usr/bin/bash
ARGC_SHELL="C:\\Program Files\\Git\\bin\\bash.exe"
```

### Customize script name

By default, argc searches for `argcfile` of the following:

- argcfile
- argcfile.sh
- Argcfile
- Argcfile.sh

You can use environment variable `ARGC_SCRIPT` to custom script name.

```
ARGC_SCRIPT=taskfile
```

## Argc CLI Usage

```
A bash cli framework, also a task management & automation tool - https://github.com/sigoden/argc

USAGE:
    argc --argc-eval SCRIPT [ARGS ...]             Parse arguments `eval $(argc --argc-eval "$0" "$@")`
    argc --argc-create [TASKS ...]                 Create a boilerplate argcfile
    argc --argc-compgen SCRIPT [ARGS ...]          Print commands and options as completion candidates 
    argc --argc-argcfile                           Print argcfile path
    argc --argc-help                               Print help information
    argc --argc-version                            Print version information
```

## Shell Completion Scripts

[Command line completion scripts](completions) are available for most popular shells.

There are two types of completion scripts:

-  `argc.*` is for argc command, they will provide completions for tasks and task parameters.
-  `script.*` is for scripts written with argc.

Please refer to your shell's documentation for how to install them.

## License

Copyright (c) 2022 argc-developers.

argc is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.