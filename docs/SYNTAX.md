# Syntax

- [Syntax](#syntax)
  - [@describe](#describe)
  - [@version](#version)
  - [@author](#author)
  - [@cmd](#cmd)
  - [@option](#option)
    - [short](#short)
    - [long](#long)
    - [modifer](#modifer)
    - [notaion](#notaion)
  - [@flag](#flag)
  - [@arg](#arg)
    - [modifer](#modifer-1)

## @describe

```
@describe [string]
```
## @version
```
@version [string]
```

## @author

```
@author [string]
```

## @cmd

```
@cmd [string]
```
Define a subcommand.

## @option
```
@cmd [short] [long][modifer] [notation] [string]
```
Define value option.

For examples.
```sh
# @option -E --regexp  A pattern to search for.
# @option --grep* <PATTERN>
# @option --dump-format[=json|yaml]
# @option --shell-arg=-cu 
```
### short

A short flag is a - followed by either a bare-character or quoted character, like -f

### long

A long flag is a -- followed by either a bare-word or a string, like --foo

### modifer

- `*`: occur multiple times, optional
- `+`: occur multiple times, required
- `!`: required
- `=value`: default value
- `[a|b|c]`: choices
- `[=a|b|c]`: choices, first is default.

### notaion

A notation is set by placing bare-word between `<>` like <FOO>.

It also very helpful when describing the type of input the user should be using, such as FILE, INTERFACE, etc.

## @flag
```
@flag [short] [long] [help string]
```
Define flag option.

For examples.
```sh
# @flag  --no-pager
# @flag  -q, --quiet Do not print anything to stdout
```

## @arg
```
@arg <name>[modifer] [help string]
```
Define positoinal argument

For examples.
```sh
# @arg pathspec* Files to add content from.
```

### modifer

- `*`: occur multiple times, optional
- `+`: occur multiple times, required
- `!`: required