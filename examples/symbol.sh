# @describe How to use `@meta symbol`
#
# Mock cargo cli
# @meta symbol +toolchain[`_choice_toolchain`]

# @cmd Compile the current package
# @alias b 
build () {
    :;
}

# @cmd Analyze the current package and report errors, but don't build object files
# @alias c
check() {
    :;
}

_choice_toolchain() {
    cat <<-'EOF'
stable
beta
nightly
EOF
}

eval "$(argc --argc-eval "$0" "$@")"