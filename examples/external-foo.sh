# @describe An external subcommand "foo"

# @arg message!   Message to echo
main() {
    echo "external-foo: $argc_message"
}

eval "$(argc --argc-eval "$0" "$@")"