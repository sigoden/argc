
# @cmd A simple positional argument
# @arg value 
cmd1() {
    :;
}

# @cmd A simple positional argument with notation
# @arg value <PATH>
cmd2() {
    :;
}

# @cmd A required positional argument
# @arg value!
cmd3() {
    :;
}

# @cmd A positional argument support multiple values
# @arg value*
cmd4() {
    :;
}

# @cmd A required positional argument support multiple values
# @arg value*
cmd5() {
    :;
}

# @cmd A positional argument with default value
# @arg value=a
cmd6() {
    :;
}

# @cmd A positional argument with choices
# @arg value[x|y|z]
cmd7() {
    :;
}


# @cmd A positional argument with choices and default value
# @arg value[=x|y|z]
cmd8() {
    :;
}

# @cmd A required positional argument with choices
# @arg value![x|y|z]
cmd9() {
    :;
}


eval "$(argc --argc-eval "$0" "$@")"

( set -o posix ; set ) | grep argc_ # print variables with argc_ prefix