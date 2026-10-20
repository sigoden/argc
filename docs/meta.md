`@meta` adds behavior to a command that is not tied to a single parameter. This page documents every metadata key supported by argc.

```sh
# @meta <name> [<value>]
```

| Syntax | Scope | Description |
| --- | --- | --- |
| [`@meta version <value>`](#version) | any | Set the version shown by `--version`. |
| [`@meta binname <value>`](#binname) | root | Override the program name in usage text. |
| [`@meta dotenv [path]...`](#dotenv) | root | Load one or more dotenv files, defaulting to `.env`. |
| [`@meta default-subcommand`](#default-subcommand) | subcmd | Make the current command the default. |
| [`@meta require-tools <tool>...`](#require-tools) | any | Require commands to be available before running. |
| [`@meta man-section <section>`](#man-section) | root | Set the man page section, defaulting to `1`. |
| [`@meta inherit-flag-options`](#inherit-flag-options) | root | Share root flags/options with subcommands. |
| [`@meta combine-shorts`](#combine-shorts) | root | Allow combined short flags such as `-rf`. |
| [`@meta external-subcommands`](#external-subcommands) | root | Enable external subcommand scripts. |
| [`@meta symbol <param>`](#symbol) | any | Define a symbolic parameter, such as `+toolchain`. |

## version

Sets the version reported by `-V`, `--version`.

```sh
# @meta version 1.0.0
```

```sh
prog --version
# prog 1.0.0
```

Declared at the root, it applies to the root command. Declared in a `@cmd` block, it applies to that subcommand. The value is required. When no version is declared, the root command reports `0.0.0` by default. A root version is not inherited by subcommands.

## binname

Overrides the program name used in usage text, help output, version output, and generated man pages. This key is root-only.

```sh
# @meta binname mycli

# @cmd Say hello
hello() { :; }
```

```text
Usage: mycli <COMMAND>
```

This is useful when a script is invoked through a different file name or a command shim. Without it, argc uses the script name. `binname` is also used as the prefix when deriving environment names for `$$` bindings.

## dotenv

Loads environment variables from one or more dotenv files before validating `@env` declarations and running the selected command. This key is root-only.

```sh
# @meta dotenv                          # load .env
# @meta dotenv .env .env.local          # load multiple files
# @meta dotenv $HOME/.env               # dotenv path can contain environment variables
```

Paths are separated by whitespace and can contain environment variables such as `$HOME`.

## default-subcommand

Makes the current command the default when its parent is invoked without a subcommand. Declare it inside a `@cmd` block.

```sh
# @cmd Upload a file
# @meta default-subcommand
upload() { :; }

# @cmd Download a file
download() { :; }
```

```sh
prog          # runs upload
prog download # runs download
```

The default command is shown as `[default]` in help output. A parent can have only one default subcommand, and the key cannot be used when the parent command has positional parameters.

## require-tools

Checks that external commands are available before running a command.

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

The tool list is whitespace-separated. Comma-separated lists are also accepted for compatibility. A root declaration applies to the whole script and is inherited by every subcommand; a command-level declaration adds tools for that command and its descendants. If any tool is missing, the command is not run.

## man-section

Sets the manual section used by `--argc-mangen`. This key is root-only.

```sh
# @meta man-section 8
```

```sh
argc --argc-mangen ./mycli ./man
# creates man/mycli.8
```

The default section is `1`. The value is used in both the generated file name and the man page header; argc does not restrict it to 1-8. See [`--argc-mangen`](./argc-tooling.md#--argc-mangen).

## inherit-flag-options

By default, flags and options defined at the root belong to the root command. Use `@meta inherit-flag-options` to make them available to every subcommand. This key is root-only.

```sh
# @meta inherit-flag-options
# @flag --user Connect to the user service manager
# @flag --no-pager Do not pipe output into a pager
# @option -t --type <TYPE> List units of a particular type

# @cmd Start one or more units
# @arg UNIT+ The unit files to start
start() { :; }

# @cmd Stop one or more units
# @arg UNIT+ The unit files to stop
stop() { :; }
```

```sh
prog --user start my-service
prog start --user my-service
```

Inherited parameters are shown in each subcommand's help and can appear before or after the subcommand name. If a subcommand declares a parameter with the same name, its own definition takes precedence.

## combine-shorts

Allows compact short flags such as `-rf`. This key is root-only.

```sh
# @meta combine-shorts
# @flag -r --recursive
# @flag -f --force
# @arg path*
```

```sh
prog -rf dir1 dir2
# equivalent to: prog -r -f dir1 dir2
```

Combined shorts work for short flags and options declared by the script.

## external-subcommands

Enables discovery of external subcommand scripts next to the main script. This key is root-only.

```sh
# @meta external-subcommands

# @cmd A built-in command
builtin() { :; }
```

With a main script named `mycli`, a file named `mycli-foo` is exposed as the external subcommand `foo`:

```sh
mycli foo
```

External scripts live in the same directory as the main script, and built-in subcommands take priority when names collide. See [External Subcommands](./external-subcommands.md) for naming rules, `ARGC_PARENT_ARGS`, completion, and limitations.

## symbol

Defines a custom prefix for a symbolic parameter. Symbolic parameters are useful for toolchain selection or argument-file syntax. The prefix is either `+` or `@`.

```sh
# @meta symbol @option-file
objdump @inspect.args
# argc_option_file=inspect.args
```
