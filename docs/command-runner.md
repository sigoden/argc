# Command Runner

  - [Turn function to task](#turn-function-to-task)
  - [Task aliases](#task-aliases)
  - [Task dependencies](#task-dependencies)
  - [Default action](#default-action)
  - [Semantic group](#semantic-group)
  - [Use positional variables](#use-positional-variables)
  - [Use argc variables](#use-argc-variables)
  - [Customize shell](#customize-shell)
  - [Customize script file](#customize-script-file)

## Turn function to task

put comment `# @cmd` on function to turn it into task.

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

helper() {
  echo Not a task
}

eval "$(argc --argc-eval $0 "$@")"
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
    help     Print this message or the help of the given subcommand(s)
    lint
    test

$ argc build
Build...
```

## Task aliases

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
argc ts
```

## Task dependencies

tasks can depend on other tasks. Just call functions, nothing is special.

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


## Default action

if run `argc` without specific task, the `main` function will be executed.

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

Tasks can be grouped using symbols, such as `foo:bar` `foo.bar` `foo@bar`.


```sh
# @cmd
test() { :; }
# @cmd
test:unit() { :; }
# @cmd
test:bin() { :; }
```

## Use positional variables

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

## Use argc variables 

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

## Customize shell

Use environment variable `ARGC_SHELL` to custom shell

```
ARGC_SHELL=/usr/bin/bash
ARGC_SHELL="C:\\Program Files\\Git\\bin\\bash.exe"
```

## Customize script file

By default, argc searches for the `argcfile` file in the current project and its parent directory.

Use environment variable `ARGC_SCRIPT` to custom script file

```
ARGC_SCRIPT=taskfile.sh
```
