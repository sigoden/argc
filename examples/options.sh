# @describe    All kind of options
# @option    --oa                   
# @option -b --ob                   short
# @option -c                        short only
# @option    --oc!                  required
# @option    --od*                  
# @option    --oe+                  required + multiple
# @option    --ona <PATH>           value notation
# @option    --onb <FILE> <FILE>    multiple value notations
# @option    --oda=a                default
# @option    --odb=`_default_fn`    default from fn
# @option    --oca[a|b]             choice
# @option    --ocb[=a|b]            choice + default
# @option    --occ*[a|b]            multiple + choice
# @option    --ocd*[+a|b]           required + multiple + choice
# @option    --ofa[`_choice_fn`]    choice from fn
# @option    --ofb[?`_choice_fn`]   choice from fn + no validation
# @option    --ofc*[`_choice_fn`]   multiple + choice from fn
options() {
    :;
}

# @describe   All kind of flags
# @flag     --fa 
# @flag  -b --fb         shoft
# @flag  -c              shoft only
# @flag     --fd*        multiple
# @flag  -e --fe*        short + multiple
flags() {
    :;
}

eval "$(argc --argc-eval "$0" "$@")"