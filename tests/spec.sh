# @describe Test all aspects
# @version    0.10.0
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

# @cmd Flags all kind of names
# @flag --flag1
# @flag --flag2       This is a flag
# @flag --flag3*
# @flag -f --flag4    This is a flag
cmd_flag_names() {
    print_argc_vars
}

# @cmd Flags without long name
# @flag -a    This is a flag
# @flag -b*
cmd_no_long_flags() {
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

# @cmd Options without long name
# @option      -a                optional
# @option      -b!               required
# @option      -c*               optional, multiple
# @option      -d+               required, multiple
# @option      -e=a              optional, default
# @option      -f[a|b|c]         choices
# @option      -g[=a|b|c]        choices, default
# @option      -i![a|b|c]        required, choices
# @option      -j[`_fn_foo`]     optional, default from fn
# @option      -k[`_fn_bars`]    choices from fn
# @option      -l![`_fn_bars`]   required, choices from fn
# @option      -m <VALUE>        optional, default
cmd_no_long_options() {
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

# @cmd Option with value notations
# @option   --opt1 <DIR>
# @option   --opt2 <DIR> <FILE>
cmd_option_notations() {
    print_argc_vars
}

# @cmd All kind of flags
# @flag      --foo1
# @flag   -a --foo2    
# @flag      --foo3     With description
# @flag   -b --foo4     With description
# @flag   -c --foo5*    Can occure multiple times
cmd_flag_formats() {
    print_argc_vars
}

# @cmd  Positional one required
# @arg   arg1! <DIR>  A required arg1
cmd_positional_only() {
    print_argc_vars
}

# @cmd  Positional all required
# @arg   dir1     A optional arg 1
# @arg   dir2     A optional arg 2
cmd_positional_many() {
    print_argc_vars
}

# @cmd  Positional all required
# @arg   dir1!     A required arg
# @arg   file2+     A required arg 2, multiple
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

# @cmd Nested command
# @help A nested command
# @version 0.1.0
# @option --opt1
cmd_nested_command() {
    print_argc_vars
}

# @cmd Subcommand of nested command
# @alias a
# @option --opt1
# @option --opt2
cmd_nested_command::foo() {
    print_argc_vars
}

# @cmd Nested command2
# @version 0.1.0
# @option --opt1
cmd_nested_command2() {
    :;
}

# @cmd Subcommand of nested command2
# @option --opt1
# @option --opt2
cmd_nested_command2::foo() {
    print_argc_vars
}

cmd_nested_command2::main() {
    print_argc_vars
}

# @cmd Notation values
# @option --opt1 <>
# @option --opt2 <ABC>
# @option --opt3 <ABC DEF>
# @option --opt4 <[ABC DEF]>
# @option --opt5 <<ABC DEF>>
cmd_notation_values() {
    print_argc_vars
}

# @cmd Dynamic compgen
# @arg args*[`_fn_args`]
cmd_dynamic_compgen() {
    print_argc_vars
}

print_argc_vars() {
    ( set -o posix ; set ) | grep argc_
}

_fn_foo() {
    echo foo
}

_fn_bars() {
    echo a1
    echo a2
    echo a3
}

_fn_args() {
    echo $@
    ( set -o posix ; set ) | grep argc_
}

_fn_dup() {
    case $(basename ${SHELL}) in
        zsh)
            _local_func() {
                echo "do this"
            }
            ;;

        bash)
            _local_func() {
                echo "do that instead"
            }
            ;;
    esac
}

eval "$(argc --argc-eval "$0" "$@")"
