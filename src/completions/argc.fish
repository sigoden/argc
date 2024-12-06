function _argc_completer
    set -g _argc_completer_words
    _argc_completer_parse_line

    argc --argc-compgen fish "" $_argc_completer_words
end

function _argc_completer_parse_line
    set -l line (commandline -cp) 
    set -l word ''
    set -l unbalance ''
    set -l prev_char ''

    for i in (seq (string length -- $line))
        set -l char (string sub -s $i -l 1 -- $line)

        if test -n "$unbalance"
            set word "$word$char"
            if test "$unbalance" = "$char"
                set unbalance ''
            end
        else if test "$char" = ' '
            if test "$prev_char" = '\\'
                set word "$word$char"
            else if test -n "$word"
                set -a _argc_completer_words "$word"
                set word ''
            end
        else if test "$char" = "'" -o "$char" = '"'
            set word "$word$char"
            set unbalance "$char"
        else if test "$char" = '\\'
            if test "$prev_char" = '\\'
                set word "$word$char"
            end
        else
            set word "$word$char"
        end

        set prev_char "$char"
    end

    set -a _argc_completer_words "$word"
end


for cmd in __COMMANDS__
    complete -x -k -c $cmd -a "(_argc_completer)"
end
