# Argc

A sh/bash cli framework.

![demo](https://user-images.githubusercontent.com/4012553/156291669-65461f81-4d7e-4c0f-851b-7276196c94f2.gif)

How Argc works:

1. Extract parameter definitions from script comments
2. Parse command line arguments
3. If the parameter is abnormal, output error text or help information
4. If everything is normal, output the parsed parameter variable

## Tag

Argc generates parsing rules and help documentation based on tags (fields marked with `@` in comments).

### @describe

```sh
# @describe [string]

# @describe A fictional versioning CLI
```

Define description

### @version

```sh
# @version [string]

# @version 2.17.1 
```

Define version


### @author

```sh
# @author [string]

# @author nobody <nobody@example.com>
```

Define author

### @cmd

```sh
# @cmd [string]

# @cmd Shows the commit log.
log() {
}
```
Define subcommand

### @option

```sh
# @cmd [short] [long][modifer] [notation] [string]

# @option -j, --threads <NUM>       Number of threads to use.
# @option --grep* <PATTERN>
# @option --dump-format[=json|yaml]
# @option --shell-arg=-cu 
```

Define value option

#### modifer

The symbol after the long option name is the modifier, such as `*` in `--grep*`

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
# @flag [short] [long] [help string]

## @flag  --no-pager
## @flag  -q, --quiet Do not print anything to stdout
```

Define flag option

### @arg

```sh
# @arg <name>[modifer] [help string]

## @arg pathspec* Files to add content from.
```
Define positional arguement

#### modifer

- `*`: occur multiple times, optional
- `+`: occur multiple times, required
- `!`: required

## License

Copyright (c) 2022 argc-developers.

argc is made available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.