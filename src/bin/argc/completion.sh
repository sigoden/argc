# @option --argc-eval~ <FILE> <ARGS>                Use `eval "$(argc --argc-eval "$0" "$@")"`
# @option --argc-create~ <RECIPES>                  Create a boilerplate argcfile
# @option --argc-run~ <FILE> <ARGS>                 Run a argc-based script
# @option --argc-build <FILE> <OUTPATH?>            Generate bashscript without argc dependency
# @option --argc-mangen <FILE> <OUTDIR>             Generate man pages
# @option --argc-completions <SHELL> <CMDS>         Generate shell completion scripts
# @option --argc-compgen <SHELL> <FILE> <ARGS>      Generate completion candidates
# @option --argc-export <FILE>                      Export command line definitions as json
# @option --argc-parallel~ <FILE> <ARGS>            Run functions in parallel
# @flag --argc-script-path                          Print current argcfile path
# @flag --argc-shell-path                           Print current shell path
# @flag --argc-help                                 Print help information
# @flag --argc-version                              Print version information

eval "$(argc --argc-eval "$0" "$@")"