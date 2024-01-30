# Hooks

- `_argc_before`: call before running any operation
- `_argc_after`: call after running the command

## Example

```sh
_argc_before() {
  echo before
}

_argc_after() {
  echo after
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
after
```
