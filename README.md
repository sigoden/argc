# Argc

[![CI](https://github.com/sigoden/argc/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/argc/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/argc.svg)](https://crates.io/crates/argc)

Argc helps you easily create and use cli that based on bashscript.

![demo](https://user-images.githubusercontent.com/4012553/228990851-fee5649f-aa24-4297-a924-0d392e0a7400.gif)

You define cli through comments, and argc takes care of the remaining tasks:

* Parse options/positionals/subcommands
* Validate user input
* Output comprehensive help text
* Initialize intuitive variables for arguments
* Provide tab-completion

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

1. Describe options, flags, positional parameters and subcommands in comments.
2. Insert `eval "$(argc --argc-eval "$0" "$@")"` into script to let argc to parse command line arguments.

Write `example.sh`

```sh
# @flag   --foo   Flag value
# @option --bar   Option value
# @arg baz*       Positional values

eval "$(argc --argc-eval "$0" "$@")"
echo foo: $argc_foo
echo bar: $argc_bar
echo baz: ${argc_baz[@]}
```

Run `./example.sh --foo --bar=xyz a b c`, you can see argc successfully parses arguments and generate variables with `argc_` prefix.

```
foo: 1
bar: xyz
baz: a b c
```

Run `./example.sh -h`, argc will print help information for you.

```
USAGE: example.sh [OPTIONS] [BAZ]...

ARGS:
  [BAZ]...  Positional values

OPTIONS:
      --foo        Flag value
      --bar <BAR>  Option value
  -h, --help       Print help
```

## Comment Tags

`argc` parses cli definition from comment tags.

### @cmd

Define a subcommand

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
USAGE: test.sh <COMMAND>

COMMANDS:
  upload    Upload a file
  download  Download a file
```

### @arg

Define a positional argument.

```sh
# @arg va
# @arg vb!                 required
# @arg vc*                 multi-values
# @arg vd+                 multi-values + required
# @arg vna <PATH>          value notation
# @arg vda=a               default
# @arg vca[a|b]            choices
# @arg vcb[=a|b]           choices + default
# @arg vx~                 capture all remaining args
```

### @option

Define a option.

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
# @option    --oca[a|b]             choices
# @option    --ocb[=a|b]            choices + default
# @option    --oxa~                 capture all remaining args
```

### @flag

Define a flag. A flag is an option of boolean type, and is always false by default (e.g. --verbose, --quiet, --all, --long, etc).


```sh
# @flag     --fa 
# @flag  -b --fb         short
# @flag  -c              short only
# @flag     --fd*        multi-occurs
```

### @alias

Add aliases for subcommand.

```sh
# @cmd Run tests
# @alias t,tst
test() {
  echo Run test
}
```

```
USAGE: test.sh <COMMAND>

COMMANDS:
  test  Run tests [aliases: t, tst]
```

### @env

Define an environment

```sh
# @env EA                 optional
# @env EB!                required
# @env EC=true            default
# @env EDA[dev|prod]      choices
# @env EDB[=dev|prod]     choices + default
```

### @describe / @version / @author

```sh
# @describe A demo cli
# @version 2.17.1 
# @author nobody <nobody@example.com>

# @cmd Run test
test() {
  echo Run test
}
```

```
test.sh 2.17.1
nobody <nobody@example.com>
A demo cli

USAGE: test.sh <COMMAND>

COMMANDS:
  test  Run test
```

### Value Notation

Value notation is used to describe value type of options and positional parameters.

```
# @option --target <FILE>
# @arg target <FILE>
```

Here are some value notation that will affect the shell completion.

- `FILE`/`PATH`: complete files
- `DIR`: complete directories

## Shell Completion

Argc provides shell completion for argc command and all the bash scripts powered by argc.

```
# bash (~/.bashrc)
source <(argc --argc-completions bash mycmd1 mycmd2)

# elvish (~/.config/elvish/rc.elv)
eval (argc --argc-completions elvish mycmd1 mycmd2 | slurp)

# fish (~/.config/fish/config.fish)
argc --argc-completions fish mycmd1 mycmd2 | source

# nushell (~/.config/nushell/config.nu)
argc --argc-completions nushell mycmd1 mycmd2 # update config.nu manually according to output

# powershell ($PROFILE)
Set-PSReadlineKeyHandler -Key Tab -Function MenuComplete
argc --argc-completions powershell mycmd1 mycmd2 | Out-String | Invoke-Expression

# xonsh (~/.config/xonsh/rc.xsh)
exec($(argc --argc-completions xonsh mycmd1 mycmd2))

# zsh (~/.zshrc)
source <(argc --argc-completions zsh mycmd1 mycmd2)

# tcsh (~/.tcshrc)
eval `argc --argc-completions tcsh mycmd1 mycmd2`
```

**Replace `mycmd1 mycmd2` with your argc scripts**.

Argc can be used as multiple shell completion engine. see [argc-completions](https://github.com/sigoden/argc-completions)

## Argcscript

Argc will automatically find and run `Argcfile.sh` unless `--argc-*` options are used to change this behavior.

Argcfile is to argc what Makefile is to make.

what is the benefit?

- Can enjoy a handy shell completion.
- Can be invoked in arbitrarily subdirectory, no need to locate script file each time.
- As a centralized entrypoint/document for executing the project's bash scripts.
- Serves as a script for a task runner.

You can use `argc --argc-create` to quickly create a boilerplate argcscript.

![argcscript](https://github.com/sigoden/argc/assets/4012553/5130d9c5-90ff-478e-8404-3db6f55ba1d0)

## Parallel

argc provides features for running commands/functions in parallel.

```sh
argc --argc-parallel "$0" cmd1 arg1 arg2 ::: cmd2
```

The above command will run `cmd1 arg1 arg2` and `cmd2` in parallel. Functions running in parallel mode can still access the `argc_*` variable.

## License

Copyright (c) 2023-2024 aichat-developers.

argc is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.
