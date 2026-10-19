argc provides a set of built-in `--argc-*` commands for evaluating scripts, generating standalone builds, producing man pages and completions, exporting command definitions, running commands in parallel, and locating runtime paths. This page documents each tool. Run `argc --argc-help` to see the same list from the command line.

| Tool | Description |
|---|---|
| [`--argc-eval`](#--argc-eval) | Parse arguments and output shell code to eval |
| [`--argc-create`](#--argc-create) | Create a boilerplate argc script. |
| [`--argc-build`](#--argc-build) | Build a standalone script without argc dependency |
| [`--argc-mangen`](#--argc-mangen) | Generate man pages for a script |
| [`--argc-completions`](#--argc-completions) | Generate shell completion scripts |
| [`--argc-compgen`](#--argc-compgen) | Generate completion candidates. |
| [`--argc-export`](#--argc-export) | Export command definitions as JSON |
| [`--argc-parallel`](#--argc-parallel) | Run recipes in parallel |
| [`--argc-script-path`](#--argc-script-path) | Print the path of the current argcfile |
| [`--argc-shell-path`](#--argc-shell-path) | Print the path of the current shell |
| [`--argc-help`](#--argc-help) | Print help information |
| [`--argc-version`](#--argc-version) | Print version information |

## --argc-eval

Parses arguments for a script and writes the shell code that applies the result.

```sh
argc --argc-eval <SCRIPT> <ARGS>...
```

An argc-based script normally calls it through `eval`:

```sh
eval "$(argc --argc-eval "$0" "$@")"
```

The generated code sets `argc_*` variables, exports the environment, and calls the selected command function. If parsing fails, it emits shell code that prints the error and exits with status `1`, so the surrounding `eval` reports the failure.

This is the core integration point between argc and a script; you rarely need to run it by hand.

## --argc-create

Creates a boilerplate argc script, with one `@cmd` function per recipe name.

```sh
argc --argc-create [-|FILE] [RECIPES]...
```

```sh
argc --argc-create - build test
argc --argc-create mycli build test
argc --argc-create /tmp/mycli build test
```

- `-` is an alias for the default argc script name, normally `Argcfile.sh`. `ARGC_SCRIPT_NAME` can change that default.
- With no arguments, argc creates the default script name with no recipes.
- Otherwise the first argument is `-` or an output path, and the remaining arguments are recipe names. `argc --argc-create build test` creates `./build` containing a `test` recipe.
- The command refuses to overwrite an existing file.
- On Unix, the new script is marked executable.
- On Windows, creating a standalone script without a `.sh` suffix also creates a `.cmd` command shim so it can be invoked without the file extension.

## --argc-build

Builds a standalone script that embeds the parsing logic, so the result does not need `argc` at runtime.

```sh
argc --argc-build <SCRIPT> [OUTPATH]
```

Write to a file:

```sh
argc --argc-build ./mycli ./build/mycli
./build/mycli --help
```

Write to standard output:

```sh
argc --argc-build ./mycli > ./mycli-standalone
chmod +x ./mycli-standalone
```

`OUTPATH` may be a file or an existing directory. When it is a directory, argc writes the script using the source script name. Missing parent directories are created automatically, and a newly created output file is made executable.

The generated file contains a block marked with `# ARGC-BUILD { ... }`. Keep that block generated; manual edits will be overwritten on the next build.

### Limitations

- External subcommands are not supported in a built standalone script. The build step cannot include scripts that are discovered at runtime.
- The generated script targets Bash and uses the same script name for help and usage text unless `@meta binname` is set.
- The original functions and their behavior remain your responsibility; build only replaces the argument-parsing layer.

## --argc-mangen

Generates roff man pages for a script and all of its subcommands.

```sh
argc --argc-mangen <SCRIPT> <OUTDIR>
```

```sh
argc --argc-mangen ./mycli ./man
man man/mycli.1
```

For a script named `mycli`, this creates:

```text
man/mycli.1
man/mycli-subcommand.1
man/mycli-group-command.1
```

Nested commands use hyphenated names. `OUTDIR` is created if it does not exist, and each generated file is reported as it is written. External subcommands are not included. To change the manual section, set [`@meta man-section`](./meta.md#man-section):

```sh
# @meta man-section 8
```

```sh
argc --argc-mangen ./mycli ./man
# creates man/mycli.8
```

## --argc-completions

Generates a shell integration script that provides tab completion for argc-based commands.

```sh
argc --argc-completions <SHELL> [CMDS]...
```

`<SHELL>` is one of `bash`, `fish`, `nushell`, `powershell`, or `zsh`. `CMDS` lists the command names to register; `argc` itself is always included. If no commands are given, only argc's own completion is generated. Nushell uses a dynamic external completer, so `CMDS` is not used for Nushell.

```sh
source <(argc --argc-completions bash mycli)
source <(argc --argc-completions zsh mycli mycli2)
argc --argc-completions powershell mycli | Out-String | Invoke-Expression
argc --argc-completions fish mycli | source
argc --argc-completions nushell | save --append $nu.config-path
```

The script is written to standard output. Commands are resolved from `PATH` by default.

## --argc-compgen

Generates completion candidates for a command line. This is the completion engine used by the integration scripts produced by `--argc-completions`.

```sh
argc --argc-compgen <SHELL> <-|FILE> <CMD> <ARGS>...
```

- `<SHELL>` is the target shell: `bash`, `fish`, `nushell`, `powershell`, or `zsh`. `generic` is also accepted for custom completion tooling.
- `-` resolves the command by name.
- `FILE` reads the command definition from an explicit script path.
- `<CMD>` is the command name being completed.
- `<ARGS>...` is the words on the command line; the last word is the one being completed.

```sh
argc --argc-compgen bash - demo download --
argc --argc-compgen bash ./examples/demo demo download --
argc --argc-compgen bash ./examples/demo demo ""
```

The final word is the one being completed. An empty final word lists all candidates; a final `--` lists long options. Output encoding depends on `<SHELL>`.

See [Compgen](./compgen.md) for the candidate protocol, output formats, control lines, and completion patterns.

## --argc-export

Exports the parsed command definition as JSON.

```sh
argc --argc-export <SCRIPT> [OUTPATH]
```

```sh
argc --argc-export ./mycli > ./mycli.json
argc --argc-export ./mycli ./mycli.json
argc --argc-export ./mycli ./outdir
```

Without `OUTPATH`, the JSON is written to standard output. If `OUTPATH` is an existing directory, argc writes `<script-name>.json` inside it; otherwise it writes to the given file and creates missing parent directories.

The export is a tree of `command` objects. Each command contains `flag_options` (`flag_option_param[]`), `positionals` (`arg_param[]`), and `envs` (`env_param[]`) arrays, plus nested `subcommands`. See [Export Formats](./export-formats.md) for the complete schema, field tables, and JSON value shapes.

This format is intended for integrations such as documentation generators, completion tooling, and editors. Treat it as a machine-readable representation of the same information shown by `--help`.

## --argc-parallel

Runs several recipe invocations concurrently while preserving `argc_*` variables.

```sh
argc --argc-parallel <SCRIPT> <ARGS>...
```

Use `:::` to separate jobs:

```sh
argc --argc-parallel "$0" test ::: lint ::: format --check
```

Each segment is a separate job: the first word is the command or recipe, and the remaining words are its arguments. Jobs run in parallel, output is collected and printed in job order, and the command exits with status `1` if any job fails.

The environment variable `ARGC_PARALLEL=1` is set inside each job. This option requires an argc-based script that uses `--argc-eval`; it is not available for arbitrary shell scripts or standalone scripts produced by `--argc-build`.

## --argc-script-path

Prints the path of the argc script that argc would use from the current directory.

```sh
argc --argc-script-path
```

Argc searches the current directory and its parents for the default script name or the name set by `ARGC_SCRIPT_NAME`, and prints the first match. The default name is matched case-insensitively and the `.sh` suffix is optional. It exits with an error when no script is found. This is useful for scripts that need to locate the project root or inspect which Argcfile is active.

## --argc-shell-path

Prints the Bash executable argc uses to run scripts.

```sh
argc --argc-shell-path
```

The result respects `ARGC_SHELL_PATH` when it is set. This is useful for scripts that need to invoke the same shell as argc.

## --argc-help

Prints help for argc itself, including the complete list of `--argc-*` commands with their usage and examples.

```sh
argc --argc-help
```

## --argc-version

Prints the installed argc version.

```sh
argc --argc-version
# argc x.y.z
```
