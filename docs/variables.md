# Variables

Argc streamlines argument parsing in your shell scripts, allowing you to utilize variables seamlessly.

## Shell Variables

You can employ shell variables within your argc-based scripts just like you normally would in Bash. Argc doesn't interfere with their behavior.

```sh
# @cmd
cmd() {
  echo $1 $2  # Accessing positional arguments
  echo "$*"   # All arguments as a single string
  echo "$@"   # All arguments as separate strings
}
```

## Argc-generated variables

Each declared parameter creates a variable with the `argc_` prefix:

| Declaration | Variable | Notes |
| --- | --- | --- |
| `@option --foo` | `$argc_foo` | |
| `@option --dry-run` | `$argc_dry_run` | `-`, `.`, `:`, and `@` become `_` |
| `@option +foo` | `$argc_plus_foo` | Plus-prefixed options add `plus_` |
| `@option -X-*` | `${argc_X[...]}` | Associative array |
| `@option --tag*` | `${argc_tag[@]}` | Array |
| `@option --pair <LEFT> <RIGHT>` | `${argc_pair[@]}` | One occurrence holds multiple values |
| `@flag --verbose` | `$argc_verbose` | Unset when absent, `1` when present |
| `@flag --verbose*` | `$argc_verbose` | Number of occurrences |
| `@flag -@` | `$argc_0x40` | Punctuation-only name uses ASCII code |
| `@flag -@.` | `$argc_0x400x2e` | Each punctuation character is encoded |
| `@flag +@` | `$argc_plus_0x40` | Plus-prefixed punctuation-only name |
| `@arg input` | `$argc_input` | |
| `@arg files*` | `${argc_files[@]}` | Array |
| `@arg rest~` | `${argc_rest[@]}` | Array of all remaining arguments |
| `@meta symbol @file` | `$argc_file` | Symbolic parameter |

When a flag or option has several names, argc chooses a canonical name for the variable and error messages:

1. Prefer the last `--` name.
2. Otherwise prefer the last multi-character `-` name.
3. Otherwise prefer the last `+` name.
4. Otherwise use the last name.

Within the same priority, a name whose body contains only letters, digits, and `_` is preferred over one containing punctuation. For example, `@flag -h -?` uses `-h`, so the variable is `$argc_h`.

A body made entirely of punctuation is encoded with its ASCII codes. `-` is not part of the body and does not get encoded:

```sh
# @flag -@        # argc_0x40
# @flag -@.       # argc_0x400x2e
# @flag +@        # argc_plus_0x40
```

In a body that also contains normal characters, `-`, `.`, `:`, and `@` still become `_`; other punctuation is encoded. `--dry-run` therefore remains `$argc_dry_run`.

Optional parameters are normally unset when they are not provided. Use `${argc_name:-}` or a default value when the script runs with `set -u`.

## Built-in Variables

Argc also provides built-in variables that offer information about the parsing process:

*   **`argc__args`**: An array holding the program name and all command-line arguments.
*   **`argc__positionals`**: An array containing only the positional arguments. Subcommand names are not included.
*   **`argc__fn`**: The name of the command function that will be executed, when one is selected.

## Environment Variables

### User-defined

| Variable | Meaning |
| --- | --- |
| `ARGC_SHELL_PATH` | Path to the shell/Bash executable used by argc. The path must exist. |
| `ARGC_SCRIPT_NAME` | Override the default script filename (`Argcfile.sh`). When set without `.sh`, argc also checks the `.sh` variant. |

### Argc-injected

| Variable | When set | Meaning |
| --- | --- | --- |
| `ARGC_PWD` | `Argcfile.sh` (or an `ARGC_SCRIPT_NAME` script) runs, including via `--argc-eval` | Directory where `argc` was invoked. `PWD` points to the script's directory (the project root for `Argcfile.sh`). |
| `ARGC_OS` | During completion or choice-function evaluation | Operating system name, such as `linux`, `macos`, or `windows`. |
| `ARGC_VARS` | `--argc-eval` output | Internal base64-encoded variable snapshot. |

## Inspecting Variables

When debugging a script, you can dump the internal variables and the `ARGC_*` environment variables from any function:

```sh
(set -o posix; set) | grep argc_
printenv | grep ARGC_
```

- `(set -o posix; set) | grep argc_` lists the argc shell variables (e.g. `argc__args`, `argc__positionals`, `argc_oa`).
- `printenv | grep ARGC_` lists the exported `ARGC_*` environment variables (e.g. `ARGC_PWD`, `ARGC_OS`).
