# @describe Test external subcommands
# @meta external-subcommands

# @cmd A built-in subcommand
# @alias b
# @arg message!   Message to echo
builtin() {
    echo "builtin: $argc_message"
}

eval "$(argc --argc-eval "$0" "$@")"
