# describe Sample CLI that uses the default command option

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