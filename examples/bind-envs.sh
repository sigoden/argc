# @cmd How to bind env to param

# @flag --fa1 $$
# @flag --fa2 $$
# @flag --fa3 $FA
# @flag --fc* $$
# @flag --fd $$
flags() {
    _debug "$@"
}

# @cmd
# @option --oa1 $$
# @option --oa2 $$
# @option --oa3 $OA
# @option --ob! $OB
# @option --oc*, $$
# @option --oda=a $$
# @option --odb=`_default_fn` $$
# @option --oca[a|b] $$
# @option --occ*[a|b] $$
# @option --ofa[`_choice_fn`] $$
# @option --ofd*,[`_choice_fn`] $$
# @option --oxa~ $$
options() {
    _debug "$@"
}

# @cmd
# @arg val $$
cmd_arg1() {
    _debug "$@"
}

# @cmd
# @arg val $VA
cmd_arg2() {
    _debug "$@"
}

# @cmd
# @arg val=xyz $$
cmd_arg_with_default() {
    _debug "$@"
}

# @cmd
# @arg val[x|y|z] $$
cmd_arg_with_choice() {
    _debug "$@"
}

# @cmd
# @arg val[`_choice_fn`] $$
cmd_arg_with_choice_fn() {
    _debug "$@"
}

# @cmd
# @arg val*,[`_choice_fn`] $$
cmd_multi_arg_with_choice_fn_and_comma_sep() {
    _debug "$@"
}

# @cmd
# @arg val1! $$
# @arg val2! $$
# @arg val3! $$
cmd_three_required_args() {
    _debug "$@"
}

# @cmd
# @option --OA $$ <XYZ>
# @arg val $$ <XYZ>
cmd_for_notation() {
    _debug "$@"
}

_debug() {
    ( set -o posix ; set ) | grep ^argc_
    echo "$argc__fn" "$@"
}

_default_fn() {
    echo argc
}

_choice_fn() {
	echo abc
	echo def
	echo ghi
}

eval "$(argc --argc-eval "$0" "$@")"