# Hooks

If the function `_argc_init` exists, argc will automatically execute it after all variables have been initialized and before executing command functions or any subsequent code.

When is this hook needed?
- The process of initializing global variables can be slow.
- Desire to use argc variables during the initialization of global variables.

## Example

```sh
_argc_init() {
  echo init
}

main() {
  echo main
}

eval "$(argc --argc-eval "$0" "$@")"
```

```
$ prog
init
main
```
