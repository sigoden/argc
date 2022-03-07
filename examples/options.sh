# @describe    All kind of options
# @option      -a --opta           A option
# @option      -b --optb!          A required option
# @option      -c --optc*          A option with multiple values
# @option      -d --optd+          A required option with multiple values
# @option      -e --opte=a         A option with default value
# @option      -x --optx[x|y|z]    A option with choices
# @option      -y --opty[=x|y|z]   A option with choices and default value
# @option      -z --optz![x|y|z]   A required option with choices

eval "$(argc -e $0 "$@")"


( set -o posix ; set ) | grep argc_ # print variables with argc_ prefix

# ./options.sh -b b -d d1 -d d2 -z z
# ./options.sh -a a -b b -c c -d d -e e -x x -y y -z z