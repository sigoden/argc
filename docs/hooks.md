# Hooks

Argc supports two kinds of hooks:

- `_argc_before`: call before performing any operation
- `_argc_before`: call after running the command function


```sh
# @flag --foo
# @option --bar

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
after
main
```
