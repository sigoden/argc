#!/bin/bash
# @describe   A fictional versioning CLI
# @version    2.17.1 
# @flag       --no-pager                         
# @flag       -p --paginate
# @option     --git-dir

# @cmd        Add file contents to the index
# @option     pathspec*     Files to add content from. 
# @flag       -n --dry-run  Don’t actually add the file
add() {
    echo "invoke add"
}


# @cmd        Update remote refs along with associated objects
# @option     repository!   The "remote" repository that is destination of a push operation.
# @option     refspec+      Specify what destination ref to update with what source object.
push() {
    echo "invoke push"
}

# @cmd        Shows the commit log.
# @flag       --follow      Continue listing the history of a file beyond renames
# @option     --decorate[=short|full|auto|no]  If no --decorate-refs is given, pretend as if all refs were included.
# @option     --grep*       Limit the commits output to ones with log message that matches the specified pattern 
log() {
    echo "invoke log"
}

eval $(target/debug/argc -e $0 "$@")