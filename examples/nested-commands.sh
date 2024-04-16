# @describe How to use nested subcommands
# 
# Mock docker cli

# @cmd
builder() { :; }
# @cmd
builder::ls() { :; }
# @cmd
builder::prune() { :; }
# @cmd
builder::rm() { :; }
# @cmd
builder::imagetools() { :; }
# @cmd
builder::imagetools::create() { :; }
# @cmd
builder::imagetools::inspect() { :; }

eval "$(argc --argc-eval "$0" "$@")"