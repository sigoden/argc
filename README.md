# Argc

[![CI](https://github.com/sigoden/argc/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/argc/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/argc.svg)](https://crates.io/crates/argc)

Argc is a powerful Bash framework that simplifies building command-line interfaces (CLIs) and automating tasks. It lets you define your CLI through comments, focusing on your core logic without dealing with argument parsing, usage text, error messages, and other boilerplate code.

![demo](https://user-images.githubusercontent.com/4012553/228990851-fee5649f-aa24-4297-a924-0d392e0a7400.gif)

## Features

- **Effortless Argument Parsing:**
  - Handles positional and optional arguments, flags, commands, and subcommands.
  - Validates and extracts user input for robust error handling.
- **Standalone Bash Script Creation:**
  - Build a single executable script without depending on Argc itself.
- **Cross-shell Autocompletion:**
  - Generate autocompletion scripts for various shells (bash, zsh, fish, powershell, etc.).
- **Man Page Generation:**
  - Automatically create comprehensive man page documentation for your script.
- **Environment Variable Integration:**
  - Define, validate, and bind environment variables to options and positional arguments.
- **Task Automation:**
  - An ideal task runner for Bash, allowing you to automate tasks using Argcfile.sh.
- **Cross-Platform Compatibility:**
  - Seamlessly run your Argc-based scripts on macOS, Linux, Windows, and BSD systems.

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

## Get Started

To write a command-line program with Argc, only requires two simple steps:

1. **Define your CLI**: Describe options, flags, positional parameters, and subcommands using comment tags (explained later).
2. **Process Arguments**: Insert `eval "$(argc --argc-eval "$0" "$@")"` into your script to enable Argc's argument handling.

```sh
# @flag -F --foo  Flag param
# @option --bar   Option param
# @option --baz*  Option param (multi-occurs)
# @arg val*       Positional param

eval "$(argc --argc-eval "$0" "$@")"

echo foo: $argc_foo
echo bar: $argc_bar
echo baz: ${argc_baz[@]}
echo val: ${argc_val[@]}
```

Run the script with some sample arguments:
```sh
./example.sh -F --bar=xyz --baz a --baz b v1 v2
```

Argc will parse them and generate variables with the `argc_` prefix, as shown in the output.

```
foo: 1
bar: xyz
baz: a b
val: v1 v2
```

Additionally, you can run `./example.sh --help` to see Argc automatically generate help information for your CLI:

```
USAGE: example [OPTIONS] [VAL]...

ARGS:
  [VAL]...  Positional param

OPTIONS:
  -F, --foo           Flag param
      --bar <BAR>     Option param
      --baz [BAZ]...  Option param (multi-occurs)
  -h, --help          Print help
  -V, --version       Print version

```
Now you're ready to start building powerful command-line programs with Argc!

## Comment Tags

Comment tags are standard Bash comments prefixed with `@` and a specific tag. They provide instructions to Argc for configuring your script's functionalities.

| Tag                                             | Description                           |
| :---------------------------------------------- | ------------------------------------- |
| [`@describe`](./docs/specification.md#describe) | Sets the description for the command. |
| [`@cmd`](./docs/specification.md#cmd)           | Defines a subcommand.                 |
| [`@alias`](./docs/specification.md#alias)       | Sets aliases for the subcommand.      |
| [`@arg`](./docs/specification.md#arg)           | Defines a positional argument.        |
| [`@option`](./docs/specification.md#option)     | Defines an option argument.           |
| [`@flag`](./docs/specification.md#flag)         | Defines a flag argument.              |
| [`@env`](./docs/specification.md#env)           | Defines an environment variable.      |
| [`@meta`](./docs/specification.md#meta)         | Adds metadata.                        |

See [specification](https://github.com/sigoden/argc/blob/main/docs/specification.md) for the grammar and usage of all the comment tags.

## Build

Build a standalone bash script without argc dependency.

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
--baz (Option param (multi-occurs))
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
