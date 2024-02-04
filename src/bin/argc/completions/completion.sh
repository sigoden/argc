#!/usr/bin/env bash

# @option --argc-eval~ <FILE> <ARGS>                                Use `eval "$(argc --argc-eval "$0" "$@")"`
# @option --argc-create~ <TASKS>                                    Create a boilerplate argcfile
# @option --argc-build <FILE> [OUTPATH]                             Build standalone bash script without depending on argc
# @option --argc-completions~[`_choice_completion`] <SHELL> <CMDS>  Generate shell completion scripts
# @option --argc-compgen~[`_choice_compgen`] <SHELL> <FILE> <ARGS>  Dynamically generating completion candidates
# @option --argc-export <FILE>                                      Export command line definitions as json
# @option --argc-parallel~ <FILE> <ARGS>                            Execute argc functions in parallel
# @flag --argc-script-path                                          Print current argcfile path
# @flag --argc-help                                                 Print help information
# @flag --argc-version                                              Print version information

_choice_completion() { :; }
_choice_compgen() { :; }

command eval "$(argc --argc-eval "$0" "$@")"
