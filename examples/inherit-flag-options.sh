# @describe How to use `@meta inherit-flag-options`
#
# Mock systemctl cli
# Examples:
#   prog --user start my-service
#   prog --user stop my-service
# 
# @meta inherit-flag-options
# @flag --user Connect to user service manager
# @flag --no-pager Do not pipe output into a pager
# @option -t --type List units of a particular type
# @option --state List units with particular LOAD or SUB or ACTIVE state

# @cmd Start (activate) one or more units
# @arg UNIT... The unit files to start
start() {
    :;
}

# @cmd Stop (deactivate) one or more units
# @arg UNIT... The unit files to stop
stop() {
    :;
}

eval "$(argc --argc-eval "$0" "$@")"