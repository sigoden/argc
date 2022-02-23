# @describe Test all aspects

# @cmd Preferred
# @arg        arg1*            A positonal arg
# @flag       -f --flag1       A flag
# @option     -o --opt1        A option
cmd_prefered() {
    print_argc_vars
}

# @cmd Omitted
# @arg        arg1
# @flag       --flag1
# @option     --opt1
cmd_omitted() {
    print_argc_vars
}

# @cmd Options all kindof names
# @option      --opt1           optional
# @option      --opt2!          required
# @option      --opt3*          optional, multiple
# @option      --opt4+          required, multiple
# @option      --opt5=a         optional, default
# @option      --opt6[a|b|c]    choices
# @option      --opt7[=a|b|c]   choices, default
cmd_option_names() {
    print_argc_vars
}

# @cmd Optiona all kindof formats
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

}

# @cmd All kindof flags
# @flag      --foo1
# @flag   -a --foo2    
# @flag      --foo3     With description
# @flag   -b --foo4     With description
cmd_flag_formats() {
    print_argc_vars
}

# @cmd  Poistional One required
# @arg   arg1!   A required arg
cmd_positional_only() {
    print_argc_vars
}

# @cmd  Poistional All required
# @arg   arg1!  A required arg
# @arg   arg2+  A required arg, multiple
cmd_positional_requires() {
    print_argc_vars
}

print_argc_vars() {
    ( set -o posix ; set ) | grep argc_
}

eval $(target/debug/argc -e $0 "$@")