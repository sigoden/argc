# Argc

[![CI](https://github.com/sigoden/argc/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/argc/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/argc.svg)](https://crates.io/crates/argc)

Easily create feature-rich CLIs in bash.

![demo](https://user-images.githubusercontent.com/4012553/228990851-fee5649f-aa24-4297-a924-0d392e0a7400.gif)

Argc lets you define your CLI through comments and focus on your specific code, without worrying about command line argument parsing, usage texts, error messages and other functions that are usually handled by a framework in any other programming language.

## Features

- Parsing user's command line and extracting:
  - Positional arguments (optional, required, default value, choices, comma-seperated, multiple values),
  - Option arguments (optional, required, default value, choices, comma-seperated, multiple values, repeated),
  - Flag arguments (repeated),
  - Sub-commands (alias, nested).
- Rendering usage texts, showing flags, options, positional arguments and sub-commands.
- Validating the arguments, printing error messages if the command line is invalid.
- Generating a single standalone bashscript without argc dependency.
- Generating man pages.
- Generating multi-shell completion scripts (require argc as completion engine).

## Install

### With cargo

```
cargo install argc
```

### Binaries on macOS, Linux, Windows

Download from [Github Releases](https://github.com/sigoden/argc/releases), unzip and add argc to your $PATH.

### GitHub Actions

[extractions/setup-crate](https://github.com/marketplace/actions/setup-crate) can be used to install just in a GitHub Actions workflow.

```yaml
- uses: extractions/setup-crate@v1
  with:
    owner: sigoden
    name: argc
```

## Usage

To write a command-line program with argc, we only need to do two things:

1. Describe options, flags, positional parameters and subcommands in [comment tags](#comment-tags).
2. Insert `eval "$(argc --argc-eval "$0" "$@")"` into script to let argc to handle command line arguments.

Write `example.sh`

```sh
# @flag -F --foo  Flag value
# @option --bar   Option value
# @option --baz*  Option values
# @arg val*       Positional values

eval "$(argc --argc-eval "$0" "$@")"
echo foo: $argc_foo
echo bar: $argc_bar
echo baz: ${argc_baz[@]}
echo val: ${argc_val[@]}
```

Run `./example.sh -F --bar=xyz --baz a --baz b v1 v2`, you can see argc successfully parses arguments and generate variables with `argc_` prefix.

```
foo: 1
bar: xyz
baz: a b
val: v1 v2
```

Run `./example.sh --help`, argc will print help information for you.

```
USAGE: example [OPTIONS] [VAL]...

ARGS:
  [VAL]...  Positional values

OPTIONS:
  -F, --foo           Flag value
      --bar <BAR>     Option value
      --baz [BAZ]...  Option values
  -h, --help          Print help
  -V, --version       Print version
```

## Build

Build a single standalone bashscript without argc dependency.

```
argc --argc-build <SCRIPT> [OUTPATH]
```

```sh
argc --argc-build ./example.sh build/

./build/example.sh -h # Run without argc dependency
```

## Manpage

Generate man pages for the CLI.

```
argc --argc-mangen <SCRIPT> [OUTDIR]
```

```sh
argc --argc-mangen ./example.sh man/

man man/example.1
```

## Completions

Argc provides shell completion for all argc-based scripts.

```
argc --argc-completions <SHELL> [CMDS]...
```

```
# bash (~/.bashrc)
source <(argc --argc-completions bash cmd1 cmd2)

# elvish (~/.config/elvish/rc.elv)
eval (argc --argc-completions elvish cmd1 cmd2 | slurp)

# fish (~/.config/fish/config.fish)
argc --argc-completions fish cmd1 cmd2 | source

# nushell (~/.config/nushell/config.nu)
argc --argc-completions nushell cmd1 cmd2 # update config.nu manually according to output

# powershell ($PROFILE)
Set-PSReadlineKeyHandler -Key Tab -Function MenuComplete
argc --argc-completions powershell cmd1 cmd2 | Out-String | Invoke-Expression

# xonsh (~/.config/xonsh/rc.xsh)
exec($(argc --argc-completions xonsh cmd1 cmd2))

# zsh (~/.zshrc)
source <(argc --argc-completions zsh cmd1 cmd2)

# tcsh (~/.tcshrc)
eval `argc --argc-completions tcsh cmd1 cmd2`
```

The core of all completion scripts is to call `argc --argc-compgen` to obtain completion candidates.

```
$ argc --argc-compgen bash ./example.sh example --
--foo (Flag value)
--bar (Option value)
--baz (Option values)
--help (Print help)
--version (Print version)
```

Argc is a completion engine, see 1000+ examples in [argc-completions](https://github.com/sigoden/argc-completions).

## Argcscript

Argc will automatically find and run `Argcfile.sh` unless the `--argc-*` options are used to change this behavior.

Argcfile is to argc what Makefile is to make. 

What is the benefit?
- Can enjoy convenient shell autocompletion.
- Can be called in any subdirectory without locating the script file every time.
- Serve as a centralized entrypoint/documentation for executing project bashscripts.

Argc is a [task runner](./docs/task-runner.md).

You can run `argc --argc-create` to quickly create a boilerplate argcscript.

```
argc --argc-create [TASKS]...
```

![argcscript](https://github.com/sigoden/argc/assets/4012553/707a3b28-5416-47f1-9d19-788f0135971a)

## Parallel

argc provides features for running commands/functions in parallel.

```sh
argc --argc-parallel "$0" cmd1 arg1 arg2 ::: cmd2
```

The above command will run `cmd1 arg1 arg2` and `cmd2` in parallel.

Compared with GNU parallel, the biggest advantage of argc parallel is that it preserves `argc_*` variables.

## Comment Tags

Comment tags is the CLI definition/documentation.

### `@cmd`

Define a subcommand.

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

```
USAGE: prog <COMMAND>

COMMANDS:
  upload    Upload a file
  download  Download a file
```

### `@alias`

Add aliases for the subcommand.

```sh
# @cmd Run tests
# @alias t,tst
test() {
  echo Run test
}
```

```
USAGE: prog <COMMAND>

COMMANDS:
  test  Run tests [aliases: t, tst]
```

### `@arg`

Define a positional argument.

```sh
# @arg va
# @arg vb!                 required
# @arg vc*                 multi-values
# @arg vd+                 multi-values + required
# @arg vna <PATH>          value notation
# @arg vda=a               default
# @arg vdb=`_default_fn`   default from fn
# @arg vca[a|b]            choices
# @arg vcb[=a|b]           choices + default
# @arg vcc[`_choice_fn`]   choices from fn
# @arg vx~                 capture all remaining args
```

### `@option`

Define a option argument.

```sh
# @option    --oa                   
# @option -b --ob                   short
# @option -c                        short only
# @option    --oc!                  required
# @option    --od*                  multi-occurs
# @option    --oe+                  multi-occurs + required
# @option    --ona <PATH>           value notation
# @option    --onb <FILE> <FILE>    two-args value notations
# @option    --oda=a                default
# @option    --odb=`_default_fn`    default from fn
# @option    --oca[a|b]             choices
# @option    --ocb[=a|b]            choices + default
# @option    --occ[`_choice_fn`]    choices from fn
# @option    --oxa~                 capture all remaining args
```

### `@flag`

Define a flag argument.


```sh
# @flag     --fa 
# @flag  -b --fb         short
# @flag  -c              short only
# @flag     --fd*        multi-occurs
```

### `@env`

Define an environment variable.

```sh
# @env EA                 optional
# @env EB!                required
# @env EC=true            default
# @env EDA[dev|prod]      choices
# @env EDB[=dev|prod]     choices + default
```

### `@meta`

Add a metadata.

```sh
# @meta key [value]
```

| usage                        | scope  | description                                                          |
| :--------------------------- | ------ | :------------------------------------------------------------------- |
| `@meta dotenv [<path>]`      | root   | Load a `.env` file from a custom path, if persent.                   |
| `@meta default-subcommand`   | subcmd | Set the current subcommand as the default.                           |
| `@meta inherit-flag-options` | root   | Subcommands will inherit the flags/options from their parent.        |
| `@meta no-inherit-env`       | root   | Subcommands don't inherit the env vars from their parent.            |
| `@meta symbol <param>`       | anycmd | Define a symbolic parameter, e.g. `+toolchain`, `@argument-file`.    |
| `@meta combine-shorts`       | root   | Short flags/options can be combined, e.g. `prog -xf => prog -x -f `. |
| `@meta man-section <1-8>`    | root   | Override the default section the man page.                           |


### `@describe` / `@version` / `@author`

```sh
# @describe A demo cli
# @version 2.17.1 
# @author nobody <nobody@example.com>
```

```
prog 2.17.1
nobody <nobody@example.com>
A demo cli

USAGE: prog
```

<details>
<summary>

### Value Notation

Value notation is used to describe the value types of options and positional parameters. 

</summary>

```
# @option --target <FILE>
# @arg target <FILE>
```

Here are value notation that will affect the shell completion:

- `FILE`/`PATH`: complete files
- `DIR`: complete directories

</details>

<details>
<summary>

## Windows

The only dependency of argc is bash. Developers under windows OS usually have [git](https://gitforwindows.org/) installed, and git has built-in bash. So you can safely use argc and GNU tools (grep, sed, awk...) under windows OS.

</summary>

## Make `.sh` file executable

If you want to run a `.sh` script file directly like a `.cmd` or `.exe` file, execute the following code in PowerShell.

```ps1
# Add .sh to PATHEXT
[Environment]::SetEnvironmentVariable("PATHEXT", [Environment]::GetEnvironmentVariable("PATHEXT", "Machine") + ";.SH", "Machine")

# Associate the .sh file extension with Git Bash
New-Item -LiteralPath Registry::HKEY_CLASSES_ROOT\.sh -Force
New-ItemProperty -LiteralPath Registry::HKEY_CLASSES_ROOT\.sh -Name "(Default)" -Value "sh_auto_file" -PropertyType String -Force
New-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Classes\sh_auto_file\shell\open\command' `
  -Name '(default)' -Value '"C:\Program Files\Git\bin\bash.exe" "%1" %*' -PropertyType String -Force
```

![windows-shell](https://github.com/sigoden/argc/assets/4012553/16af2b13-8c20-4954-bf58-ccdf1bbe23ef)

</details>

## License

Copyright (c) 2023-2024 argc developers.

argc is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.
