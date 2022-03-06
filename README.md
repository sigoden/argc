# Argc

A handy way to handle sh/bash cli parameters.

![demo](https://user-images.githubusercontent.com/4012553/156678751-0a72e309-75f2-40eb-bad6-1bcf03402e2e.gif)

How Argc works:

To write a command line program with Argc, we only need to do two things:

1. Describe the options, parameters, and subcommands in comments
2. Call the following command to entrust Argc to process command line parameters for us

```sh
eval "$(argc -e $0 "$@")"
```

Argc will do the following for us:

1. Extract parameter definitions from comments
2. Parse command line arguments
3. If the parameter is abnormal, output error text or help information
4. If everything is normal, output the parsed parameter variable
5. If there is a subcommand, call the subcommand function

We can easily access the corresponding option or parameter through the variable `$argc_<name>`.

Try [examples/demo.sh](examples/demo.sh) your self.

## Tag

Argc generates parsing rules and help documentation based on tags (fields marked with `@` in comments).

 - [@describe](#describe)
 - [@version](#version)
 - [@author](#author)
 - [@cmd](#cmd)
 - [@alias](#alias)
 - [@option](#option)
   - [modifier](#modifier)
   - [notation](#notation)
 - [@flag](#flag)
 - [@arg](#arg)
   - [modifier](#modifier-1)

### @describe

```sh
@describe [string]

# @describe A demo cli
```

Define description

### @version

```sh
@version [string]

# @version 2.17.1 
```

Define version


### @author

```sh
@author [string]

# @author nobody <nobody@example.com>
```

Define author

### @cmd

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

### @alias

```sh
@alias name(,name)+

# @cmd
# @alias t,tst
test() {
}
```
Define alias for a subcommand

### @option

```sh
 @option [short] [long][modifer] [notation] [string]

 ## @option    --foo                A option
 ## @option -f --foo                A option with short alias
 ## @option    --foo <PATH>         A option with notation
 ## @option    --foo!               A required option
 ## @option    --foo*               A option with multiple values
 ## @option    --foo+               A required option with multiple values
 ## @option    --foo[a|b]           A option with choices
 ## @option    --foo[=a|b]          A option with choices and default value
 ## @option -f --foo <PATH>         A option with short alias and notation
```

Define value option

#### modifier

The symbol after the long option name is the modifier

- `*`: occur multiple times, optional
- `+`: occur multiple times, required
- `!`: required
- `=value`: default value
- `[a|b|c]`: choices
- `[=a|b|c]`: choices, first is default.

#### notation

Used to indicate that the option is a value option, other than a flag option.

If not provided, the option name is used as a placeholder by default.

You can use placeholder hint option value types `<NUM>`, `<PATH>`, `<PATTERN>`, `<DATE>`

### @flag

```sh
@flag [short] [long] [help string]

# @flag     --foo       A flag
# @flag  -f --foo       A flag with short alias
```

Define flag option

### @arg

```sh
@arg <name>[modifier] [help string]

# @arg value            A positional argument
# @arg value!           A required positional argument
# @arg value*           A positional argument support multiple values
# @arg value+           A required positional argument support multiple values
# @arg value[a|b]       A positional argument with choices
# @arg value[=a|b]      A positional argument with choices and default value
```
Define positional argument

#### modifier

- `*`: occur multiple times, optional
- `+`: occur multiple times, required
- `!`: required
- `[a|b|c]`: choices
- `[=a|b|c]`: choices, first is default.

## License

Copyright (c) 2022 argc-developers.

argc is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.