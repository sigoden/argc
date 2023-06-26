function _argc_complete_impl
    set -l cur $argv[-1]
    set -l candidates (argc --argc-compgen fish $argv 2>/dev/null)
    set -l skip 0
    if test (count $candidates) -gt 0
        if [ $candidates[1] = "__argc_value:file" ]
            set skip 1
            __fish_complete_path $cur
        else if [ $candidates[1] = "__argc_value:dir" ]
            set skip 1
            __fish_complete_directories $cur
        end
    end
    for item in $candidates[(math $skip + 1)..]
        echo $item
    end
end

function _argc_completer
    set -l args (commandline -o)
    set -l cur (commandline -t)
    if [ ! "$cur" ]
        set -a args ''
    end

    set -l scriptfile
    if [ $args[1] = "argc" ]
        set scriptfile (argc --argc-script-path 2>/dev/null)
    else
        set scriptfile (which $args[1])
        if [ ! "$scriptfile" ]
            __fish_complete_path $cur
            return
        end
    end

    _argc_complete_impl "$scriptfile" $args
end
