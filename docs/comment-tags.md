# Comment Tags

Argc reads CLI definitions from comments inside a Bash script. A comment tag is a comment whose first non-space character after `#` is `@`:

```sh
# @describe A small command-line tool

# @cmd Say hello
# @option --name=world <NAME>  Name to greet
hello() {
    echo "Hello, $argc_name!"
}

eval "$(argc --argc-eval "$0" "$@")"
```

Tags apply to the command declared by the nearest enclosing `@cmd` block, or to the root command when they appear before any `@cmd`. The final `eval` line asks argc to parse the arguments, set the `argc_*` variables, and call the selected function.

This page is the complete reference for every supported tag. For the formal EBNF grammar, see [grammar.ebnf](./grammar.ebnf). For project tasks, see [Command Runner](./command-runner.md).

## Tag overview

| Tag | Purpose |
| --- | --- |
| [`@describe`](#describe) | Set the command description |
| [`@cmd`](#cmd) | Declare a subcommand |
| [`@alias`](#alias) | Add subcommand aliases |
| [`@arg`](#arg) | Declare a positional argument |
| [`@option`](#option) | Declare an option that takes a value |
| [`@flag`](#flag) | Declare a boolean flag |
| [`@env`](#env) | Declare an environment variable |
| [`@meta`](#meta) | Add metadata |

## @describe

Sets the description shown in help output.

**Format:** `@describe [description]`

```sh
# @describe A demo CLI
```

The first line is the short description. Extra comment lines after it become the long description:

```sh
# @describe A demo CLI
#
# This text appears as a longer description in help output.
```

## @cmd

Defines a subcommand. The function name is the command name.

**Format:** `@cmd [description]`

```sh
# @cmd Upload a file
upload() {
    echo Run upload
}

# @cmd Download a file
download() {
    echo Run download
}
```

```text
Usage: prog <COMMAND>

Commands:
  upload    Upload a file
  download  Download a file
```

Use `::` in the function name to create nested commands. Declare each parent group function before any child command that uses it:

```sh
# @cmd Manage repositories
repo() { :; }

# @cmd Manage autolink references
repo::autolink() { :; }

# @cmd Create an autolink reference
repo::autolink::create() { :; }

# @cmd Manage codespaces
codespace() { :; }

# @cmd Manage forwarded ports
codespace::ports() { :; }

# @cmd Change the visibility of forwarded ports
codespace::ports::visibility() { :; }
```

```sh
prog repo autolink create
prog codespace ports visibility
```

## @alias

Adds aliases to the current subcommand.

**Format:** `@alias name ("," name)*`

```sh
# @cmd Run tests
# @alias t,tst
test() {
    echo Run test
}
```

```sh
prog test
prog t
prog tst
```

```text
Commands:
  test  Run tests [aliases: t, tst]
```

Command names containing `_` automatically get a hyphenated alias, unless the name starts with `_`. For example, `run_tests` can also be invoked as `run-tests`.

## @arg

Declares a positional argument.

**Format:** `@arg name modifier? param-value? bind-env? notation? description?`

```sh
# @arg source! <PATH>        Source file
# @arg destination <PATH>    Destination file
# @arg files* <FILE>         Zero or more input files
# @arg paths+ <PATH>         One or more paths
# @arg command~              Remaining arguments, including flags
```

The generated variable is normally `$argc_<name>`. Multi-value arguments are arrays:

```sh
# @arg files*
cmd() {
    printf '%s\n' "${argc_files[@]}"
}
```

argc also populates the normal Bash positional values:

```sh
# @arg name
# @arg files*
cmd() {
    echo "$argc_name"
    echo "${argc_files[@]}"
    echo "$1"  # first positional value
    echo "$@"  # all positional values
}
```

### Defaults

```sh
# @arg mode=fast
# @arg ip=`_default_ip`
```

A default function is called when argc applies the parsed results, before the selected command runs. Its output becomes the default value.

### Choices

```sh
# @arg mode[dev|prod]        # value from a fixed set
# @arg mode[=dev|prod]       # fixed set with a default
# @arg mode[`_choice_mode`]  # choices from a function
# @arg mode[?`_choice_mode`] # choices for completion, no validation
```

Choices are validated during parsing:

```text
error: invalid value `staging` for `<MODE>`
  [possible values: dev, prod]
```

A function-based choice can be combined with `*` or `+` for multiple values.

With `[=a|b]`, the first value (`a`) is the default value.

### Value notation

Value notation controls the placeholder shown in help and enables path completion for path-aware names:

```sh
# @arg target <FILE>
# @arg output <PATH>
# @arg dir <DIR>
```

`FILE`, `PATH`, `DIR`, and `FOLDER` (and names containing those words) enable file or directory completion. `FOLDER` is an alias for `DIR`. See [Compgen](./compgen.md#path-completion) for the exact rules.

A positional argument accepts at most one notation and does not support notation modifiers such as `*`, `+`, or `?`. To accept multiple values, use a parameter modifier: `@arg files* <FILE>`.

### Environment binding

```sh
# @arg target $TARGET
# @arg port $$
```

See [Bind environment](#bind-environment) for details.

## @option

Declares an option that takes a value.

**Format:** `@option (short ws)? long modifier? param-value? bind-env? notations* description?`

```sh
# @option -o --output <FILE>          Write output to FILE
# @option -j <N>                      Number of parallel jobs
# @option --config <FILE>             Path to configuration file
# @option --include* <PATTERN>        Include matching files
# @option --color[auto|always|never]  Colorize output
# @option --token $GITHUB_TOKEN       GitHub access token
```

There are also less common option styles, such as `--foo=abc` (where `--foo=` appears in completions), prefix options like `-Xfoo -Xbar`, and plus-prefixed options like `+foo`. All of these are supported. See [Uncommon Option Styles](./advanced.md#uncommon-option-styles).

The generated variable is normally `$argc_<name>`. Multi-value options are arrays; plus-prefixed and symbolic parameters use different names. See [Variables](./variables.md#argc-generated-variables) for the full mapping.

```sh
# @option --tag*
cmd() {
    printf '%s\n' "${argc_tag[@]}"
}
```

### Defaults

```sh
# @option --tries=3
# @option --cpus=`_default_cpus`
```

```sh
prog cmd                  # argc_tries=3
prog cmd --tries 5        # argc_tries=5
```

### Choices

```sh
# @option --mode[dev|prod]
# @option --mode[=dev|prod]
# @option --mode[`_choice_mode`]
# @option --mode[?`_choice_mode`]
```

`?` disables validation but still uses the function for completion.

### Notation

Value notation uses angle brackets to show the value placeholder in help and usage text:

```sh
# @option --output <FILE>
# @option --argc-mangen <FILE> <OUTDIR>
# @option --argc-compgen <SHELL> <-|FILE> <CMD> <ARGS+>
```

`@option` may use multiple notations, such as `<LEFT> <OUTDIR>`, to consume several values per occurrence. `FILE`, `PATH`, `DIR`, and `FOLDER` (and names containing those words) enable path completion. See [Compgen](./compgen.md#path-completion) for the exact path-completion rules.

### Notation modifiers

Only `@option` supports notation modifiers. The final notation may carry one of these modifiers:

| Modifier | Meaning |
| --- | --- |
| `*` | Zero or more values |
| `+` | One or more values |
| `?` | Zero or one value |

Options may use multiple notations, such as `<LEFT> <RIGHT>`; only the final notation may have a modifier.

```sh
# @option --pair <LEFT> <RIGHT>
# @option --files <FILE+>
```

```sh
prog cmd --pair a b
prog cmd --files a.txt b.txt
```

The option value is an array:

```sh
# @option --pair <LEFT> <RIGHT>
cmd() {
    echo "${argc_pair[0]} ${argc_pair[1]}"
}
```

### Environment binding

```sh
# @option --token $API_TOKEN
# @option --port $$
```

See [Bind environment](#bind-environment) for details.

## @flag

Declares a boolean flag. A flag does not accept a value.

**Format:** `@flag (short ws)? long "*"? bind-env? description?`

```sh
# @flag -f --force       Overwrite existing files
# @flag -q               Suppress output
# @flag --verbose        Show detailed output
# @flag --trace*         Repeatable trace flag
```

A flag is `1` when present and unset when absent:

```sh
# @flag --force
cmd() {
    echo "${argc_force:-0}"
}
```

For repeatable flags, the variable contains the number of occurrences:

```sh
# @flag --verbose*
prog cmd --verbose --verbose
# argc_verbose=2
```

### Environment binding

```sh
# @flag --verbose $VERBOSE
# @flag --debug $$
```

See [Bind environment](#bind-environment) for details.

## @env

Declares and validates an environment variable.

**Format:** `@env name "!"? param-value? description?`

```sh
# @env CI                  Detected in CI environments
# @env GITHUB_TOKEN!       Token for GitHub API access
# @env LOG_LEVEL=info      Default log level
# @env APP_ENV[dev|prod]   Deployment environment
# @env NODE_ENV[=dev|prod] Node environment
```

Environment names use the same name syntax as other parameters; uppercase names are conventional. Bindings such as `$NAME` and `$$` still require uppercase letters and underscores. `@env` declarations on the root command are inherited by subcommands.

If a required environment variable is missing, argc reports an error before running the command. If a variable has choices, argc validates its value. Environment variables are exported to the command.


## @meta

Adds metadata that changes command behavior.

**Format:** `@meta name [value]`

| Syntax | Scope | Purpose |
| --- | --- | --- |
| [`@meta version <value>`](./meta.md#version) | Any | Set the version shown by `--version` |
| [`@meta binname <value>`](./meta.md#binname) | Root | Override the program name in usage text |
| [`@meta dotenv [path]...`](./meta.md#dotenv) | Root | Load a dotenv file |
| [`@meta default-subcommand`](./meta.md#default-subcommand) | Subcommand | Select a default command |
| [`@meta require-tools <tool>...`](./meta.md#require-tools) | Any | Require external commands before running |
| [`@meta man-section <section>`](./meta.md#man-section) | Root | Set the man page section |
| [`@meta inherit-flag-options`](./meta.md#inherit-flag-options) | Root | Share parent flags/options with subcommands |
| [`@meta combine-shorts`](./meta.md#combine-shorts) | Root | Allow `-rf` style combined short flags |
| [`@meta external-subcommands`](./meta.md#external-subcommands) | Root | Discover external subcommand scripts |
| [`@meta symbol <param>`](./meta.md#symbol) | Any | Define a custom symbolic parameter prefix |

Unknown metadata keys are preserved in `--argc-export` output, but do not change argc behavior. See [Metadata](./meta.md) for every key, its scope, and examples.

## Shared parameter syntax

The following parts are shared by the parameter tags `@arg`, `@option`, `@flag`, and `@env`, although not every tag supports every part. Their complete EBNF definitions are in [grammar.ebnf](./grammar.ebnf).

### Modifiers

| Modifier | Meaning |
| --- | --- |
| `!` | Required |
| `*` | May be specified multiple times, or accepts multiple values |
| `+` | Required and may be specified multiple times |
| `~` | Capture all remaining arguments |
| `,` | Split values on a separator |

### Values

Ways to provide a parameter value:

| Form | Meaning |
| --- | --- |
| `=value` | Default value |
| `` =`fn` `` | Default value from a function |
| `[a\|b]` | Choices |
| `[=a\|b]` | Choices with a default |
| `` [`fn`] `` | Choices from a function |
| `` [?`fn`] `` | Choices from a function without validation |

Values may be quoted with single or double quotes when they contain spaces or special characters.

### Value functions

Default and choice functions are ordinary shell functions. They run in the context of the script and can use `argc_*` variables.

```sh
# @option --profile[`_choice_profile`]
cmd() {
    echo "$argc_profile"
}

_choice_profile() {
    echo dev
    echo staging
    echo prod
}
```

For completion, a function may print one candidate per line. A tab separates the value from an optional description:

```sh
_choice_profile() {
    printf 'dev\tDevelopment\n'
    printf 'prod\tProduction\n'
}
```

Default functions return a single value:

```sh
_default_profile() {
    echo dev
}
```

Without `?`, values returned by a choice function are validated. With `?`, the function is used only for completion.

### Value notation

Value notation controls the placeholder shown in help and usage text. It applies to `@arg` and `@option`; `@flag` and `@env` do not accept notation:

```sh
# @arg target <FILE>
# @option --file <FILE>
```

`FILE`, `PATH`, `DIR`, and `FOLDER` are recognized for file and directory completion:

- `FILE`: complete file paths.
- `PATH`: complete file paths; `PATH` and `FILE` behave the same.
- `DIR`: complete directories only.
- `FOLDER`: alias for `DIR`; completes directories only.

Names containing these words also work, such as `<CONFIG_FILE>` and `<OUTPUT_DIR>`. See [Compgen](./compgen.md#path-completion) for extension filters and fallback behavior.

### Bind environment

A parameter can use an environment variable as a fallback value:

```sh
# @option --token $API_TOKEN
# @arg target $TARGET
```

- `$NAME` uses `NAME`.
- `$$` derives the name from the root command name (or `@meta binname`) and the parameter name.

For a script named `serve`, `--port $$` binds `SERVE_PORT`. This is based on the root name, not the subcommand containing the parameter.

Explicit command-line values take precedence over environment values.

## Help and version

argc accepts `-h`, `--help`, and `-help` automatically, and supports the `help` subcommand:

```sh
prog --help
prog help
prog help cmd
prog cmd --help
```

When a version is declared, the command also accepts `-V`, `--version`, and `-version`:

```sh
# @meta version 1.0.0

# @cmd
# @meta version 2.0.0
cmd() { :; }
```

```sh
prog --version      # prog 1.0.0
prog cmd --version  # prog-cmd 2.0.0
```

A version declared on the root command is not inherited by subcommands. Declare `@meta version` inside a `@cmd` block to give that subcommand its own version.

The root command accepts `--version` even without an explicit version and reports `0.0.0` by default.

Help output includes descriptions, defaults, possible values, environment bindings, and value notation. When a command has no options or arguments, the help output can be very small.
