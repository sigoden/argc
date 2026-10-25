# Advanced Topics

This page covers features that extend a basic argc CLI.

## The `main` Function

- **no subcommand**
    - **no main**: parse params, call nothing; the script continues.
    - **main**: parse params and call `main`.
- **subcommand/nested-subcommand**
    - **no main**: with no child selected, show help or report a subcommand error;
    - **main**: with no child selected, call `main` for the root or `<cmd>[::<subcmd>...]::main` for the nested subcommand.

`main` is not a fallback for unknown subcommand names; those remain normass parsing errors.

## Uncommon Option Styles

Single-dash long options are supported:

```sh
# @option -output <FILE>     # invoked as -output
```

An option name ending in `:` requires the assigned `--name=value` form:

```sh
# @option --follow:[a|b]     # `--follow=` is shown in completion
```

```sh
prog cmd --follow=a   # valid
prog cmd --follow a   # error
```

A prefix option consumes the rest of the argument as its value:

```sh
# @option -X-*
```

```sh
prog cmd -Xabc -Xdef=3
```

Each occurrence is recorded in an associative array:

```sh
declare -A argc_X
argc_X[abc]=1
argc_X[def]=3
```

Plus-prefixed options use `+` instead of `-`:

```sh
# @option +oa
# @option +b +ob
# @option +c
```

Plus-prefixed options are exposed with a `plus_` prefix in the variable name:

```sh
# @option +oa
prog +oa value
# argc_plus_oa=value
```

## Escape Hatch `___internal___`

`___internal___` tells an argc script to call one of its own functions instead of running a command. argc uses it to call choice functions and completion functions while reusing the current command-line context.

```sh
./prog ___internal___ <fn> [<args>...]
```

- `<fn>` is the function to call.
- `<args>` are parsed like a normal invocation, populating `argc_*` variables.
- If the first argument after `<fn>` is `--`, parsing is skipped and all remaining arguments are passed to `<fn>` verbatim.

Example:

```sh
# @cmd
# @arg val*[`_choice_fn`]
foo() { :; }

_choice_fn() {
    echo "$@"
}

eval "$(argc --argc-eval "$0" "$@")"
```

```sh
./prog ___internal___ _choice_fn prog foo a b
# a b

./prog ___internal___ _choice_fn -- prog foo a b
# prog foo a b
```

> This mechanism is intended for argc's own use and for testing. You normally do not need to call it directly. A standalone script produced by `--argc-build` rejects `___internal___`.

### Trailing underscore

All trailing `_` characters are stripped from a command name, so both `cat_` and `cat__` expose the command `cat` while the underlying functions keep their original names.

This convention avoids collisions with commonly used commands. Recipes are ordinary shell functions, so a recipe named `cat` would define a function named `cat` and shadow the external `cat` command inside the script. By naming the function `cat_`, you can expose a command called `cat` while calls to `cat` inside the script still reach the real command.

```sh
# @cmd
cat_() { :; }
```

The command is invoked as `prog cat`, while the underlying function remains named `cat_`.
