#!/usr/bin/env bash

set -eu


# @flag      --fa 
# @option    --oa                   
# @option    --of*,                 multi-occurs + comma-separated list
# @option    --oda=a                default
# @option    --oca[a|b]             choice
# @option    --ofa[`_choice_fn`]    choice from fn
# @option    --oxa~                 capture all remaining args

main() {
    ( set -o posix ; set ) | grep ^argc_
    echo "${argc__fn:-}" "$@"
}

_choice_fn() {
    echo abc
    echo def
	echo ghi
}

eval "$(argc --argc-eval "$0" "$@")"