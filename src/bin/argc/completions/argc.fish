function _argc_complete_impl
    set -l cur $argv[-1]
    set -l candidates (argc --argc-compgen fish $argv 2>/dev/null)
    if test (count $candidates) -eq 1
        if [ $candidates[1] = "__argc_value:file" ]
            __fish_complete_path $cur
            return
        else if [ $candidates[1] = "__argc_value:dir" ]
            __fish_complete_directories $cur
            return
        end
    end
    for item in $candidates
        echo $item
    end
end

function _argc_completer
    set -l args (commandline -o)
    set -l cur (commandline -t)
    if [ $cur = "" ]
        set -a args ''
    end

    set -l scriptfile
    if [ $args[1] = "argc" ]
        set scriptfile (argc --argc-script-path 2>/dev/null)
    else
        set scriptfile (which $args[1])
    end
    if not test -f $scriptfile
        __fish_complete_path $cur
        return
    end

    _argc_complete_impl $scriptfile $args
end
