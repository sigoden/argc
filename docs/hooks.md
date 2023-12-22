# Hooks

If the function `_argc_before` exists, argc will automatically execute it after initializing variables.

## Example

```sh
# @flag --foo
# @option --bar

_argc_before() {
  echo before
}

main() {
  echo main
}

eval "$(argc --argc-eval "$0" "$@")"
```

```
$ prog
before
main
```
