# @describe How to use `@meta combine-shorts`
#
# Mock rm cli
# Examples:
#   prog -rf dir1 dir2
# 
# @meta combine-shorts
# @flag -r --recursive remove directories and their contents recursively
# @flag -f --force ignore nonexistent files and arguments, never prompt
# @arg path* the path to remove
              
eval "$(argc --argc-eval "$0" "$@")"

_debug() {
    ( set -o posix ; set ) | grep ^argc_
    echo "$argc__fn" "$@"
}

_debug