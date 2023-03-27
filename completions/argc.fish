# Fish completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

set ARGC_SCRIPTS mycmd1 mycmd2
set -q ARGC_BASH || set ARGC_BASH bash

function __fish_complete_argc
    set -l tokens (commandline -c | string trim -l | string split " " --)
    set -l scriptfile (which $tokens[1])
    if not test -f $scriptfile
        return 0
    end
    set -l line "$tokens[2..]"
    set -l IFS '\n'
    set -l compgen_values (argc --compgen "$scriptfile" "$line" 2>/dev/null)
    set -l candicates
    set -l arg_value
    for item in $compgen_values
        if string match -qr -- '^-' "$item"
            set -a candicates "$item"
        else if string match -qr '^`[^` ]+`' -- "$item"
            set -l name (string sub "$item" -s 2 -e -1)
            set -a candicates ("$ARGC_BASH" "$scriptfile" $name "$line" 2>/dev/null)
        else if string match -q -- '<*' "$item" || string match -q -- '[*'
            set arg_value "$item"
        else
            set -a candicates "$item"
        end
    end
    if test -z "$candicates"
        if string match -qir -- '(file|path)' "$item"
            __fish_complete_path
        else if string match -qir -- 'dir' "$item"
            __fish_complete_directories 
        end
    else if test -n "$arg_value"
        set -a candicates "$arg_value"
    end
    for item in $candicates
        echo $item
    end
end

for argc_script in $ARGC_SCRIPTS
    complete -x -c $argc_script  -n 'true' -a "(__fish_complete_argc)"
end