# @cmd
cmd() { :; }
# @cmd
cmd::foo() { :; }
# @cmd
cmd::bar() { :; }
# @cmd
cmd::bar::baz() { :; }

eval "$(argc --argc-eval "$0" "$@")"