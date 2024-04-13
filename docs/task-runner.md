# Task runner

Argc is an ideal task runner for automating complex tasks, especially for users familiar with Bash.

## Why Choose Argc?

- **Leverage Bash Skills:** No need to learn a new language, perfect for Bash aficionados.
- **GNU Toolset Integration:** Utilize familiar tools like awk, sed, grep, and find within your tasks.
- **Environment variables Management**: Load dotenv, document, and validate environment variables effortlessly.
- **Powerful Shell Autocompletion:** Enjoy autocomplete suggestions for enhanced productivity.
- **Cross-Platform Compatibility::** Works seamlessly across Linux, macOS, Windows, and BSD systems.

## Defining Tasks

A task is a regular shell function with a `# @cmd` comment tag above it.

### Create an Argcfile.sh 

Use `--argc-create` to quickly generate an Argcfile.sh for your project.

```sh
argc --argc-create build test
```

This creates a basic Argcfile.sh with sample `build` and `test` tasks.

Here's what Argcfile.sh looks like:

## Handling dependencies

Since task are functions, manage dependencies by calling them sequentially within other functions.

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

This example demonstrates how the `current` task calls both `before` and `after` tasks.

```
$ argc current
before
current
after
```

### Organizing Tasks

Organize related tasks into groups for better readability.

```sh
# @cmd
test() { :; }
# @cmd
test-unit() { :; }
# @cmd
test-bin() { :; }
```

> Valid group formats include: `foo:bar` `foo.bar` `foo@bar`.

## Running Tasks

### Default task recipe

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

Now you can run the `test` task using the alias `t`:
```
$ argc t
```

### Positional arguments

Accessed through shell positional variables (`$1`, `$2`, `$@`, `$*` etc.).

```sh
# @cmd
build() {
  echo $1 $2
  echo "$@"
}
```

```
$ argc build foo bar
foo bar
foo bar
```

### Flag/option arguments

Define and use flags and options for more control.

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

## Environment variable

### Loading dotenv

Use `@meta dotenv` to load environment variables from a `.env` file.

```sh
# @meta dotenv                                    # Load .env
# @meta dotenv .env.local                         # Load .env.local
```

### Documentation and Validation

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

When running argc under the subdirectory other than project root,
`PWD` points to the project root, while `ARGC_PWD` points to the current directory.
