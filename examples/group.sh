# @cmd
foo@bar() {
    echo "foo@bar"
}

# @cmd
foo@baz() {
    echo "foo@baz"
}

# @cmd
foo.bar() {
    echo "foo.bar"
}

# @cmd
foo.baz() {
    echo "foo.baz"
}

# @cmd
foo-bar() {
    echo "foo-bar"
}

# @cmd
foo-baz() {
    echo "foo-baz"
}

# @cmd
foo_bar() {
    echo "foo_bar"
}

# @cmd
foo_baz() {
    echo "foo_baz"
}

# @cmd
foo:bar() {
    echo "foo:bar"
}

# @cmd
foo:baz() {
    echo "foo:baz"
}

eval $(argc --argc-eval "$0" "$@")