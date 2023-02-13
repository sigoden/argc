# @describe    All kind of options
# @option      -a --opt1           A option
# @option      -b --opt2!          A required option
# @option      -c --opt3*          A option with multiple values
# @option      -d --opt4+          A required option with multiple values
# @option      -e --opt5=a         A option with default value
# @option      -x --opt6[x|y|z]    A option with choices
# @option      -y --opt7[=x|y|z]   A option with choices and default value
# @option      -z --opt8![x|y|z]   A required option with choices

eval "$(argc "$0" "$@")"


( set -o posix ; set ) | grep argc_ # print variables with argc_ prefix

# ./options.sh -b b -d d1 -d d2 -z z
# ./options.sh -a a -b b -c c -d d -e e -x x -y y -z z