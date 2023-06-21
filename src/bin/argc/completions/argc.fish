function _argc_completer
    set -l words (commandline -o)
    set -l cur (commandline -t)
    if [ $cur = "" ]
        set -a words ''
    end
    set -l cmd $words[1]
    set -l scriptfile
    if [ "$cmd" = "argc" ]
        set scriptfile (argc --argc-script-path 2>/dev/null)
    else
        set scriptfile (which "$cmd")
    end
    if not test -f "$scriptfile"
        __fish_complete_path
        return
    end
    set -l candidates (argc --argc-compgen fish $scriptfile $words 2>/dev/null)
    if test (count $candidates) -eq 1
        if [ "$candidates[1]" = "__argc_comp:file" ]
            __fish_complete_path $cur
            return
        else if [ "$candidates[1]" = "__argc_comp:dir" ]
            __fish_complete_directories $cur
            return
        end
    end
    for item in $candidates
        echo "$item"
    end
end
