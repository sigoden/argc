# Argc

[![CI](https://github.com/sigoden/argc/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/argc/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/argc.svg)](https://crates.io/crates/argc)

A bash cli framework, also a task management & automation tool.

![demo](https://user-images.githubusercontent.com/4012553/228990851-fee5649f-aa24-4297-a924-0d392e0a7400.gif)

Argc lets you define your CLI through comments and focus on your specific code, without worrying about command line argument parsing, usage text, error messages and other functions that are usually handled by a framework in any other programming language.

## Features

- **Argument parsing**: Parsing user's command line, validating and extracting:
  - Optional or required positional arguments.
  - Optional or required option arguments.
  - Standard flags (like --help, --version, --verbose, -vvv).
  - Commands (and sub-commands).
- **Build bashscript**: Build a single standalone bashscript without argc dependency.
- **Cross-shell autocompletion**: Generate completion scripts for bash, zsh, fish, powershell, and more.
- **Man page**: Generate manage page documentation for your script.
- **Task runner**: An ideal task runner in Bash to automate the execution of predefined tasks with Argcfile.sh.
- **Self documentation**: Comments with tags are CLI definitions, documentation, usage text.
- **Cross platform**: A single executable file that can run on macOS, Linux, Windows, and BSD systems.

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

1. Describe options, flags, positional parameters and subcommands in [comment tag](#comment-tag).
2. Insert `eval "$(argc --argc-eval "$0" "$@")"` into script to let argc to handle command line arguments.

Write `example.sh`

```sh
# @flag -F --foo  Flag param
# @option --bar   Option param
# @option --baz*  Option param (multi-occur)
# @arg val*       Positional param

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
  [VAL]...  Positional param

OPTIONS:
  -F, --foo           Flag param
      --bar <BAR>     Option param
      --baz [BAZ]...  Option param (multi-occur)
  -h, --help          Print help
  -V, --version       Print version
```

## Comment Tag

Argc uses comments with a `JsDoc` inspired syntax to add functionality to the scripts at runtime.
This syntax, known as a `comment tag`, is a normal Bash comment followed by an `@` sign and a tag.
It's how the argc parser identifies configuration.

| Tag                                             | Description                          |
| :---------------------------------------------- | ------------------------------------ |
| [`@describe`](./docs/specification.md#describe) | Set the description for the command. |
| [`@cmd`](./docs/specification.md#cmd)           | Define a subcommand.                 |
| [`@alias`](./docs/specification.md#alias)       | Set aliases for the subcommand.      |
| [`@arg`](./docs/specification.md#arg)           | Define a positional argument.        |
| [`@option`](./docs/specification.md#option)     | Define an option argument.            |
| [`@flag`](./docs/specification.md#flag)         | Define a flag argument.              |
| [`@env`](./docs/specification.md#env)           | Define an environment variable.      |
| [`@meta`](./docs/specification.md#meta)         | Add a metadata.                      |

See [specification](https://github.com/sigoden/argc/blob/main/docs/specification.md) for the grammar and usage of all the comment tags.

## Build

Build a single standalone bashscript without argc dependency.

```
argc --argc-build <SCRIPT> [OUTPATH]
```

```sh
argc --argc-build ./example.sh build/

./build/example.sh -h       # Run the script without argc dependency
```

## Argcscript

Argc will automatically find and run `Argcfile.sh` unless the `--argc-*` options are used to change this behavior.

Argcfile is to argc what Makefile is to make. 

What is the benefit?
- Can enjoy convenient shell autocompletion.
- Can be called in any subdirectory without locating the script file every time.
- Serve as a centralized entrypoint/documentation for executing project bashscripts.

Argc is a [task runner](https://github.com/sigoden/argc/blob/main/docs/task-runner.md).

You can run `argc --argc-create` to quickly create a boilerplate argcscript.

```
argc --argc-create [TASKS]...
```

![argcscript](https://github.com/sigoden/argc/assets/4012553/707a3b28-5416-47f1-9d19-788f0135971a)

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
--foo (Flag param)
--bar (Option param)
--baz (Option param (multi-occur))
--help (Print help)
--version (Print version)
```

Argc is a completion engine, see 1000+ examples in [argc-completions](https://github.com/sigoden/argc-completions).

## Manpage

Generate man pages for the CLI.

```
argc --argc-mangen <SCRIPT> [OUTDIR]
```

```sh
argc --argc-mangen ./example.sh man/

man man/example.1
```

## Parallel

argc provides features for running commands/functions in parallel.

```sh
argc --argc-parallel "$0" cmd1 arg1 arg2 ::: cmd2
```

The above command will run `cmd1 arg1 arg2` and `cmd2` in parallel.

Compared with GNU parallel, the biggest advantage of argc parallel is that it preserves `argc_*` variables.

<details>
<summary>

## Windows

The only dependency of argc is bash.  Developers who work on Windows OS usually have [git](https://gitforwindows.org/) (which includes git-bash) installed, so you can safely use argc and GNU tools (grep, sed, awk...) on windows OS.

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
