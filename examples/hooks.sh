#/usr/bin/env node

set -e

_argc_init() {
  echo init
}

main() {
  echo main
}

eval "$(argc --argc-eval "$0" "$@")"