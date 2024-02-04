# @meta dotenv
# @env TEST_EA                   optional
# @env TEST_EB!                  required
# @env TEST_EDA=a                default
# @env TEST_EDB=`_default_fn`    default from fn
# @env TEST_ECA[a|b]             choice
# @env TEST_ECB[=a|b]            choice + default
# @env TEST_EFA[`_choice_fn`]    choice from fn

# @cmd
# @env TEST_EA                   override
# @env TEST_NEW                  append
run() {
    _debug
}

main() {
    _debug
}

_debug() {
    printenv | grep ^TEST_ | sort
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