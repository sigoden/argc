# Argc

## Example

```sh
#!/bin/bash
# @describe   A fictional versioning CLI
# @version    2.17.1 
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

eval $(argc $0 "$@")
```

```

## Syntax

### @describe

```
@describe [string]
```
Provide cli description.

### @version
```
@version [string]
```
Providde cli version.

### @cmd

```
@cmd [help string]
```
Define a subcommand.

### @option
```
@cmd [short] [long][modifer] [value name] [help string]
```
Define a option.

```sh
# @option -E --regexp  A pattern to search for.
# @option --grep*
# @option --dump-format[=json|yaml]
# @option --shell-arg=-cu 
```
#### modifer

- `*`: multiple, optional
- `+`: multiple, required
- `!`: required
- `=value`: default value
- `[a|b|c]`: with choices
- `[=a|b|c]`: with choices, default value

### @flag
```
@flag [short] [long] [help string]
```
Define a flag.

### @arg
```
@arg <name>[modifer] [help string]
```
Define a positoinal argument

```sh
# @arg pathspec* Files to add content from.
```

#### modifer

- `*`: multiple, optional
- `+`: multiple, required
- `!`: required