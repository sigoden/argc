# @meta combine-shorts

# @cmd All kind of options
# @option    --oa                   
# @option -b --ob                   short
# @option -c                        short only
# @option    --oc!                  required
# @option    --od*                  multi-occurs
# @option    --oe+                  required + multi-occurs
# @option    --of*,                 multi-occurs + comma-separated list
# @option    --ona <PATH>           value notation
# @option    --onb <FILE> <FILE>    two-args value notations
# @option    --onc <CMD> <FILE+>    unlimited-args value notations
# @option    --oda=a                default
# @option    --odb=`_default_fn`    default from fn
# @option    --oca[a|b]             choice
# @option    --ocb[=a|b]            choice + default
# @option    --occ*[a|b]            multi-occurs + choice
# @option    --ofa[`_choice_fn`]    choice from fn
# @option    --ofb[?`_choice_fn`]   choice from fn + no validation
# @option    --ofc*[`_choice_fn`]   multi-occurs + choice from fn
# @option    --ofd*,[`_choice_fn`]  multi-occurs + choice from fn + comma-separated list
# @option    --oxa~                 capture all remaining args
options() {
    _debug "$@";
}

# @cmd All kind of flags
# @flag     --fa 
# @flag  -b --fb         short
# @flag  -c              short only
# @flag     --fd*        multi-occurs
# @flag  -e --fe*        short + multi-occurs
flags() {
    _debug "$@";
}

# @cmd Flags or options with single hyphen
# @flag    -fa
# @flag -b -fb
# @flag    -fd*
# @option  -oa
# @option  -od*
# @option  -ona <PATH>
# @option  -oca[a|b]
# @option  -ofa[`_choice_fn`]
options-one-hyphen() {
    _debug "$@";
}

# @cmd Value notation modifier
# @option --oa <VALUE*>           multi values, zero or more
# @option --ob <VALUE+>           multi values, one or more
# @option --oc <VALUE?>           zero or one
options-notation-modifier() {
    _debug "$@";
}

# @cmd All kind of options
# @option     +oa                   
# @option +b  +ob                   short
# @option +c                        short only
# @option     +oc!                  required
# @option     +od*                  multi-occurs
# @option     +oe+                  required + multi-occurs
# @option     +ona <PATH>           value notation
# @option     +onb <FILE> <FILE>    two-args value notations
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
# @option     +ofd*,[`_choice_fn`]  multi-occurs + choice from fn + comma-separated list
# @option     +oxa~                 capture all remaining args
options-plus() {
    _debug "$@";
}

# @cmd All kind of flags
# @flag      +fa 
# @flag  +b  +fb         short
# @flag  +c              short only
# @flag      +fd*        multi-occurs
# @flag  +e  +fe*        short + multi-occurs
flags-plus() {
    _debug "$@";
}

# @cmd Mixed `-` and `+` options
# @option +a -a
# @option -b +b
# @option +c --c
options-mixed() {
    _debug "$@";
}

# @cmd Prefixed option
# @option -X-*[`_choice_fn`]       prefixied + multi-occurs + choice from fn
# @option +X-*[`_choice_fn`]       prefixied + multi-occurs + choice from fn
options-prefixed() {
    _debug "$@";
}

# @cmd Prefixed option
# @option -f --follow:[a|b]       assigned + choice
options-assigned() {
    _debug "$@";
}

# @cmd
# @flag   -a
# @flag      --fa
# @flag   -f --fb*
# @flag      -sa
# @flag      -sb*
# @option -e
# @option    --oa
# @option    --ob*
# @option    --oc <DIR>
# @option -o --od <FILE> <FILE>
# @option    --oe*,
# @option    --ca[x|y|z]
# @option    --cc[`_choice_fn`]
# @option    --cd[?`_choice_fn`]
# @option    --ce*[`_choice_fn`]
# @option -s -soa
test1() {
    _debug "$@";
}

# @cmd
# @option -a --oa!
# @option    --ob+
# @option    --oc+,
# @option    --oca![`_choice_fn`]
# @option    --ocb+[`_choice_fn`]
# @option    --occ+,[`_choice_fn`]
test2() {
    _debug "$@";
}

# @cmd
# @option    --oe=val
# @option    --of=`_default_fn`
# @option    --cb[=x|y|z]
test3() {
    _debug "$@";
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