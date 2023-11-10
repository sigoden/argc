# @cmd All kind of options
# @option    --oa                   
# @option -b --ob                   short
# @option -c                        short only
# @option    --oc!                  required
# @option    --od*                  multi-occurs
# @option    --oe+                  required + multi-occurs
# @option    --ona <PATH>           value notation
# @option    --onb <CMD> <FILE>     two-args value notations
# @option    --onc <CMD> <FILE+>    unlimited-args value notations
# @option    --oda=a                default
# @option    --odb=`_default_fn`    default from fn
# @option    --oca[a|b]             choice
# @option    --ocb[=a|b]            choice + default
# @option    --occ*[a|b]            multi-occurs + choice
# @option    --ocd+[a|b]            required + multi-occurs + choice
# @option    --ofa[`_choice_fn`]    choice from fn
# @option    --ofb[?`_choice_fn`]   choice from fn + no validation
# @option    --ofc*[`_choice_fn`]   multi-occurs + choice from fn
# @option    --oxa~                 capture all remaining args
options() {
    :;
}

# @cmd All kind of flags
# @flag     --fa 
# @flag  -b --fb         short
# @flag  -c              short only
# @flag     --fd*        multi-occurs
# @flag  -e --fe*        short + multi-occurs
flags() {
    :;
}

# @cmd Flags or options with single dash
# @flag    -fa
# @flag -b -fb
# @flag    -fd*
# @option  -oa
# @option  -od*
# @option  -ona <PATH>
# @option  -oca[a|b]
# @option  -ofa[`_choice_fn`]
1dash() {
    :;
}


# @cmd All kind of options
# @option     +oa                   
# @option +b  +ob                   short
# @option +c                        short only
# @option     +oc!                  required
# @option     +od*                  multi-occurs
# @option     +oe+                  required + multi-occurs
# @option     +ona <PATH>           value notation
# @option     +onb <CMD> <FILE>     two-args value notations
# @option     +onc <CMD> <FILE+>    unlimited-args value notations
# @option     +oda=a                default
# @option     +odb=`_default_fn`    default from fn
# @option     +oca[a|b]             choice
# @option     +ocb[=a|b]            choice + default
# @option     +occ*[a|b]            multi-occurs + choice
# @option     +ocd+[a|b]            required + multi-occurs + choice
# @option     +ofa[`_choice_fn`]    choice from fn
# @option     +ofb[?`_choice_fn`]   choice from fn + no validation
# @option     +ofc*[`_choice_fn`]   multi-occurs + choice from fn
# @option     +oxa~                 capture all remaining args
plus_options() {
    :;
}


# @cmd All kind of flags
# @flag      +fa 
# @flag  +b  +fb         short
# @flag  +c              short only
# @flag      +fd*        multi-occurs
# @flag  +e  +fe*        short + multi-occurs
plus_flags() {
    :;
}

# @cmd Flags or options with single dash
# @flag    +fa
# @flag +b +fb
# @flag    +fd*
# @option  +oa
# @option  +od*
# @option  +ona <PATH>
# @option  +oca[a|b]
# @option  +ofa[`_choice_fn`]
plus_1dash() {
    :;
}


# @cmd Mixed `-` and `+` options
# @option +b --ob
mix_options() {
    :;
}

_default_fn() {
    whoami
}

_choice_fn() {
    echo abc
    echo def
	echo ghi
}

eval "$(argc --argc-eval "$0" "$@")"