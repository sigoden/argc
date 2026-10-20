# External Subcommands

External subcommands let one script delegate to other argc scripts in the same directory. They are useful when a CLI grows too large for one file or when subcommands are installed separately.

## Enable

Add `@meta external-subcommands` to the root script:

`./external`
```sh
# @describe A CLI with external subcommands
# @meta external-subcommands

# @cmd A built-in subcommand
# @alias b
# @arg message!   Message to echo
builtin() {
    echo "builtin: $argc_message"
}

eval "$(argc --argc-eval "$0" "$@")"
```

Create external scripts whose name starts with the main script's name:

`./external-foo`
```sh
# @describe An external subcommand "foo"

# @arg message!   Message to echo
main() {
    echo "external-foo: $argc_message"
}

eval "$(argc --argc-eval "$0" "$@")"
```

`./external-bar`
```sh
# @describe An external subcommand "bar" that echoes a message

# @flag   -v --verbose   Enable verbose output
# @arg    message!       Message to echo
main() {
    echo "external-bar: $argc_message"
}

eval "$(argc --argc-eval "$0" "$@")"
```

## Example output

```text
$ external --help
A CLI with external subcommands

Usage: external <COMMAND>

Commands:
  builtin  A built-in subcommand [aliases: b]

External Commands:
  bar  An external subcommand "bar" with a verbose flag
  foo  An external subcommand "foo" that echoes a message
```

```sh
external foo hello
# external-foo: hello

external bar -v hey
# external-bar: hey

external builtin hello
# builtin: hello
```

## Naming convention

| Main script | External subcommand | Invocation |
| --- | --- | --- |
| `external` | `external-foo` | `external foo ...` |
| `external.sh` | `external-foo.sh` | `external foo ...` |
| `Argcfile.sh` | `Argcfile-build.sh` | `argc build ...` |

Rules:

- If the main script ends in `.sh`, discovered external scripts must also end in `.sh`.
- If the main script has no `.sh` suffix, files starting with `<main-script>-` are considered.
- External files must be in the same directory as the main script. Only that directory is scanned.
- Internal subcommands take priority over external ones with the same name.
- External subcommand names starting with `-` are ignored.
- External scripts are regular argc scripts and must parse their own arguments.


## `ARGC_PARENT_ARGS`

When an external subcommand is dispatched, argc exports `ARGC_PARENT_ARGS` containing the parent command and its arguments before the subcommand. This lets the external script call back into the parent while preserving global options:

```sh
_git() {
    eval "$ARGC_PARENT_ARGS" "$@"
}

_git log --oneline
```

For `git --git-dir=<path> mytool ...`, the external `mytool` script receives:

```sh
ARGC_PARENT_ARGS="git --git-dir=<path>"
```

## Completion

External command names, descriptions, and options are included in completion just like built-in commands.

## Limitations

- `--argc-build` does not support external subcommands. A standalone build cannot discover scripts at runtime.
- Only single-level external subcommands are supported. `prog foo` can resolve `prog-foo`, but `prog foo bar` does not resolve `prog-foo-bar`.
- External scripts must live next to the main script.
