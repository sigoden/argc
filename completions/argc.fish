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
    set -l option_values
    set -l value_kind 0
    for item in $compgen_values
        if string match -qr -- '^-' "$item"
            set -a option_values $item
        else if string match -qr '^`[^` ]+`' -- "$item"
            set -l name (string sub "$item" -s 2 -e -1)
            set -a candicates ("$ARGC_BASH" "$scriptfile" $name "$line" 2>/dev/null)
        else if string match -q -- '<*' "$item"
            if string match -qi -- '<args>...' "$item"
                set value_kind 1
            else if string match -qir -- '(file|path)>(\.\.\.)?' "$item"
                set value_kind 2
            else if string match -qir -- 'dir>(\.\.\.)?' "$item"
                set value_kind 3
            else
                set value_kind 9
            end
        else
            set -a candicates $item
        end
    end
    if [ $value_kind -eq 0 ]
        if test -z "$candicates"
            set -a candicates $option_values
        end
    else if [ $value_kind -eq 1 ]
        if test -z "$candicates"
            set -a candicates $option_values
        end
        if test -z "$candicates"
            __fish_complete_path
        end
    else if [ $value_kind -eq 2 ]
        __fish_complete_path
    else if [ $value_kind -eq 3 ]
        __fish_complete_directories 
    end
    for item in $candicates
        echo $item
    end
end

for argc_script in $ARGC_SCRIPTS
    complete -x -c $argc_script  -n 'true' -a "(__fish_complete_argc)"
end