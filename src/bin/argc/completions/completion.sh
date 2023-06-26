#!/usr/bin/env bash

# @option --argc-eval~ <FILE> <ARGS>                                Use `eval "$(argc --argc-eval "$0" "$@")"`
# @option --argc-create~ <TASKS>                                    Create a boilerplate argcfile
# @option --argc-completions~[`_choice_completion`] <SHELL> <CMDS>  Generate completion scripts for bash,elvish,fish,nushell,powershell,xsh,zsh
# @option --argc-compgen~[`_choice_compgen`] <SHELL> <FILE> <ARGS>  Generate dynamic completion word
# @option --argc-export <FILE>                                      Export command line definitions as json
# @flag --argc-script-path                                          Print current argcfile path
# @flag --argc-help                                                 Print help information
# @flag --argc-version                                              Print version information

_choice_completion() { :; }
_choice_compgen() { :; }

command eval "$(argc --argc-eval "$0" "$@")"
