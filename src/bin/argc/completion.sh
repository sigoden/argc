# @option --argc-eval~ <FILE> <ARGS>                    Parse arguments and output shell code to eval
# 
# Example: eval "$(argc --argc-eval "$0" "$@")"

# @option --argc-create~ <-|FILE> <RECIPES>             Create a boilerplate argc script. Use '-' as an alias for Argcfile.sh
#
# Examples:
#     argc --argc-create - build test                   # Create Argcfile.sh
#     argc --argc-create mycli foo bar
#     argc --argc-create /tmp/mycli foo bar

# @option --argc-build <FILE> <OUTPATH?>                Build a standalone script without argc dependency
#
# Examples:
#     argc --argc-build ./examples/demo > /tmp/demo
#     argc --argc-build ./examples/demo /tmp/demo

# @option --argc-mangen <FILE> <OUTDIR>                 Generate man pages for a script
#
# Example: argc --argc-mangen ./examples/demo ./man

# @option --argc-completions <SHELL> <CMDS*>            Generate shell completion scripts
#
# Examples:
#     source <(argc --argc-completions bash mycli)
#     source <(argc --argc-completions zsh mycli)
#     argc --argc-completions powershell mycli | Out-String | Invoke-Expression
#     argc --argc-completions fish mycli | source
#     argc --argc-completions nushell | save --append $nu.config-path

# @option --argc-compgen <SHELL> <-|FILE> <CMD> <ARGS+> Generate completion candidates. Use '-' to resolve the command from $PATH.
#
# Examples:
#     argc --argc-compgen bash - demo download --       # Find demo in $PATH
#     argc --argc-compgen bash ./examples/demo demo download --

# @option --argc-export <FILE> <OUTPATH?>               Export command definitions as JSON
#
# Examples:
#     argc --argc-export ./examples/demo > /tmp/demo.json
#     argc --argc-export ./examples/demo /tmp/demo.json

# @option --argc-parallel~ <FILE> <ARGS>                Run recipes in parallel
#
# Example: argc --argc-parallel "$0" mycli foo --opt val ::: bar val

# @flag --argc-script-path                              Print the path of the current argcfile

# @flag --argc-shell-path                               Print the path of the current shell

# @flag --argc-help                                     Print help information

# @flag --argc-version                                  Print version information

eval "$(argc --argc-eval "$0" "$@")"