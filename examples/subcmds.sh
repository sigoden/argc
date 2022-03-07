
# @cmd A simple positional argument
# @arg value 
cmda() {
    :;
}

# @cmd A required positional argument
# @arg value!
cmdb() {
    :;
}

# @cmd A positional argument support multiple values
# @arg value*
cmdc() {
    :;
}

# @cmd A required positional argument support multiple values
# @arg value*
cmdd() {
    :;
}

# @cmd A positional argument with default value
# @arg value=a
cmde() {
    :;
}

# @cmd A positional argument with choices
# @arg value[x|y|z]
cmdx() {
    :;
}


# @cmd A positional argument with choices and default value
# @arg value[=x|y|z]
cmdy() {
    :;
}

# @cmd A required positional argument with choices
# @arg value![x|y|z]
cmdz() {
    :;
}


eval "$(argc -e $0 "$@")"

( set -o posix ; set ) | grep argc_ # print variables with argc_ prefix