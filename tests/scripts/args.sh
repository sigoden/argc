# @cmd
# @alias a
cmda() { :; }

# @cmd
# @arg val
cmdb() { :; }

# @cmd
# @arg val*
cmdc() { :; }

# @cmd
# @arg val+
cmdd() { :; }

# @cmd
# @arg val!
cmde() { :; }

# @cmd
# @arg val=xyz
cmdf() { :; }

# @cmd
# @arg val=`_default_fn`
cmdg() { :; }

# @cmd
# @arg val[x|y|z]
cmdh() { :; }

# @cmd
# @arg val[=x|y|z]
cmdi() { :; }

# @cmd
# @arg val[`_choice_fn`]
cmdj() { :; }

# @cmd
# @arg val[?`_choice_fn`]
cmdk() { :; }

# @cmd
# @arg val*[`_choice_fn`]
cmdl() { :; }

# @cmd
# @arg val![`_choice_fn`]
cmdm() { :; }

# @cmd
# @arg val+[`_choice_fn`]
cmdn() { :; }

# @cmd
# @arg val <FILE>
cmdo() { :; }

# @cmd
# @arg val1*
# @arg val2*
cmdp() { :; }

# @cmd
# @arg val1!
# @arg val2+
cmdq() { :; }

# @cmd
# @arg val1!
# @arg val2!
# @arg val3!
cmdr() { :; }

_default_fn() {
	echo abc
}

_choice_fn() {
	echo abc
	echo def
	echo ghi
}

eval "$(argc --argc-eval "$0" "$@")"