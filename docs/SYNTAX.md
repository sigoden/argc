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
    - [value notaion](#value-notaion)
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
@cmd [help string]
```
Define a subcommand.

## @option
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
### short

A short flag is a - followed by either a bare-character or quoted character, like -f

### long

A long flag is a -- followed by either a bare-word or a string, like --foo

### modifer

- `*`: occur multiple times, is optional
- `+`: occur multiple times, is required
- `!`: is required
- `=value`: has default value
- `[a|b|c]`: choices
- `[=a|b|c]`: choices, the first choice is is default value.

### value notaion

A value notation is set by placing bare-word between `<>` like <FOO>.

It also very helpful when describing the type of input the user should be using, such as FILE, INTERFACE, etc.

## @flag
```
@flag [short] [long] [help string]
```
Define a flag.

## @arg
```
@arg <name>[modifer] [help string]
```
Define a positoinal argument

For examples.
```sh
# @arg pathspec* Files to add content from.
```

### modifer

- `*`: occur multiple times, is optional
- `+`: occur multiple times, is required
- `!`: is required