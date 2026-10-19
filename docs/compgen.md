# Compgen

`argc --argc-compgen` is the completion engine behind argc's shell integrations. It parses a command line and prints completion candidates. This page covers command lookup, path completion, choice functions, the candidate protocol, control lines, and completion patterns.

## Command lookup

The generated integration scripts normally resolve commands from `PATH`. You can also keep completion scripts outside `PATH` and point argc at them with `ARGC_COMPLETIONS_PATH`. It is a list of directories separated by `:` (`;` on Windows). For a command `mycli`, argc looks for:

```text
$ARGC_COMPLETIONS_PATH/mycli.sh
$ARGC_COMPLETIONS_PATH/mycli/<subcommand>.sh
```

`ARGC_COMPLETIONS_PATH` is searched before `PATH`.

For a one-off check, call the completion engine directly:

```sh
argc --argc-compgen bash ./mycli mycli --
argc --argc-compgen bash - mycli --
```

- `./mycli` tells argc to read the script from an explicit path.
- `-` tells argc to resolve the command from `ARGC_COMPLETIONS_PATH` or `PATH`.

The final word is the word being completed. Use an empty final word to list all candidates, or only `--` to list long options:

```sh
argc --argc-compgen bash ./mycli mycli ""
argc --argc-compgen bash ./mycli mycli --
```

## Path completion

### Fallback behavior

When a command has no positional parameters and no other candidates are produced, argc falls back to path completion. A final word beginning with `./` or `.\` also triggers path completion.

### Value notation

`FILE`, `PATH`, `DIR`, and `FOLDER` are recognized in value notation (case-insensitive):

- `FILE`: complete file paths. Directories remain in the candidate list so you can navigate into them.
- `PATH`: same behavior as `FILE`; use it when the value is a path rather than a single file.
- `DIR`: complete directories only.
- `FOLDER`: alias for `DIR`; completes directories only.

Argc also matches these words inside longer notation names, so `<CONFIG_FILE>`, `<SOURCE_PATH>`, and `<OUTPUT_DIR>` work as expected. `ARG` and `ANY` behave like `FILE`.

```sh
# @arg input <FILE>
# @arg output <PATH>
# @arg cache-dir <DIR>
```

To filter files by extension, append `:` and a comma-separated extension list:

```sh
# @option --config <FILE:.json,.js>
```

### Runtime controls

A choice function can request path completion dynamically:

```sh
_choice_config() {
    printf '__argc_value=file:.toml\n'
}
```

- `__argc_value=path` and `__argc_value=file` both complete paths. Files and directories are offered; `file` only expresses that the value is intended to be a file.
- `__argc_value=dir` completes directories only.
- Extension filters apply to `path` and `file`, not to `dir`. Separate multiple extensions with commas; the leading dot is optional.
- `__argc_value` adds path candidates; it does not discard candidates that the function has already printed.

`__argc_cd` changes the base directory for relative paths. It does not change the text inserted into the command line:

```sh
# @option --root <DIR>
# @arg file[`_choice_file`]

_choice_file() {
    printf '__argc_cd=%s\n' "${argc_root:-.}"
    printf '__argc_value=path\n'
}
```

Path completion handles `~`, `./`, `../`, and `file://` values where the target shell supports them. On Windows, the shell integration uses the native path separator. See [Path controls](#path-controls) for the control-line protocol.

## Choice functions

Choice functions provide completion values:

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

The function prints one candidate per line. To add a description, separate the value and description with a tab:

```sh
_choice_profile() {
    printf 'dev\tDevelopment environment\n'
    printf 'prod\tProduction environment\n'
}
```

The same function can be used with `@arg`, `@option`, and symbolic parameters.

Choice functions run as ordinary shell functions and can read `argc_*` variables parsed from the current command line:

```sh
# @option --profile
# @arg target[`_choice_target`]

_choice_target() {
    case "${argc_profile:-dev}" in
        dev) echo localhost ;;
        prod) echo api.example.com ;;
    esac
}
```

### Completion without validation

A leading `?` means "use this function for completion, but do not validate the value":

```sh
# @option --branch[?`_choice_branch`]
```

Without `?`, argc rejects values that are not returned by the function.

## Completion context

### Environment variables

argc exports these variables while a choice function runs for completion:

| Variable | Meaning |
| --- | --- |
| `ARGC_COMPGEN` | `1`. The variable is not set during normal command runs. |
| `ARGC_CWORD` | The part of the current word after the completion split point. For `--option=value`, it is `value`. |
| `ARGC_LAST_ARG` | The full last argument, including an `--option=value` prefix and any quotes. |
| `ARGC_OS` | Operating system name, such as `linux`, `macos`, or `windows`. |

