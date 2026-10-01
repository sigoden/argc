# Misc

## Inspecting Variables

When debugging a script, you can dump the internal variables and the `ARGC_*` environment variables from any function:

```sh
(set -o posix; set) | grep argc_
printenv | grep ARGC_
```

- `(set -o posix; set) | grep argc_` lists the argc shell variables (e.g. `argc__args`, `argc__positionals`, `argc_oa`). The `set -o posix` makes the output portable across shells.
- `printenv | grep ARGC_` lists the exported `ARGC_*` environment variables (e.g. `ARGC_PWD`, `ARGC_OS`).


## Using `___internal___`

`___internal___` is an internal marker that tells an argc script to invoke one of its own functions instead of running a command. Argc uses it to call functions defined in the script — such as choice functions, default functions, and completion functions — while reusing the current command-line context.

The syntax is:

```sh
./prog ___internal___ <fn> [<args>...]
```

- `<fn>` is the name of the function to call.
- `<args>` are parsed like a normal invocation, populating the `argc_*` variables, and `<fn>` is then called with the resulting positional arguments.
- If the first argument after `<fn>` is `--`, argument parsing is skipped and all remaining arguments are passed to `<fn>` verbatim.

### Example

```sh
# @cmd
# @arg val[`_choice_fn`]
foo() {
    :;
}

_choice_fn() {
    echo "$@"
}

eval "$(argc --argc-eval "$0" "$@")"
```

Without `--`, the arguments are parsed (`prog` is the program name and `foo` is a subcommand), and `_choice_fn` receives the resulting positional arguments:

```sh
$ ./prog ___internal___ _choice_fn prog foo a b
a b
```

With `--`, parsing is skipped and `_choice_fn` receives the raw arguments:

```sh
$ ./prog ___internal___ _choice_fn -- prog foo a b
prog foo a b
```

> `___internal___` is intended for argc's own use and for testing. You normally don't need to call it directly.

## Trailing underscore

A trailing `_` is stripped from the command name. This lets a recipe reuse an existing command name without shadowing it, such as `cat` or `echo`:

```sh
# @cmd
cat_() { :; }
```

The command is invoked as `prog cat`, while the underlying function name remains `cat_`.
