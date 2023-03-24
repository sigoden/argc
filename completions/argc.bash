# Bash completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

ARGC_SCRIPTS=( mycmd1 mycmd2 )
ARGC_BASH=${ARGC_BASH:-bash}

_argc_completion() {
    cur="${COMP_WORDS[COMP_CWORD]}"
    COMPREPLY=()
    ( set -o posix ; set ) | grep COMP_ > /tmp/file1
    local scriptfile=$(which ${COMP_WORDS[0]})
    if [[ ! -f "$scriptfile" ]]; then
        return 0
    fi
    local line=${COMP_LINE:${#COMP_WORDS[0]}}
    local IFS=$'\n'
    local compgen_values=($(argc --compgen "$scriptfile" "$line" 2>/dev/null))
    local option_values=()
    local value_kind=0
    local candicates=()
    for item in ${compgen_values[@]}; do
        if [[ "$item" == '-'* ]]; then
            option_values+=( "$item" )
        elif [[ "$item" == \`*\` ]]; then
            local choices=($("$ARGC_BASH" "$scriptfile" "${item:1:-1}" 2>/dev/null))
            candicates=( "${candicates[@]}" "${choices[@]}" )
        elif [[ "$item" == '<'* ]]; then
            if echo "$item" | grep -qi '<args>...'; then
                value_kind=1
            elif echo "$item" | grep -qi '\(file\|path\)>\(\.\.\.\)\?'; then
                value_kind=2
            elif echo "$item" | grep -qi 'dir>\(\.\.\.\)\?'; then
                value_kind=3
            else
                value_kind=9
            fi
        else
            candicates+=( "$item" )
        fi
    done
    if [[ "$value_kind" == 0 ]]; then
        if [[ "${#candicates[@]}" -eq 0 ]]; then
            candicates=( "${option_values[@]}" )
        fi
    elif [[ "$value_kind" == 1 ]]; then
        if [[ "${#candicates[@]}" -eq 0 ]]; then
            candicates=( "${option_values[@]}" )
        fi
        if [[ "${#candicates[@]}" -eq 0 ]]; then
            _filedir
        fi
    elif [[ "$value_kind" == 2 ]]; then
        _filedir
    elif [[ "$value_kind" == 3 ]]; then
        _filedir -d
    fi
    if [[ ${#candicates[@]} -gt 0 ]]; then
        candicates=($(compgen -W "${candicates[*]}" -- "${cur}"))
        if [ ${#candicates[@]} -gt 0 ]; then
            COMPREPLY=( "${COMPREPLY[@]}" $(printf '%q\n' "${candicates[@]}"))
        fi
    fi
}

complete -F _argc_completion ${ARGC_SCRIPTS[@]}

# Perform tilde (~) completion
# @return  True (0) if completion needs further processing,
#          False (> 0) if tilde is followed by a valid username, completions
#          are put in COMPREPLY and no further processing is necessary.
_tilde()
{
    local result=0
    if [[ ${1-} == \~* && $1 != */* ]]; then
        # Try generate ~username completions
        COMPREPLY=($(compgen -P '~' -u -- "${1#\~}"))
        result=${#COMPREPLY[@]}
        # 2>/dev/null for direct invocation, e.g. in the _tilde unit test
        ((result > 0)) && compopt -o filenames 2>/dev/null
    fi
    return "$result"
}

# This function quotes the argument in a way so that readline dequoting
# results in the original argument.  This is necessary for at least
# `compgen' which requires its arguments quoted/escaped:
#
#     $ ls "a'b/"
#     c
#     $ compgen -f "a'b/"       # Wrong, doesn't return output
#     $ compgen -f "a\'b/"      # Good
#     a\'b/c
#
# See also:
# - https://lists.gnu.org/archive/html/bug-bash/2009-03/msg00155.html
# - https://www.mail-archive.com/bash-completion-devel@lists.alioth.debian.org/msg01944.html
# @param $1  Argument to quote
# @param $2  Name of variable to return result to
_quote_readline_by_ref()
{
    if [[ $1 == \'* ]]; then
        # Leave out first character
        printf -v "$2" %s "${1:1}"
    else
        printf -v "$2" %q "$1"

        # If result becomes quoted like this: $'string', re-evaluate in order
        # to drop the additional quoting.  See also:
        # https://www.mail-archive.com/bash-completion-devel@lists.alioth.debian.org/msg01942.html
        if [[ ${!2} == \$\'*\' ]]; then
            local value=${!2:2:-1} # Strip beginning $' and ending '.
            value=${value//'%'/%%} # Escape % for printf format.
            # shellcheck disable=SC2059
            printf -v value "$value" # Decode escape sequences of \....
            local "$2" && _comp_upvars -v "$2" "$value"
        fi
    fi
} # _quote_readline_by_ref()

# Assign variables one scope above the caller
# Usage: local varname [varname ...] &&
#        _comp_upvars [-v varname value] | [-aN varname [value ...]] ...
# Available OPTIONS:
#     -aN  Assign next N values to varname as array
#     -v   Assign single value to varname
# @return  1 if error occurs
# @see https://fvue.nl/wiki/Bash:_Passing_variables_by_reference
_comp_upvars()
{
    if ! (($#)); then
        echo "bash_completion: $FUNCNAME: usage: $FUNCNAME" \
            "[-v varname value] | [-aN varname [value ...]] ..." >&2
        return 2
    fi
    while (($#)); do
        case $1 in
            -a*)
                # Error checking
                [[ ${1#-a} ]] || {
                    echo "bash_completion: $FUNCNAME:" \
                        "\`$1': missing number specifier" >&2
                    return 1
                }
                printf %d "${1#-a}" &>/dev/null || {
                    echo bash_completion: \
                        "$FUNCNAME: \`$1': invalid number specifier" >&2
                    return 1
                }
                # Assign array of -aN elements
                # shellcheck disable=SC2015,SC2140  # TODO
                [[ $2 ]] && unset -v "$2" && eval "$2"=\(\"\$"{@:3:${1#-a}}"\"\) &&
                    shift $((${1#-a} + 2)) || {
                    echo bash_completion: \
                        "$FUNCNAME: \`$1${2+ }$2': missing argument(s)" \
                        >&2
                    return 1
                }
                ;;
            -v)
                # Assign single value
                # shellcheck disable=SC2015  # TODO
                [[ $2 ]] && unset -v "$2" && eval "$2"=\"\$3\" &&
                    shift 3 || {
                    echo "bash_completion: $FUNCNAME: $1:" \
                        "missing argument(s)" >&2
                    return 1
                }
                ;;
            *)
                echo "bash_completion: $FUNCNAME: $1: invalid option" >&2
                return 1
                ;;
        esac
    done
}

# This function performs file and directory completion. It's better than
# simply using 'compgen -f', because it honours spaces in filenames.
# @param $1  If `-d', complete only on directories.  Otherwise filter/pick only
#            completions with `.$1' and the uppercase version of it as file
#            extension.
#
_filedir()
{
    local IFS=$'\n'

    _tilde "${cur-}" || return

    local -a toks
    local reset arg=${1-}

    if [[ $arg == -d ]]; then
        reset=$(shopt -po noglob)
        set -o noglob
        toks=($(compgen -d -- "${cur-}"))
        IFS=' '
        $reset
        IFS=$'\n'
    else
        local quoted
        _quote_readline_by_ref "${cur-}" quoted

        # Munge xspec to contain uppercase version too
        # https://lists.gnu.org/archive/html/bug-bash/2010-09/msg00036.html
        # news://news.gmane.io/4C940E1C.1010304@case.edu
        local xspec=${arg:+"!*.@($arg|${arg^^})"} plusdirs=()

        # Use plusdirs to get dir completions if we have a xspec; if we don't,
        # there's no need, dirs come along with other completions. Don't use
        # plusdirs quite yet if fallback is in use though, in order to not ruin
        # the fallback condition with the "plus" dirs.
        local opts=(-f -X "$xspec")
        [[ $xspec ]] && plusdirs=(-o plusdirs)
        [[ ${BASH_COMPLETION_FILEDIR_FALLBACK-${COMP_FILEDIR_FALLBACK-}} ||
            ! ${plusdirs-} ]] ||
            opts+=("${plusdirs[@]}")

        reset=$(shopt -po noglob)
        set -o noglob
        toks+=($(compgen "${opts[@]}" -- "$quoted"))
        IFS=' '
        $reset
        IFS=$'\n'

        # Try without filter if it failed to produce anything and configured to
        [[ ${BASH_COMPLETION_FILEDIR_FALLBACK-${COMP_FILEDIR_FALLBACK-}} &&
            $arg && ${#toks[@]} -lt 1 ]] && {
            reset=$(shopt -po noglob)
            set -o noglob
            toks+=($(compgen -f ${plusdirs+"${plusdirs[@]}"} -- "$quoted"))
            IFS=' '
            $reset
            IFS=$'\n'
        }
    fi

    if ((${#toks[@]} != 0)); then
        # 2>/dev/null for direct invocation, e.g. in the _filedir unit test
        compopt -o filenames 2>/dev/null
        COMPREPLY+=("${toks[@]}")
    fi
} # _filedir()
