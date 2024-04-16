# describe How to use `@meta default-subcommand`

# @cmd Upload a file
# @meta default-subcommand
upload() {
    echo upload "$@"
}

# @cmd Download a file
download() {
    echo download "$@"
}

eval "$(argc --argc-eval "$0" "$@")"