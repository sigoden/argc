# @describe An external subcommand "bar" with options

# @flag   -v --verbose   Enable verbose output
# @arg    message!       Message to echo
main() {
    echo "external-bar: $argc_message"
}

eval "$(argc --argc-eval "$0" "$@")"