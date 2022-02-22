#!/bin/bash
# @describe   A fictional versioning CLI
# @version    2.17.1 
# @author     nobody <nobody@example.com>
# @flag       --no-pager                         
# @flag       -p --paginate
# @option     --git-dir

# @cmd        Add file contents to the index
# @arg        pathspec*     Files to add content from. 
# @flag       -n --dry-run  Don’t actually add the file
add() {
    echo "git add"
}

# @cmd        Shows the commit log.
# @arg        refspec*      Specify what destination ref to update with what source object.
# @flag       --follow      Continue listing the history of a file beyond renames
# @option     --decorate[=short|full|auto|no]  If no --decorate-refs is given, pretend as if all refs were included.
# @option     --grep*       Limit the commits output to ones with log message that matches the specified pattern 
log() {
    echo "git log"
}

# @cmd        Update remote refs along with associated objects
# @arg        repository!   The "remote" repository that is destination of a push operation.
# @arg        refspec+      Specify what destination ref to update with what source object.
push() {
    echo "git push"
}


eval $(target/debug/argc $0 "$@")