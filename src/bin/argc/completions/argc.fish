function _argc_complete_impl
    set -l cur $argv[-1]
    if not test -f $argv[1]
        __fish_complete_path $cur
        return
    end
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

function _argc_complete_locate
    if [ "$argv[1]" = "argc" ]
        argc --argc-script-path 2>/dev/null
    else
        which $argv[1]
    end
end

function _argc_completer
    set -l args (commandline -o)
    set -l cur (commandline -t)
    if [ $cur = "" ]
        set -a args ''
    end
    _argc_complete_impl (_argc_complete_locate $args[1]) $args
end
