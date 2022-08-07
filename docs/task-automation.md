# Command automation

- [Command automation](#command-automation)
  - [Task is function](#task-is-function)
  - [Task arguments](#task-arguments)
  - [Task aliases](#task-aliases)
  - [Task dependencies](#task-dependencies)
  - [Default task](#default-task)
  - [Semantic group](#semantic-group)
  - [Customize shell path](#customize-shell-path)
  - [Customize script name](#customize-script-name)

## Task is function

Define a task by put put comment tag `@cmd` above a function.

```sh
# @cmd Build project
build() {
  echo Build...
}

# @cmd
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
    test     

$ argc build
Build...
```

## Task arguments

Task can have flags, options and positional argument.

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
  echo $1 $2
}
```

```
$ argc build foo bar
foo bar
```

## Task aliases

Tasks can be aliases with comment tag `@alias`.

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

## Task dependencies

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

## Default task

The default task is implemented through the main function

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

## Semantic group

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

## Customize shell path

Argc needs `bash` to run `argcfile`.

Argc uses built-in bash in macos/linux, **uses git bash in windows**.

You can use environment variable `ARGC_SHELL` to custom shell path.

```
ARGC_SHELL=/usr/bin/bash
ARGC_SHELL="C:\\Program Files\\Git\\bin\\bash.exe"
```

## Customize script name

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