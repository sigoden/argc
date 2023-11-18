# Environment variables

## User provide

- `ARGC_SHELL_PATH`: Specify the shell/bash path to use for `argc`.
- `ARGC_SCRIPT_NAME`: Specify the script filename to override the default `Argcfile.sh`. e.g. `Taskfile.sh`
- `ARGC_COMPGEN_DESCRIPTION`: If value is 0 or false, the generated completion candidates do not contain descriptions.
- `ARGC_COMPLETIONS_PATH`: Argc-based completion script searching path.
                           Colon-seperated in non-windows OS. Semicolon-seperated in Windows.
                           Only if the arc-based completion script for the `<command>` is under the `ARGC_COMPLETIONS_PATH` or `PATH`, can it enable completion by sourcing `argc --argc-completions bash <command>`.

## Argc injected into Argcfile.sh

- `ARGC_PWD`: Current workdir. Only available in Argcfile.sh.

## Argc injected into choice-fn

- `ARGC_OS`: The OS type
- `ARGC_COMPGEN`: If value is 1, the script is called to generate completion candidates.
                  If value is 0, the scirpt is called to validate a param value.
- `ARGC_CWORD`: The last word in the command line (processed). Used to filter completion candidates.
- `ARGC_LAST_ARG`: The last word in the command line (raw).

The difference between `ARGC_CWORD` and `ARGC_LAST_ARG`:
- If the command line is `git --git-dir=git`, then ARGC_LAST_ARG=`--git-dir=git` ARGC_CWORD=`git`
- If the command line is `bat --theme 'Solarized`, then ARGC_LAST_ARG=`'Solarized` ARGC_CWORD=`Solarized`
