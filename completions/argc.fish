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
    if [ (count $opts) = 0 ]
        return 0
    else if [ (count $opts) = 1 ]
        if string match -qr '^`[^` ]+`' -- "$opts[1]"
            set -l name (string sub $opts[1] -s 2 -e -1)
            set opts (bash "$argcfile" $name 2>/dev/null)
        end
    end
    for item in $opts
        if test "$item" = "<FILE>" || test "$item" = "<PATH>" || test "$item" = "<FILE>..." || test "$item" = "<PATH>..."
            __fish_complete_path
        else if test "$item" = "<DIR>" || test "$item" = "<DIR>..."
            __fish_complete_directories 
        else
            echo $item
        end
    end
end

for argc_script in $ARGC_SCRIPTS
    complete -x -c $argc_script  -n 'true' -a "(__fish_complete_argc)"
end