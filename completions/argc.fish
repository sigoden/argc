# Fish completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

set ARGC_SCRIPTS mycmd1 mycmd2

function __fish_complete_argc
    set -l line (commandline -opc) (commandline -ct)
    set -l tokens (echo $line | string trim | string split " " --)
    set -l argcfile (which $tokens[1])
    if test -z $argcfile
        return 0
    end
    set -l opts (argc --compgen "$argcfile" $tokens[2..-1] 2>/dev/null)
    if string match -q "__argc_compgen_cmd:*" -- $opts
        set -l fn_name (string replace "__argc_compgen_cmd:" "" $opts)
        set opts (bash "$argcfile" $fn_name 2>/dev/null)
    end
    echo $opts | string trim | string split " " --
end

for argc_script in $ARGC_SCRIPTS
    complete -x -c $argc_script  -n 'true' -a "(__fish_complete_argc)"
end