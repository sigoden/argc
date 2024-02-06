#/usr/bin/env node

# Argc supports two hooks:
#   _argc_before: call before running the command function (after initialized variables)
#   _argc_after: call after running the command function

set -e

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