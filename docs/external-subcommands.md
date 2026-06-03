# External Subcommands

External subcommands are standalone scripts named `<main-command>-<sub-command>[.ext]` that live in the same directory as the main script. When enabled, they can be invoked as if they were built-in subcommands.

## Enable

Add `@meta external-subcommands` to the root of your argc script:

```sh
# @describe A CLI with external subcommands
# @meta external-subcommands

# @cmd A built-in subcommand
builtin() {
    echo "builtin cmd"
}

eval "$(argc --argc-eval "$0" "$@")"
```

## Convention

External subcommands follow the naming pattern `<main-command>-<sub-command>.sh`:

| Main script | External subcommand | Invocation |
|-------------|-------------------|------------|
| `prog.sh` | `prog-foo.sh` | `prog foo ...` |
| `prog.sh` | `prog-bar.sh` | `prog bar ...` |
| `Argcfile.sh` | `Argcfile-build.sh` | `argc build ...` |

- The extension (`.sh`) must match the main script
- External scripts are regular argc scripts — they must define their own args/flags and end with `eval "$(argc --argc-eval "$0" "$@")"`
- Internal subcommands take priority over external ones
- Only the immediate parent directory of the main script is scanned

## Example

### Main script: `external.sh`

```sh
# @describe A CLI with external subcommands
# @meta external-subcommands

# @cmd A built-in subcommand
builtin() {
    echo "builtin cmd"
}

eval "$(argc --argc-eval "$0" "$@")"
```

### External subcommand: `external-foo.sh`

```sh
# @describe An external subcommand "foo"

# @arg message!   Message to echo
main() {
    echo "external-foo: $argc_message"
}

eval "$(argc --argc-eval "$0" "$@")"
```

### External subcommand with flags: `external-bar.sh`

```sh
# @describe An external subcommand "bar" with options

# @flag   -v --verbose   Enable verbose output
# @arg    message!       Message to echo
main() {
    echo "external-bar: $argc_message"
}

eval "$(argc --argc-eval "$0" "$@")"
```

### Usage

```
$ external --help
Test external subcommands

USAGE: external <COMMAND>

COMMANDS:
  builtin  A built-in subcommand [aliases: b]

EXTERNAL COMMANDS:
  bar  An external subcommand "bar" with options
  foo  An external subcommand "foo"

$ external foo hello
external-foo: hello

$ external bar -v hey
verbose mode
external-bar: hey

$ external builtin hello
builtin: hello
```

## Completion

External subcommand names and their descriptions appear in tab-completion results, just like internal subcommands:

```
$ external <TAB>
builtin  bar  foo  help
$ external f<TAB>
$ external foo <TAB>
```

## How It Works

1. When `@meta external-subcommands` is set, argc scans the script's directory for files matching `<script-name>-*.sh`
2. Each matching file is registered as an external subcommand
3. During argument matching, if an arg doesn't match any internal subcommand, it's checked against the external list
4. If matched, the eval output generates `export ARGC_PARENT_ARGS=...` and `bash <external-script> <remaining-args>` as the last statement

## `ARGC_PARENT_ARGS`

When an external subcommand is dispatched, argc exports the `ARGC_PARENT_ARGS` environment variable containing the parent command and its flags (the args before the subcommand name). This allows external scripts to re-invoke the parent command with the original global options preserved.

For example, with `git --git-dir=<path> mytool ...`, the external subcommand `mytool` receives:

```sh
export ARGC_PARENT_ARGS="git --git-dir=<path>"
bash examples/git-mytool.sh ...
```

The external script can define a helper function to call back to the parent:

```sh
_git() {
  eval "$ARGC_PARENT_ARGS" "$@"
}

# Usage:
_git log --oneline
```

## Limitations

- **`--argc-build` does NOT support external subcommands** — the build feature compiles a single script into a standalone file and cannot include external scripts
- External scripts must be in the same directory as the main script (no recursive search)
- Only single-level external subcommands are supported (`prog foo` works, `prog foo bar` does not resolve `prog-foo-bar`)
- External subcommand names starting with `-` are ignored to avoid flag-name conflicts
