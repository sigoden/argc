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
    - [Customize shell path](#customize-shell-path)
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

To write a command-line program with Argc, we only need to do two things:

1. Describe the options, parameters, and subcommands in comment.
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

Comments matching `# @<name> [args...]` are comment tags. `argc` parses comment tags to get the definition of cli.

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
@alias name(,name)+

# @cmd
# @alias t,tst
test() {
}
```

### @option

Add a option to command.

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

Adds a flag to command.

```sh
@flag [short] <long> [help string]

# @flag     --foo       A flag
# @flag  -f --foo       A flag with short alias
```

### @arg

Adds a positional argument to command.

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
- @author: Sets cli's author,

```sh
# @describe A demo cli
# @version 2.17.1 
# @author nobody <nobody@example.com>
```

## Task automation tool

![task automation](https://user-images.githubusercontent.com/4012553/183369248-a898021b-bf5b-414b-b353-786522d85f13.png)

Write your tasks as subcommands, name your script file `argcfile`, then invoke `argc`.

`argc` will search for the argcfile file in the current project and its parent directory and execute it with `bash`.

`argc` runs `argcfile` like `make` runs `makefile`.

### Linux, MacOS, and Windows are supported

`argc` binaries are available in linux, macos, and windows.

`argc` require `bash`. `bash` is already builtin in macos/linux.
On windows, most developers already have git installed, `argc` use git bash by default.

GNU tools like `ls`, `rm`, `grep`, `find`, `sed`, `awk`... are also available, welcome to use them.

### Task is just function

Define a task by put put comment tag `@cmd` above a function.

```sh
# @cmd Build project
build() {
  echo Build...
}

# @cmd Run tests
test() {
  echo Test...
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

Shell positional parameters are available.

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

If you define a main function, invoke `argc` without any subcommand will call this function, otherwise `argc` will print help message then exit.

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

See snippets above, `argc` prints beautiful help messages.

`argc` will list all tasks and they're description and aliases.

You can also use `argc <task> -h` to print task's flags, options and positional arguments.


### Customize shell path

Argc needs `shell` to run `argcfile`.

Argc uses built-in bash in macos/linux, **uses git bash in windows**.

You can use environment variable `ARGC_SHELL` to custom shell path.

```
ARGC_SHELL=/usr/bin/bash
ARGC_SHELL="C:\\Program Files\\Git\\bin\\bash.exe"
```

### Customize script name

By default, argc searches for the `argcfile` file in the current project and its parent directory.

The `argcfile` can be named any of the following. Using a .sh suffix helps with editor syntax highlighting.

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
argc --argc-eval SCRIPT [ARGS ...]             Print code snippets for `eval $(argc --argc-eval "$0" "$@")`
argc --argc-compgen SCRIPT [ARGS ...]          Print commands and options as completion candidates 
argc --argc-argcfile                           Print argcfile path
argc --argc-help                               Print help information
argc --argc-version                            Print version information
```

## Shell Completion Scripts

[Command line completion scripts](completions) are available for most popular shells.

There are two types of completion scripts:

-  `argc.*` is for argc command, they will provide completions for tasks and task's options.
-  `script.*` is for scripts written with argc. 

Please refer to your shell's documentation for how to install them.

## License

Copyright (c) 2022 argc-developers.

argc is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.