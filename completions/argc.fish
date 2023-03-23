# Fish completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

set ARGC_SCRIPTS mycmd1 mycmd2

function __fish_complete_argc
    set -l tokens (commandline -c | string trim -l | string split " " --)
    set -l argcfile (which $tokens[1])
    if test -z $argcfile
        return 0
    end
    set -l IFS '\n'
    set -l opts (argc --compgen "$argcfile" "$tokens[2..]" 2>/dev/null)
    set comp_file 0
    set comp_dir 0
    for opt in $opts
        if string match -q -- '^-' "$opt"
            echo $opt
        else if string match -qr '^`[^` ]+`' -- "$opt"
            set -l name (string sub "$opt" -s 2 -e -1)
            bash "$argcfile" $name 2>/dev/null
        else if string match -q -- '<*' "$opt"
            if string match -qir -- '(file|path)>(\.\.\.)?' "$opt"
                set comp_file 1
            else if string match -qir -- 'dir>(\.\.\.)?' "$opt"
                set comp_dir 1
            else
                echo $opt
            end
        else
            echo $opt
        end
    end
    if [ $comp_file -eq 1 ]
        __fish_complete_path
    else if [ $comp_dir -eq 1 ]
        __fish_complete_directories 
    end
end

for argc_script in $ARGC_SCRIPTS
    complete -x -c $argc_script  -n 'true' -a "(__fish_complete_argc)"
end