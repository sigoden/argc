function _argc_completer
    set -l args (commandline -o)
    set -l cur (commandline -t)
    if [ ! "$cur" ]
        set -a args ''
    end

    argc --argc-compgen fish "" $args
end
