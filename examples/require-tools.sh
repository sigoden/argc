# @describe Demonstrate how to use require-tools meta

# @meta require-tools curl,sed

# @cmd
# @meta require-tools git
cmd1() {
    :;
}

# @cmd
# @meta require-tools not-found
cmd2() {
    :;
}

eval "$(argc --argc-eval "$0" "$@")"
