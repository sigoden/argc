#!/bin/bash
# @describe   A fictional versioning CLI
# @version    2.17.1 
# @author     nobody <nobody@example.com>
# @flag       --no-pager          Do not pipe Git output into a pager
# @flag       -p --paginate       Pipe all output into less or $PAGER
# @option     --git-dir=.git      Set the path to the repository

# @cmd        Shows the commit log.
# @arg        refspec*            Specify what destination ref
# @flag       --source            Print out the ref name
# @option     --decorate[=no|short|full|auto]
# @option     --grep* <REGEX>     Limit the commits output
log() {
    echo git log ${argc_refspec[@]}
}

# @cmd        Add file contents to the index
# @arg        pathspec+           Files to add content from. 
# @flag       -n --dry-run        Don’t actually add the file
add() {
    echo git add ${argc_pathspec[@]}
}

eval "$(argc -e $0 "$@")"