#/usr/bin/env node

set -e

# @flag --foo
# @option --bar

_argc_before() {
  echo before
}

main() {
  echo main
}

eval "$(argc --argc-eval "$0" "$@")"