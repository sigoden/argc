# Argc

## Example

```sh
#!/bin/bash
# @describe   A fictional versioning CLI
# @version    2.17.1 
# @author     nobody <nobody@example.com>
# @flag       --no-pager          Do not pipe Git output into a pager.
# @flag       -p --paginate       Pipe all output into less (or if set, $PAGER)
# @option     --git-dir           Set the path to the repository (".git" directory)

# @cmd        Add file contents to the index
# @arg        pathspec+           Files to add content from. 
# @flag       -n --dry-run        Don’t actually add the file
add() {
    echo "git add"
}

# @cmd        Shows the commit log.
# @arg        refspec*            Specify what destination ref
# @flag       --follow            Continue listing the history
# @option     --decorate[=short|full|auto|no]
# @option     --grep* <REGEX>     Limit the commits output
log() {
    echo "git log"
}

eval $(argc $0 "$@")
```
