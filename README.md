# Argc

## Example

```sh
#!/bin/bash
# @describe   A fictional versioning CLI
# @version    2.17.1 
# @author     nobody <nobody@example.com>
# @flag       --no-pager                         
# @flag       -p --paginate
# @option     --git-dir

# @cmd        Add file contents to the index
# @arg        pathspec*           Files to add content from. 
# @flag       -n --dry-run        Don’t actually add the file
add() {
    echo "git add"
}

# @cmd        Shows the commit log.
# @arg        refspec*            Specify what destination ref to update with what source object.
# @flag       --follow            Continue listing the history of a file beyond renames
# @option     --decorate[=short|full|auto|no]  If no --decorate-refs is given, pretend as if all refs were included.
# @option     --grep* <REGEX>     Limit the commits output to ones with log message that matches the specified pattern 
log() {
    echo "git log"
}

# @cmd        Update remote refs along with associated objects
# @arg        repository!         The "remote" repository that is destination of a push operation.
# @arg        refspec+            Specify what destination ref to update with what source object.
push() {
    echo "git push"
}

eval $(target/debug/argc $0 "$@")
```

## Syntax

### @describe

```
@describe [string]
```
### @version
```
@version [string]
```

### @author

```
@author [string]
```

### @cmd

```
@cmd [help string]
```
Define a subcommand.

### @option
```
@cmd [short] [long][modifer] [value notation] [help string]
```
Define a option.

For examples.
```sh
# @option -E --regexp  A pattern to search for.
# @option --grep* <PATTERN>
# @option --dump-format[=json|yaml]
# @option --shell-arg=-cu 
```
#### short

A short flag is a - followed by either a bare-character or quoted character, like -f

#### long

A long flag is a -- followed by either a bare-word or a string, like --foo

#### modifer

- `*`: occur multiple times, is optional
- `+`: occur multiple times, is required
- `!`: is required
- `=value`: has default value
- `[a|b|c]`: choices
- `[=a|b|c]`: choices, the first choice is is default value.

#### value notaion

A value notation is set by placing bare-word between `<>` like <FOO>.

It also very helpful when describing the type of input the user should be using, such as FILE, INTERFACE, etc.

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

For examples.
```sh
# @arg pathspec* Files to add content from.
```

#### modifer

- `*`: occur multiple times, is optional
- `+`: occur multiple times, is required
- `!`: is required