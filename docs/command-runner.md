# Command Runner

Argc is also a command runner for Bash. Instead of writing argument parsing for every project task, you put the tasks in an `Argcfile.sh`, and `argc` handles routing, help, validation, and completion.

This guide covers the runner-specific features. For the underlying tag reference, see [Comment Tags](./comment-tags.md).

## Create an Argcfile.sh

Recipes are stored in a file named `Argcfile.sh`:

```sh
argc --argc-create - build test
```

`-` is an alias for `Argcfile.sh`. The command creates a boilerplate file:

```sh
#!/usr/bin/env bash

set -e

# @cmd
build() {
    echo TODO build
}

# @cmd
test() {
    echo TODO test
}

# See more details at https://github.com/sigoden/argc
eval "$(argc --argc-eval "$0" "$@")"
```

Run a recipe with `argc`:

```sh
argc build
argc test
```

`argc` searches for `Argcfile.sh` in the current directory and its parents, so you can run it from a subdirectory. The name is matched case-insensitively, and the `.sh` suffix is optional, so `Argcfile`, `argcfile.sh`, and `ARGCFILE` are all recognized.

## Recipes

A recipe is a normal Bash function with a `@cmd` tag:

```sh
# @cmd Build the project
build() {
    cargo build
}
```

### Dependencies

Since recipes are ordinary Bash functions, manage dependencies by calling other recipes or helper functions in order. With `set -e` (the default in generated scripts), a failing step stops the recipe.

```sh
# @cmd
current() {
    before
    echo current
    after
}

before() {
    echo before
}

after() {
    echo after
}
```

```text
$ argc current
before
current
after
```

Functions without `@cmd` are not exposed as commands, which makes it easy to keep helper functions private.

## Organize recipes

Related recipes can share a prefix and a separator:

```sh
# @cmd
test:unit() { :; }

# @cmd
test:bin() { :; }
```

```sh
argc test:unit
argc test:bin
```

The same convention works with `.` or `@`, for example `test.unit` and `test@unit`. These are single command names with readable prefixes; use `::` instead when you need real nested commands (see [`@cmd`](./comment-tags.md#cmd)).

## Set a default recipe

Run `argc` without a subcommand to display the available recipes. If the root command has a `main` function or a default subcommand, that command runs instead (see below).

```text
$ argc
Usage: Argcfile.sh <COMMAND>

Commands:
  build
  test
```

There are two ways to run a recipe by default.

### Use `main`

```sh
# @cmd
build() { :; }

# @cmd
test() { :; }

main() {
    build
}
```

`main` runs when no subcommand is selected.

### Use `@meta default-subcommand`

```sh
# @cmd
# @meta default-subcommand
build() { :; }

# @cmd
test() { :; }
```

The marked recipe runs when no subcommand is selected and appears as `[default]` in help.

## Aliases

```sh
# @cmd
# @alias t
test() {
    echo test
}
```

```sh
argc t
```

Like command names in a regular argc script, recipe names containing `_` also get a hyphenated alias automatically.

## Access positional arguments

Recipes access positional arguments through the normal Bash variables:

```sh
# @cmd
build() {
    echo "$1"
    echo "$@"
    echo "$*"
}
```

```sh
argc build foo bar
```

argc also exposes the same values through `argc_*` variables when the arguments are declared with `@arg`:

```sh
# @cmd
# @arg target!
build() {
    echo "$argc_target"
}
```

## Access flags and options

Declare them above the recipe and read the generated variables:

```sh
# @cmd A simple command
# @flag -f --flag       A flag parameter
# @option --option <VALUE>  An option parameter
# @arg arg              A positional parameter
cmd() {
    echo "flag:   $argc_flag"
    echo "option: $argc_option"
    echo "arg:    $argc_arg"
}
```

```text
$ argc cmd -h
A simple command

Usage: Argcfile.sh cmd [OPTIONS] [ARG]

Args:
  [ARG]  A positional parameter

Options:
  -f, --flag             A flag parameter
      --option <VALUE>   An option parameter
  -h, --help             Print help

$ argc cmd -f --option foo README.md
flag:   1
option: foo
arg:    README.md
```

## Hooks

argc recognizes two optional hooks:

| Hook | When it runs |
| --- | --- |
| `_argc_before` | Before the selected command function, after variables are initialized |
| `_argc_after` | After the selected command function |

```sh
_argc_before() {
    echo "before"
}

_argc_after() {
    echo "after"
}

# @cmd
build() {
    echo "build"
}
```

```sh
argc build
```

```text
before
build
after
```

Hooks also run for the root command and for nested subcommands. They are useful for setup, cleanup, timing, and logging.

## Environment variables

Use `@meta dotenv` to load a `.env` file:

```sh
# @meta dotenv              # loads .env
# @meta dotenv .env.local   # or load .env.local
```

Use `@env` to document and validate environment variables:

```sh
# @env FOO               A normal variable
# @env BAR!              A required variable
# @env MODE[dev|prod]    A variable with possible values
```

```sh
argc -h
```

```text
Environments:
  FOO   A normal variable
  BAR*  A required variable
  MODE  A variable with possible values [possible values: dev, prod]
```

If a required variable is missing, argc reports an error. If a variable has choices, argc validates the value. Root-level `@env` declarations are inherited by subcommands automatically.

## Require external tools

Use `@meta require-tools` to check that commands are available before running a recipe:

```sh
# @meta require-tools awk sed

# @cmd
# @meta require-tools git
require-git() { :; }
```

```sh
argc require-git
# error: missing tools: git
```

The check runs before the recipe. A root-level declaration applies to the whole script; a command-level declaration applies to that command.

## Project root directory

When `argc` finds an `Argcfile.sh` in a parent directory, it changes the working directory to the directory containing the Argcfile:

```text
project/
├── Argcfile.sh
└── src/
```

```sh
# @cmd
build() {
    echo "$PWD"
    echo "$ARGC_PWD"
}
```

```text
$ cd /tmp/project/src
$ argc build
/tmp/project
/tmp/project/src
```

- `PWD` points to the project root (the Argcfile directory).
- `ARGC_PWD` points to the directory where `argc` was invoked.

This makes recipes independent of the caller's current directory.
