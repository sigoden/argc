# @describe    All kind of options
# @option      -a --opt1           A option
# @option      -b --opt2!          A required option
# @option      -c --opt3*          A option with multiple values
# @option      -d --opt4+          A required option with multiple values
# @option      -e --opt5=a         A option with default value
# @option      -f --opt6=`_fn`     A option with default value from fn
# @option      -x --opt7[x|y|z]    A option with choices
# @option      -y --opt8[=x|y|z]   A option with choices and default value
# @option      -z --opt9![x|y|z]   A required option with choices
# @option      -w  --opt10[`_fn2`]  A option with choices from fn 

_fn() {
    echo abc
}

_fn2() {
    echo "x y z"
}

eval "$(argc --argc-eval "$0" "$@")"


( set -o posix ; set ) | grep argc_ # print variables with argc_ prefix

# ./options.sh -b b -d d1 -d d2 -z z
# ./options.sh -a a -b b -c c -d d -e e -x x -y y -z z