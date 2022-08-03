# Command Runner

- [Command Runner](#command-runner)
  - [Defining and running task functions](#defining-and-running-task-functions)
  - [Task arguments](#task-arguments)
  - [Task aliases](#task-aliases)
  - [Task dependencies](#task-dependencies)
  - [Default task](#default-task)
  - [Semantic group](#semantic-group)
  - [Customize shell](#customize-shell)
  - [Customize script file](#customize-script-file)

## Defining and running task functions 

Define a task by put put comment `# @cmd` above a function.

```sh
# @cmd Build project
build() {
  echo Build...
}

# @cmd
test() {
  echo Test...
}

# @cmd
lint() {
  echo Lint...
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
    lint
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

Shell positional parameters also work.

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
  echo test
}
```

```
argc test
argc t
argc tst
```

## Task dependencies

Tasks can depend on other tasks. Just call functions, nothing is special.

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

Define main function to run default task.

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

Tasks can be grouped using `_`, `-`, `@`, `.`, `-`.

```sh
# @cmd
test() { :; }
# @cmd
test.unit() { :; }
# @cmd
test.bin() { :; }
```

## Customize shell

Argc needs `bash` to run `argcfile`.

> In Windows OS, argc will automatically locate `bash` that comes with git. 

Use environment variable `ARGC_SHELL` to custom shell

```
ARGC_SHELL=/usr/bin/bash
ARGC_SHELL="C:\\Program Files\\Git\\bin\\bash.exe"
```

## Customize script file

By default, argc searches for the `argcfile` file in the current project and its parent directory.

The `argcfile` can be named any of the following. Using a .sh suffix helps with editor syntax highlighting.

- argcfile
- argcfile.sh
- Argcfile
- Argcfile.sh

Use environment variable `ARGC_SCRIPT` to custom script file

```
ARGC_SCRIPT=taskfile
```
