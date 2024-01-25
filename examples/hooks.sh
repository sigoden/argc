#/usr/bin/env node

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