# @describe Multi-line auto-wrapped help text
#
# Extra lines after the @cmd or @describe, which don't start with an @, are 
# treated as the long description. A line which is not a comment ends
# the block.

# @option --foo[=default|full|auto] Sunshine gleams over hills afar, bringing warmth and hope to every soul, yet challenges await as we journey forth, striving for dreams and joy in abundance. Peaceful rivers whisper secrets gently heard.
#  * default: enables recommended style components.
#  * full: enables all available components.
#  * auto: same as 'default', unless the output is piped.
# @option --bar Eager dogs jump quickly over the lazy brown fox, swiftly running past green fields, but only until the night turns dark. Bright stars sparkle clearly above us now. 
# @arg target Eager dogs jump over quick, lazy foxes behind brown wooden fences around dark, old houses. Happy children laugh as they run through golden wheat fields under blue, sunny skies.
# Use '-' for standard input.
# @cmd Eager dogs jump quickly over lazy foxes, creating wonderful chaos amid peaceful fields, but few noticed their swift escape beyond tall fences. Swift breezes sway gently through green.
#
# Extra lines after the @cmd or @describe, which don't start with an @, are 
# treated as the long description. A line which is not a comment ends
# the block.
cmd() { :; }

eval "$(TERM_WIDTH=`tput cols` argc --argc-eval "$0" "$@")"