# @describe Test all aspects
# @version    0.10
# @author     nobody <nobody@example.com>
# @cmd Preferred
# @arg        arg1*            A positional arg
# @flag       -f --flag1       A flag
# @option     -o --opt1        A option
cmd_preferred() {
    print_argc_vars
}

# @cmd Omitted
# @arg        arg1
# @flag       --flag1
# @option     --opt1
cmd_omitted() {
    print_argc_vars
}

# @cmd Options all kind of names
# @option      --opt1                optional
# @option      --opt2!               required
# @option      --opt3*               optional, multiple
# @option      --opt4+               required, multiple
# @option      --opt5=a              optional, default
# @option      --opt6[a|b|c]         choices
# @option      --opt7[=a|b|c]        choices, default
# @option      --opt8![a|b|c]        required, choices
# @option      --opt9=`_fn_foo`      optional, default from fn
# @option      --op10[`_fn_bars`]    choices from fn
# @option      --op11![`_fn_bars`]   required, choices from fn
cmd_option_names() {
    print_argc_vars
}

# @cmd Options all kind of formats
# @option      --opt1
# @option  -a  --opt2
# @option      --opt3 <OPT>    
# @option      --opt4           With description
# @option  -b  --opt5           With description
# @option  -c  --opt6 <OPT>
# @option      --opt7 <OPT>     With description
cmd_option_formats() {
    print_argc_vars
}

# @cmd Option value quoted
# @option      --opt1=a
# @option      --opt2="a b"
# @option      --opt3[a 3|b|c]
# @option      --opt4[=a b|c d|e f]
# @option      --opt5[="a|b"|"c]d"|ef]
cmd_option_quotes() {
    print_argc_vars
}

# @cmd All kind of flags
# @flag      --foo1
# @flag   -a --foo2    
# @flag      --foo3     With description
# @flag   -b --foo4     With description
cmd_flag_formats() {
    print_argc_vars
}

# @cmd  Positional one required
# @arg   arg1! <ARG>  A required arg
cmd_positional_only() {
    print_argc_vars
}

# @cmd  Positional all required
# @arg   arg1!     A required arg
# @arg   arg2+     A required arg, multiple
cmd_positional_requires() {
    print_argc_vars
}

# @cmd  Positional with choices
# @arg   arg[a|b]   A arg with choices
cmd_positional_with_choices() {
    print_argc_vars
}

# @cmd  Positional with default value
# @arg   arg=a      A arg with default value
cmd_positional_with_default() {
    print_argc_vars
}

# @cmd  Positional with default value
# @arg   arg=`_fn_foo`  A arg with default fn
cmd_positional_with_default_fn() {
    print_argc_vars
}

# @cmd  Positional with choices and value
# @arg   arg[=a|b]   A arg with choices and default value
cmd_positional_with_choices_and_default() {
    print_argc_vars
}

# @cmd  Positional with choices and value
# @arg   arg[`_fn_bars`]  A arg with choices fn
cmd_positional_with_choices_fn() {
    print_argc_vars
}

# @cmd  Positional with choices and required
# @arg   arg![a|b]   A arg with choices and required
cmd_positional_with_choices_and_required() {
    print_argc_vars
}

# @cmd  Positional with choices and value
# @arg   arg![`_fn_bars`]  A arg with choices fn and required
cmd_positional_with_choices_fn_and_required() {
    print_argc_vars
}

# @cmd  Command without any arg
cmd_without_any_arg() {
    print_argc_vars
}

# @cmd  Command with alias
# @alias a,alias
cmd_alias() {
    print_argc_vars
}

# @cmd  Command with hyphens
# @arg       hyphen-positional
# @flag     --hyphen-flag
# @option   --hyphen-option
cmd_with_hyphens() {
    print_argc_vars
}

print_argc_vars() {
    ( set -o posix ; set ) | grep argc_
}

_fn_foo() {
    echo "foo"
}

_fn_bars() {
    echo " a1 a2 a3 "
}

eval "$(argc "$0" "$@")"
