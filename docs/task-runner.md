# Task runner

Argc can be used as a task runner.

Benefits:
  - Supports Linux/macOS/Windows
  - PowerShell shell auto-completion
  - No need to learn new technology, just Bash
  - Feel free to use GNU tools such as awk, sed, grep, find, etc.

## use boilerplate

```
$ argc --argc-create build test lint
Argcfile.sh has been successfully created.
```

Created boilerplate Argcfile.sh content as follows

```sh
#!/usr/bin/env bash

set -e

# @cmd
build() {
    echo To implement command: build
}

# @cmd
test() {
    echo To implement command: test
}

# @cmd
lint() {
    echo To implement command: lint
}

eval "$(argc --argc-eval "$0" "$@")"
```

## function is task

```sh
# @cmd
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

func() {
  echo "Plain function other than a task"
}

eval "$(argc --argc-eval "$0" "$@")"
```

```
$ argc
USAGE: Argcfile.sh <COMMAND>

COMMANDS:
  build
  test
  lint

```

## use positional variables

```sh
# @cmd
build() {
  echo $1 $2
  echo $@
}
```

```
$ argc build foo bar
foo bar
foo bar
```

## use `argc_*` variables

```sh
# @cmd  A simple task
# @flag -f --flag   A flag parameter
# @option -option   A option parameter
# @arg arg          A positional parameter
cmd() {
  echo "flag:    $argc_flag"
  echo "option:  $argc_option"
  echo "arg:     $argc_arg"
}
```

```
$ argc cmd -h
A simple task

USAGE: test cmd [OPTIONS] [ARG]

ARGS:
  [ARG]  A positional parameter

OPTIONS:
  -f, --flag             A flag parameter
      --option <OPTION>  A option parameter
  -h, --help             Print help

$ argc cmd -f --option foo README.md
flag:    1
option:  foo
arg:     README.md
```

## task alias

```sh
# @cmd
# @alias t
test() {
  echo test
}
```
```
$ argc t
```

## semantic group

Tasks can be grouped using symbols, common forms are `foo:bar` `foo.bar` `foo@bar`.

```sh
# @cmd
test() { :; }
# @cmd
test:unit() { :; }
# @cmd
test:bin() { :; }
```

## task dependencies

```sh
# @cmd
foo() {
  echo foo
}

# @cmd
bar() { foo;
  echo bar
baz; }

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

## default task

If the task name is not specified when calling, the `main` function is executed by default.

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

## locate project root

Argc automatically cds into the directory of the Argcfile.sh it finds in the parent hierarchy; 

Project directory structure as follows:

```
$ tree /tmp/project

/tmp/project
├── Argcfile.sh
└── src
```

The code of build task as follows:

```sh
# @cmd
build() {
    echo $PWD
    echo $ARGC_PWD
}
```

Run the build in the project dir:
```
$ argc build
/tmp/project
/tmp/project
```

Change directory (cd) into the subdirectory and run the build:
```
$ cd src && argc build
/tmp/project
/tmp/project/src
```

In the subdirectory, `argc` can correctly locate and execute the script. `PWD`
`PWD` points to the project root, while `ARGC_PWD` points to the current directory.