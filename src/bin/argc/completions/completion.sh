#!/usr/bin/env bash

# @describe An elegant command-line argument parser - https://github.com/sigoden/argc

# @option --argc-eval~ <FILE> <ARGS>                                Use `eval "$(argc --argc-eval "$0" "$@")"`
# @option --argc-create~ <TASKS>                                    Create a boilerplate argcfile
# @option --argc-build <FILE> <OUTPATH?>                            Generate bashscript without argc dependency
# @option --argc-mangen <FILE> <OUTDIR>                             Generate man pages
# @option --argc-completions~[`_choice_completion`] <SHELL> <CMDS>  Generate shell completion scripts
# @option --argc-compgen~[`_choice_compgen`] <SHELL> <FILE> <ARGS>  Generate completion candidates
# @option --argc-export <FILE>                                      Export command line definitions as json
# @option --argc-parallel~ <FILE> <ARGS>                            Execute argc functions in parallel
# @flag --argc-script-path                                          Print current argcfile path
# @flag --argc-shell-path                                           Print current shell path
# @flag --argc-help                                                 Print help information
# @flag --argc-version                                              Print version information

_choice_completion() { :; }
_choice_compgen() { :; }

command eval "$(argc --argc-eval "$0" "$@")"