`ARGC_CWORD` and `ARGC_LAST_ARG` describe different views of the same word:

| Command line | `ARGC_CWORD` | `ARGC_LAST_ARG` |
| --- | --- | --- |
| `git --git-dir=git` | `git` | `--git-dir=git` |
| `prog --option=value` | `value` | `--option=value` |
| `prog --output build/` | `build/` | `build/` |

When the current word starts with a quote, argc moves the quote into the generated prefix and removes it from the completion filter, so `ARGC_CWORD` contains the unquoted remainder.

### Argc variables

argc parses the command line before calling a choice function, so the usual `argc_*` parameter variables are available. Optional parameters that have not been provided are unset. Use `${argc_name:-}` when running with `set -u`.

General built-ins such as `argc__args` and `argc__positionals` are documented in [Variables](./variables.md#built-in-variables). During completion, `argc__positionals` includes the current word, which may be empty.

### Completion-only built-ins

These built-in variables are mainly useful during completion:

| Variable | Meaning |
| --- | --- |
| `argc__cmd_arg_index` | Index of the selected command name within `argc__args`. |
| `argc__cmd_fn` | Name of the selected command function. |
| `argc__dash` | Number of positional arguments before the first `--`. It is set only when `--` is present and while a parameter function runs. |
| `argc__option` | Generated variable name of the option currently being completed, such as `argc_oa`. |

`argc__option` is useful when one choice function is shared by several options:

```sh
_choice_value() {
    case "$argc__option" in
        argc_color) echo red green blue ;;
        argc_size) echo small medium large ;;
    esac
}
```

## Candidate protocol

A choice function prints one candidate per line. The general shape of a line is:

```text
value[\0][\t/color:<name>[,<style>]][\t<description>]
```

- `value` is the text inserted into the command line.
- A trailing `\0` suppresses the space after the value.
- A `/color:` field sets the candidate color.
- The last field is the description.

A candidate normally has to start with the current completion filter. Use `__argc_filter` to change that filter, as shown below.

### Suppressing the trailing space

Put `\0` at the end of the value field:

```sh
_choice_dir() {
    printf 'src/\0\n'
    printf 'docs/\0\tDocumentation\n'
}
```

This is useful for directories and for values that are followed by another part, such as `key=`, `user:`, or `name,`. Do not put `\0` after the description.

### Colors

Add a tab-separated color field before the description:

```sh
_choice_build() {
    printf 'debug\t/color:yellow\tDebug build\n'
    printf 'release\t/color:green,bold\tRelease build\n'
}
```

Supported color names are `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, and `default`. The optional style is `regular` or `bold`. zsh and PowerShell render colors; `generic` includes the color field in its output; bash, fish, and nushell ignore it.

Set `NO_COLOR=1` to disable colors in zsh and PowerShell candidates.

### Descriptions

Descriptions are enabled by default. Set `ARGC_COMPGEN_DESCRIPTION=1` or `true` to force them on; any other value disables them. `ARGC_COMPGEN_DESCRIPTION_MAX_WIDTH` controls how much of a description is shown before it is truncated. The default is `80`.

### Shell output

Each shell receives candidates in a different shape:

| Shell | Shape |
| --- | --- |
| `bash` | `value` or `value (description)` |
| `fish` | `value<TAB>description` |
| `nushell` | `value<TAB>description` |
| `zsh` | `value<TAB>display:description<TAB>value<TAB>color` |
| `powershell` | `value<TAB>space<TAB>display<TAB>description<TAB>color` |
| `generic` | Control lines first, then `value[\0]<TAB>/color:<name>[,<style>][<TAB>description]` |

Custom tooling should use the same shell name as the integration that will consume the output. `generic` is intended for tooling; it prints control lines before candidates.

### Duplicates

Candidates are keyed by value. If the same value is produced more than once, the first non-empty description wins, and nospace is enabled if any occurrence requests it.

## Parameter-driven completion

Most completion behavior comes from the parameter declaration itself. The full syntax is in [Comment Tags](./comment-tags.md); this section highlights the parts that affect completion.

### Multiple values and separators

A `*` or `+` modifier allows several values. A separator after the modifier splits each value into parts:

```sh
# @option --tags*[`_choice_tag`]        # repeatable, one value per occurrence
# @option --tags*,[`_choice_tag`]       # comma-separated values
# @option --tags*|[`_choice_tag`]       # pipe-separated values
# @arg val+[`_choice_val`]              # one or more values
```

The current word is split at the last separator, the preceding parts are kept in the prefix, values already present in the current word are removed, and no trailing space is added after the separator.

### Prefixed options

A prefixed option such as `-X-*` consumes the rest of the argument as its value:

```sh
# @option -X-*[`_choice_option`]

_choice_option() {
    printf 'batch\tDisable background compilation\n'
    printf 'bootclasspath=\0\tSet the boot class path\n'
}
```

For `-X<tab>`, `ARGC_CWORD` contains the text after `-X`. Values are recorded in the associative array `argc_X`.

### Symbolic parameters

A symbol declared with [`@meta symbol`](./meta.md#symbol) completes the text after its prefix; the prefix is kept in the inserted value.

```sh
# @meta symbol +toolchain[`_choice_toolchain`]
# @cmd Build the package
build() { :; }

_choice_toolchain() {
    printf '%s\n' stable beta nightly
}
```

```sh
prog +<tab>              # stable, beta, nightly
prog +n<tab>             # +nightly
prog +nightly build      # argc_toolchain=nightly
```

Inside the choice function, `ARGC_CWORD` contains the text after the prefix (`n` for `+n`), and `ARGC_LAST_ARG` contains the full word (`+n`). The function supplies candidates only; it does not validate the value.

Without a choice function, use a path-aware name:

```sh
# @meta symbol @file
# @flag --disassemble
```

```sh
prog @<tab>              # files and directories
prog @in<tab>            # paths starting with "in"
```

## Control lines

A choice function prints a line-oriented protocol. Most lines are candidates. A line whose first field starts with `__argc_` is a control line: an instruction to argc, not a candidate. In shell integrations, control lines are consumed and never displayed. They control how argc splits the current word, which part of the word is matched, what text is inserted around the selected value, and whether path candidates are added.

argc generates a few internal control lines from the parameter declaration. A choice function must not print these:

| Line | Meaning |
| --- | --- |
| `__argc_fn=FN` | Run choice function `FN`. |
| `__argc_multi=CHAR` | Split a declared multi-value parameter at `CHAR`. |

These are the control lines a choice function can emit:

| Line | Effect |
| --- | --- |
| `__argc_prefix=PREFIX` | Set the text kept before the candidate. |
| `__argc_filter=FILTER` | Set the text a candidate must start with. |
| `__argc_suffix=SUFFIX` | Append `SUFFIX` after the candidate. |
| `__argc_value=KIND` | Add path candidates (`path`, `file`, or `dir`). |
| `__argc_cd=DIR` | Use `DIR` as the base for relative path candidates. |

argc also emits `__argc_value` from declarations such as `<FILE>` and `<DIR>`; the format is the same.

### Prefix and filter

The word being completed is split into two parts:

```text
word     = prefix + filter
inserted = prefix + candidate + suffix
```

The prefix is kept on the command line. The filter only selects candidates: a candidate is used when it starts with the filter. argc performs the splits it can infer from the declaration:

| Word | Prefix | Filter |
| --- | --- | --- |
| `--define=mode=d` | `--define=` | `mode=d` |
| `-Xfoo` | `-X` | `foo` |
| `+nightly` | `+` | `nightly` |
| `--tags=a,b` | `--tags=a,` | `b` |

A choice function can move the split point with `__argc_prefix` and `__argc_filter`. The common case is a value made of several parts.

For a path such as `git add src/ma` (with a modified `main.rs`), the function emits only the next part as a candidate:

```text
$ git add src/ma<TAB>
__argc_prefix=src/
__argc_filter=ma
main.rs
```

argc inserts `src/main.rs`. The candidate must not be `src/main.rs`: with the filter `ma`, that value would not match.

Key-value values use the same split. For `scp -o KEY=VALUE`, the completion first offers keys:

```text
$ scp -o <TAB>
__argc_filter=
AddKeysToAgent=\0
AddressFamily=\0
BatchMode=\0
...
```

The trailing `\0` suppresses the space after `=`, so the user can continue with the value. After `=` is typed, the function splits the key from the value:

```text
$ scp -o AddKeysToAgent=<TAB>
__argc_prefix=AddKeysToAgent=
__argc_filter=
yes
ask
confirm
no
```

The candidates are only the values. argc inserts the prefix, so selecting `yes` produces `AddKeysToAgent=yes`.

A complete choice function for this pattern looks like:

```sh
# @option -o[`_choice_option`]

_choice_option() {
    local word=$ARGC_CWORD
    if [[ $word == *=* ]]; then
        local key=${word%%=*}
        printf '__argc_prefix=%s=\n' "$key"
        printf '__argc_filter=%s\n' "${word#*=}"
        case $key in
            AddKeysToAgent) printf 'yes\nask\nconfirm\nno\n' ;;
            CertificateFile) printf '__argc_value=file\n' ;;
        esac
    else
        printf '__argc_filter=%s\n' "$word"
        printf 'AddKeysToAgent=\0\n'
        printf 'CertificateFile=\0\n'
    fi
}
```

The `AddKeysToAgent` branch prints ordinary values. The `CertificateFile` branch prints a path control line instead, so the same key-value split switches to file completion after `=`.

`__argc_prefix` is relative to the prefix argc has already split off. For a separate option value such as `scp -o AddKeysToAgent=`, that prefix is empty, so the function emits the whole `AddKeysToAgent=`. For an option written as `--define=mode=d`, argc puts `--define=` in the prefix, so the function only adds `mode=`:

```sh
printf '__argc_prefix=mode=\n'
printf '__argc_filter=d\n'
```

Do not include `--define=` again.

`__argc_filter` changes only the matching part. It does not change `ARGC_CWORD`; that variable still contains the word from when the function started. Save it before changing the filter:

```sh
local word=$ARGC_CWORD
```

Before a separator, the prefix is usually already correct, so only `__argc_filter` is needed. After the separator, set both controls. Within a choice function's output, control lines are processed in order. `__argc_filter` and `__argc_suffix` affect only the candidates printed after them; they do not retroactively change candidates that have already been printed. For the other controls, the last value wins, and `__argc_prefix` is used as the prefix for the whole candidate set.

### Suffix

`__argc_suffix` appends text to candidates. It is mainly useful for path completion, where argc generates the candidates and a choice function cannot add the suffix line by line.

For example, `minikube mount` completes a host path followed by a colon:

```text
$ minikube mount ./<TAB>
__argc_filter=./
__argc_suffix=:\0
__argc_value=path
```

A file candidate becomes `./file:`. Directories keep their trailing `/` so the user can continue into them, and do not get the suffix. The `\0` on the control line suppresses the space after the suffix.

For ordinary candidates, a literal suffix in the candidate line is often simpler. For `chown user:group`, user candidates end with `:`:

```sh
printf 'root:\0\n'
printf 'daemon:\0\n'
```

### Path controls

`__argc_value` adds path candidates to the current completion:

- `path` and `file` both offer files and directories. Use `file` when the value is intended to be a file; it does not exclude directories.
- `dir` offers directories only.
- Extension filters apply to `path` and `file`, not to `dir`:

```text
__argc_value=path:.json,jsonc
__argc_value=file:.toml,.yaml
```

`__argc_value` does not discard candidates already printed. For example, `systemctl enable` offers both unit names and file paths:

```text
__argc_value=path
nginx.service
docker.service
...
```

The shell integration adds matching paths to this list.

`__argc_cd` changes only the directory used to resolve relative paths. It does not change the text inserted into the command line. This is useful when a command resolves paths relative to a project file. For example, `go mod edit` resolves the replacement path in `-replace=old=new` relative to the directory containing `go.mod`:

```text
__argc_prefix=example.com/old=
__argc_filter=
__argc_cd=/path/to/module
__argc_value=path
```

In `generic` output, path controls are emitted instead of expanded paths. Real shell integrations expand them. See [Path completion](#path-completion) for the notation forms and extension syntax.

### Internal control lines

`__argc_fn` and `__argc_multi` are generated by argc from the parameter declaration:

- `__argc_fn=FN` tells argc which choice function to run.
- `__argc_multi=CHAR` splits a declared multi-value parameter at its last `CHAR`, keeps the preceding parts in the prefix, and removes values that have already been selected.

For example, with:

```sh
# @option --tags*,
```

the command line `cmd --tags=a,b` is split into prefix `--tags=a,` and filter `b` before the choice function runs, and `a` is not offered again. When the separator is not declared on the parameter, a choice function can do the same split itself with `__argc_prefix` and `__argc_filter`; for example, `find . -type f,` offers the remaining types after the comma.

Do not print either internal line from a choice function. Also avoid starting ordinary candidate values with `__argc_`, because argc consumes them as control lines.

`generic` output shows the public control lines and path controls, but the internal `__argc_fn` and `__argc_multi` signals are already consumed.


## Debugging

### Inspect raw candidates

Use `generic` to see the control lines and candidates before shell-specific formatting is applied:

```sh
argc --argc-compgen generic ./mycli mycli ""
```

`generic` does not expand paths. Instead, it emits `__argc_value=...` for path completion. Control lines come first, followed by candidates. The internal `__argc_fn` and `__argc_multi` signals are already consumed and do not appear.

Use the target shell name to see exactly what the shell integration receives:

```sh
argc --argc-compgen bash ./mycli mycli ""
argc --argc-compgen zsh ./mycli mycli ""
```

### Run a choice function directly

The `___internal___` escape hatch runs a function with the current command-line context:

```sh
./mycli ___internal___ _choice_profile mycli --profile dev
```

See [Advanced Topics](./advanced.md#escape-hatch-___internal___) for details.
