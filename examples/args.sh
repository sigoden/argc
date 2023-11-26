# @cmd
cmd() {
    _debug "$@";
}

# @cmd
# @alias a
cmd_alias() {
    _debug "$@";
}

# @cmd
# @arg val
cmd_arg() {
    _debug "$@";
}

# @cmd
# @arg val*
cmd_multi_arg() {
    _debug "$@";
}

# @cmd
# @arg val+
cmd_required_multi_arg() {
    _debug "$@";
}

# @cmd
# @arg val!
cmd_required_arg() {
    _debug "$@";
}

# @cmd
# @arg val=xyz
cmd_arg_with_default() {
    _debug "$@";
}

# @cmd
# @arg val=`_default_fn`
cmd_arg_with_default_fn() {
    _debug "$@";
}

# @cmd
# @arg val[x|y|z]
cmd_arg_with_choices() {
    _debug "$@";
}

# @cmd
# @arg val[=x|y|z]
cmd_arg_with_choices_and_default() {
    _debug "$@";
}

# @cmd
# @arg val*[x|y|z]
cmd_multi_arg_with_choices() {
    _debug "$@";
}

# @cmd
# @arg val+[x|y|z]
cmd_required_multi_arg_with_choices() {
    _debug "$@";
}

# @cmd
# @arg val[`_choice_fn`]
cmd_arg_with_choice_fn() {
    _debug "$@";
}

# @cmd
# @arg val[?`_choice_fn`]
cmd_arg_with_choice_fn_and_skip_check() {
    _debug "$@";
}


# @cmd
# @arg val![`_choice_fn`]
cmd_required_arg_with_choice_fn() {
    _debug "$@";
}

# @cmd
# @arg val*[`_choice_fn`]
cmd_multi_arg_with_choice_fn() {
    _debug "$@";
}

# @cmd
# @arg val+[`_choice_fn`]
cmd_required_multi_arg_with_choice_fn() {
    _debug "$@";
}


# @cmd
# @arg val*,[`_choice_fn`]
cmd_multi_arg_with_choice_fn_and_comma_sep() {
    _debug "$@";
}


# @cmd
# @arg vals~
cmd_terminaled() {
    _debug "$@";
}

# @cmd
# @arg val <FILE>
cmd_arg_with_notation() {
    _debug "$@";
}

# @cmd
# @arg val1*
# @arg val2*
cmd_two_multi_args() {
    _debug "$@";
}

# @cmd
# @arg val1!
# @arg val2+
cmd_one_required_second_required_multi() {
    _debug "$@";
}

# @cmd
# @arg val1!
# @arg val2!
# @arg val3!
cmd_three_required_args() {
    _debug "$@";
}

_debug() {
    printenv | grep ARGC_
    ( set -o posix ; set ) | grep argc_
    echo "$@"
}

_default_fn() {
	echo abc
}

_choice_fn() {
	echo abc
	echo def
	echo ghi
}

eval "$(argc --argc-eval "$0" "$@")"
