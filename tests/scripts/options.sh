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
# @option    --ca[x|y|z]
# @option    --cc[`_choice_fn`]
# @option    --cd[?`_choice_fn`]
# @option    --ce*[`_choice_fn`]
# @option -s -soa
cmda() { :; }

# @cmd
# @option -a --oa!
# @option    --ob+
# @option    --oca![`_choice_fn`]
# @option    --ocb+[`_choice_fn`]
cmdb() { :; }

# @cmd
# @option    --oe=val
# @option    --of=`_default_fn`
# @option    --cb[=x|y|z]
cmdc() { :; }

_default_fn() {
	echo abc
}

_choice_fn() {
	echo abc
	echo def
	echo ghi
}

eval "$(argc --argc-eval "$0" "$@")"