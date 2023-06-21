# @describe Multi-line auto-wrapped help text
#
# Extra lines after the @cmd or @describe, which don't start with an @, are 
# treated as the long description. A line which is not a comment ends
# the block.

# @option --foo Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Neque laoreet suspendisse libero id. 
#  * default: enables recommended style components (default).
#  * full: enables all available components.
#  * auto: same as 'default', unless the output is piped.
# @arg target Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Sed viverra tellus in hac habitasse platea.
# Use '-' for standard input.
# @cmd Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Arcu cursus euismod quis viverra. 
#
# Extra lines after the @cmd or @describe, which don't start with an @, are 
# treated as the long description. A line which is not a comment ends
# the block.
cmd() { :; }

eval "$(TERM_WIDTH=`tput cols` argc --argc-eval "$0" "$@")"