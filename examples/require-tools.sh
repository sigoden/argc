# @describe how to use `@meta require-tools`

# @meta require-tools awk,sed

# @cmd
# @meta require-tools git
require-git() {
    :;
}

# @cmd
# @meta require-tools not-found
require-not-found() {
    :;
}

eval "$(argc --argc-eval "$0" "$@")"
