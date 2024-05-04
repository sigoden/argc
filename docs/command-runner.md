# Command runner

Argc is a also command runner built for those who love the efficiency and flexibility of Bash scripting.

This guide provides instructions on how to effectively use `argc` for this purpose.

## Create an Argcfile.sh 

Commands, called recipes, are stored in a file called argcfile .

Use `--argc-create` to quickly generate an `Argcfile.sh` for your project.

```sh
argc --argc-create build test
```

This creates a basic Argcfile.sh with sample `build` and `test` recipes.

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

A recipe is a regular shell function with a `@cmd` comment tag above it.

## Handle dependencies

Since recipe are functions, manage dependencies by calling them sequentially within other functions.

```sh
# @cmd
current() { before;
  echo current
after; }

# @cmd
before() {
  echo before
}

# @cmd
after() { 
  echo after
}
```

This example demonstrates how the `current` recipe calls both `before` and `after` recipes.

```
$ argc current
before
current
after
```

## Organize Recipes

Organize related recipes into groups for better readability.

```sh
# @cmd
test() { :; }
# @cmd
test-unit() { :; }
# @cmd
test-bin() { :; }
```

> Valid group formats include: `foo:bar` `foo.bar` `foo@bar`.

## Set default recipe

When invoked without a specific recipe, Argc displays available recipes.

```
$ argc
USAGE: Argcfile.sh <COMMAND>

COMMANDS:
  build
  test

```

Use `main` function to set a default recipe to run automatically.

```sh
# @cmd
build() { :; }

# @cmd
test() { :; }

main() {
  build
}
```

Another way is to use `@meta default-subcommand`

```sh
# @cmd
# @meta default-subcommand
build() { :; }

# @cmd
test() { :; }
```

Remember, you can always use `--help` for detailed help information.

## Aliases

Aliases allow recipes to be invoked on the command line with alternative names:

```sh
# @cmd
# @alias t
test() {
  echo test
}
```

Now you can run the `test` recipe using the alias `t`:
```
$ argc t
```

## Access positional arguments

Accessed through shell positional variables (`$1`, `$2`, `$@`, `$*` etc.).

```sh
# @cmd
build() {
  echo $1 $2
  echo $@
  echo $*
}
```

```
$ argc build foo bar
foo bar
foo bar
```

## Access Flag/option arguments

Define and use flags/options for more control.

```sh
# @cmd  A simple command
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
A simple command

USAGE: Argcfile.sh cmd [OPTIONS] [ARG]

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

## Load environment variables from dotenv file

Use `@meta dotenv` to load environment variables from a `.env` file.

```sh
# @meta dotenv                                    # Load .env
# @meta dotenv .env.local                         # Load .env.local
```

## Document and Validate environment variables

Define environment variables using `@env`.

```sh
# @env  FOO               A env var
# @env  BAR!              A required env var
# @env  MODE[dev|prod]    A env var with possible values
```

Argc automatically generates help information for environment variables.

By running `argc -h`, you'll see a list of variables with descriptions and any restrictions.

```
$ argc -h
USAGE: Argcfiles.sh

ENVIRONMENTS:
  FOO   A env var
  BAR*  A required env var
  MODE  A env var with possible values [possible values: dev, prod]
```

Argc also validates environment variables as per the `@env` definitions.

If `$BAR` is missing, Argc will report an error:

```
error: the following required environments were not provided:
  BAR$MO
```

For `$MODE`, which has predefined values, Argc verifies the input values and reports errors if they do not match:

```
error: invalid value `abc` for environment variable `MODE`
  [possible values: dev, prod]
```

## Align the project's rootdir

Argc automatically cds into the directory of the Argcfile.sh it finds in the parent hierarchy.

Project directory structure as follows:

```
$ tree /tmp/project

/tmp/project
├── Argcfile.sh
└── src
```

The code of build recipe as follows:

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

When running argc under the subdirectory other than project root,
`PWD` points to the project root, while `ARGC_PWD` points to the current directory.
