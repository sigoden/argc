#/usr/bin/env node

set -e

# @cmd
cmd1() {
    sleep 3
    echo cmd1 "$@"
    echo argc_oa: $argc_oa
    echo cmd1 stderr >&2
}

# @cmd
cmd2() {
    sleep 3
    echo cmd2 "$@"
    echo argc_oa: $argc_oa
    echo cmd2 stderr >&2
}

# @cmd
# @option --oa
foo() {
    argc --argc-parallel "$0" cmd1 abc ::: func ::: cmd2 
}

# @cmd
# @option --oa
bar() {
    cmd1 abc
    func
    cmd2
}

func() {
    echo func
}

eval "$(argc --argc-eval "$0" "$@")"