# @cmd
cmd() { :; }
# @cmd
cmd::foo() { :; }
# @cmd
cmd::bar() { :; }
# @cmd
cmd::bar::baz() { :; }
# @cmd
cmd::bar::qux() { :; }

eval "$(argc --argc-eval "$0" "$@")"