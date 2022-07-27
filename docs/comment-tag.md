
# Comment Tag

Argc generates parsing rules and help documentation based on comment tags (fields marked with `@` in comments).

  - [@describe](#describe)
  - [@version](#version)
  - [@author](#author)
  - [@help](#help)
  - [@cmd](#cmd)
  - [@alias](#alias)
  - [@option](#option)
    - [modifier](#modifier)
    - [notation](#notation)
  - [@flag](#flag)
  - [@arg](#arg)

## @describe

```sh
@describe [string]

# @describe A demo cli
```

Define description

## @version

```sh
@version [string]

# @version 2.17.1 
```

Define version


## @author

```sh
@author [string]

# @author nobody <nobody@example.com>
```

Define author

## @help

```sh
@help [false|string]

# @help false   
# @help Print help information
```
Customize help subcommand.

1. disable help subcommand with `# @help false`
2. custom help subcommand message with `# @help Print help information`

## @cmd

```sh
@cmd [string]

# @cmd Upload a file
upload() {
}

# @cmd Download a file
download() {
}
```
Define subcommand

## @alias

```sh
@alias name(,name)+

# @cmd
# @alias t,tst
test() {
}
```
Define alias for a subcommand

## @option

```sh
 @option [short] [long][modifer] [notation] [string]

 # @option    --foo                A option
 # @option -f --foo                A option with short alias
 # @option    --foo <PATH>         A option with notation
 # @option    --foo!               A required option
 # @option    --foo*               A option with multiple values
 # @option    --foo+               A required option with multiple values
 # @option    --foo=a              A option with default value
 # @option    --foo[a|b]           A option with choices
 # @option    --foo[=a|b]          A option with choices and default value
 # @option    --foo![a|b]          A required option with choices
 # @option -f --foo <PATH>         A option with short alias and notation
```

Define value option

## modifier

The symbol after the long option name is the modifier

- `*`: occur multiple times, optional
- `+`: occur multiple times, required
- `!`: required
- `=value`: default value
- `[a|b|c]`: choices
- `[=a|b|c]`: choices, first is default.
- `![a|b|c]`: choices, required

## notation

Used to indicate that the option is a value option, other than a flag option.

If not provided, the option name is used as a placeholder by default.

You can use placeholder hint option value types `<NUM>`, `<PATH>`, `<PATTERN>`, `<DATE>`

## @flag

```sh
@flag [short] [long] [help string]

# @flag     --foo       A flag
# @flag  -f --foo       A flag with short alias
```

Define flag option

## @arg

```sh
@arg <name>[modifier] [help string]

# @arg value            A positional argument
# @arg value!           A required positional argument
# @arg value*           A positional argument support multiple values
# @arg value+           A required positional argument support multiple values
# @arg value=a          A positional argument with default value
# @arg value[a|b]       A positional argument with choices
# @arg value[=a|b]      A positional argument with choices and default value
# @arg value![a|b]      A required positional argument with choices
```
Define positional argument

The modifier for @arg is same to [modifier for @option](#modifier)
