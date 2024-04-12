# Specification

### `@describe`

Set the description for the command.

> **<sup>Syntax</sup>**\
> `@describe` [_description_]

```sh
# @describe A demo CLI
```

## `@cmd`

Define a subcommand.

> **<sup>Syntax</sup>**\
> `@cmd` [_description_]<sup>?</sup>

```sh
# @cmd Upload a file
upload() {
  echo Run upload
}

# @cmd Download a file
download() {
  echo Run download
}
```

```
USAGE: prog <COMMAND>

COMMANDS:
  upload    Upload a file
  download  Download a file
```

### `@alias`

Set aliases for the subcommand.

> **<sup>Syntax</sup>**\
> [_name_] (`,` [_name_])<sup>\*</sup>

```sh
# @cmd Run tests
# @alias t,tst
test() {
  echo Run test
}
```

```
USAGE: prog <COMMAND>

COMMANDS:
  test  Run tests [aliases: t, tst]
```

### `@arg`

Define a positional argument.

> **<sup>Syntax</sup>**\
> `@arg` [_name_] [_modifier_]<sup>?</sup> [_param-value_]<sup>?</sup>
>   [_bind-env_]<sup>?</sup>
>   [_notation_]<sup>?</sup>
>   [_description_]<sup>?</sup>

```sh
# @arg va
# @arg vb!                        required
# @arg vc*                        multi-values
# @arg vd+                        multi-values + required
# @arg vna <PATH>                 value notation
# @arg vda=a                      default
# @arg vdb=`_default_fn`          default from fn
# @arg vca[a|b]                   choices
# @arg vcb[=a|b]                  choices + default
# @arg vcc*[a|b]                  multi-values + choice
# @arg vcd+[a|b]                  required + multi-values + choice
# @arg vfa[`_choice_fn`]          choice from fn
# @arg vfb[?`_choice_fn`]         choice from fn + no validation
# @arg vfc*[`_choice_fn`]         multi-values + choice from fn
# @arg vfd*,[`_choice_fn`]        multi-values + choice from fn + comma-separated list
# @arg vxa~                       capture all remaining args
# @arg vea $$                     bind-env
# @arg veb $BE <PATH>             bind-named-env
```

### `@option`

Define an option argument.

> **<sup>Syntax</sup>**\
> `@option` [_short_]<sup>?</sup> [_long_] [_modifier_]<sup>?</sup> [_param-value_]<sup>?</sup>
>   [_bind-env_]<sup>?</sup>
>   [_notations_]<sup>?</sup>
>   [_description_]<sup>?</sup>

```sh
# @option    --oa                   
# @option -b --ob                   short
# @option -c                        short only
# @option    --oc!                  required
# @option    --od*                  multi-occurs
# @option    --oe+                  required + multi-occurs
# @option    --of*,                 multi-occurs + comma-separated list
# @option    --ona <PATH>           value notation
# @option    --onb <FILE> <FILE>    two-args value notations
# @option    --onc <CMD> <FILE+>    unlimited-args value notations
# @option    --oda=a                default
# @option    --odb=`_default_fn`    default from fn
# @option    --oca[a|b]             choice
# @option    --ocb[=a|b]            choice + default
# @option    --occ*[a|b]            multi-occurs + choice
# @option    --ocd+[a|b]            required + multi-occurs + choice
# @option    --ofa[`_choice_fn`]    choice from fn
# @option    --ofb[?`_choice_fn`]   choice from fn + no validation
# @option    --ofc*[`_choice_fn`]   multi-occurs + choice from fn
# @option    --ofd*,[`_choice_fn`]  multi-occurs + choice from fn + comma-separated list
# @option    --oxa~                 capture all remaining args
# @option    --oea $$               bind-env
# @option    --oeb $BE <PATH>       bind-named-env
```

### `@flag`

Define a flag argument. Flag is a special option that does not accept any value.

> **<sup>Syntax</sup>**\
> `@flag` [_short_]<sup>?</sup> [_long_]`*`<sup>?</sup>
>   [_bind-env_]<sup>?</sup>
>   [_description_]<sup>?</sup>

```sh
# @flag     --fa 
# @flag  -b --fb         short
# @flag  -c              short only
# @flag     --fd*        multi-occurs
# @flag     --ea $$      bind-env
# @flag     --eb $BE     bind-named-env
```

### `@env`

Define an environment variable.

> **<sup>Syntax</sup>**\
> `@arg` [_NAME_]`!`<sup>?</sup>[_param-value_]<sup>?</sup>
>   [_notation_]<sup>?</sup>
>   [_description_]<sup>?</sup>

```sh
# @env EA                 optional
# @env EB!                required
# @env EC=true            default
# @env EDA[dev|prod]      choices
# @env EDB[=dev|prod]     choices + default
```

### `@meta`

Add a metadata.

> **<sup>Syntax</sup>**\
> `@meta` [_name_] [_value_]<sup>?</sup>

| syntax                       | scope  | description                                                          |
| :--------------------------- | ------ | :------------------------------------------------------------------- |
| `@meta version <value>`      | any    | Set the version for the command.                                     |
| `@meta author <value>`       | any    | Set the author for the command.                                      |
| `@meta dotenv [<path>]`      | root   | Load a dotenv file from a custom path, if persent.                   |
| `@meta symbol <param>`       | any    | Define a symbolic parameter, e.g. `+toolchain`, `@argument-file`.    |
| `@meta man-section <1-8>`    | root   | Override the section for the man page, defaulting to 1.              |
| `@meta default-subcommand`   | subcmd | Set the current subcommand as the default.                           |
| `@meta inherit-flag-options` | root   | Subcommands will inherit the flags/options from their parent.        |
| `@meta no-inherit-env`       | subcmd | Subcommands will not inherit the @env from their parent.             |
| `@meta combine-shorts`       | root   | Short flags/options can be combined, e.g. `prog -xf => prog -x -f `. |


```sh
# @meta version 1.0.0
# @meta author nobody <nobody@example.com>
# @meta dotenv
# @meta dotenv .env.local
# @meta symbol +toolchain[`_choice_fn`]
# @meta man-section 8
```

## Deprecated tags

Deprecated tags can still be used, but are not recommended and may be completely abandoned in the next major version (v2).

### `@version`

Set the version for the command.

```diff
-- # @version 2.17.1
++ # @meta version 2.17.1 
```

### `@author`

Set the author for the command.

```diff
-- # @author nobody <nobody@example.com>
++ # @meta author nobody <nobody@example.com>
```

## Internal components

### short

The short version of the flag / option.

> **<sup>Syntax</sup>**\
> &nbsp;&nbsp; -[_short-char_] \
> | +[_short-char_]

### long

The long version of the flag / option.

> **<sup>Syntax</sup>**\
> &nbsp; -- [_name_] \
> | -[_name_] \
> | +[_name_]

### modifier

> **<sup>Syntax</sup>**\
> &nbsp; `!` \
> | `*` [_separated-char_]<sup>?</sup> \
> | `+` [_separated-char_]<sup>?</sup>

- `!`: required
- `*`: multi-occurs for @option; multi-values for @arg;
- `+`: required + multi

### param-value

> **<sup>Syntax</sup>**\
> &nbsp; =[_value_] \
> | =\`[_fn-name_]\` \
> | [[_choices_]] \
> | [=[_choices_]] \
> | [\`[_fn-name_]\`] \
> | [?\`[_fn-name_]\`]

### choices

> **<sup>Syntax</sup>**\
> [_value_] (`|` [_value_])<sup>\*</sup>

### notations

> **<sup>Syntax</sup>**\
> ([_notation_] )<sup>\*</sup>  [_notation-last_] 


### notation

Placeholder for the argument’s value in the help message / usage.

> **<sup>Syntax</sup>**\
> `<` [_value_]` >`

Notations that will affect the shell completion:

- `FILE`/`PATH`: complete files
- `DIR`: complete directories

### notation-last

> **<sup>Syntax</sup>**\
> `<` [_value_] [_notation-modifier_]<sup>?</sup> `>`

### notation-modifier

> **<sup>Syntax</sup>**\
> &nbsp; `*` \
> | `+` \
> | `?`

- `*`: take zero or multiple values
- `+`: take one or multiple values
- `?`: take zero or one values

### short-char

A-Z a-z 0-9 `!` `#` `$` `%` `*` `+` `,` `.` `/` `:` `=` `?` `@` `[` `]` `^` `_` `{` `}` `~`

### separated-char

`,` `:` `@` `|` `/`

### bind-env

Flags/options bind environment variables

- `$$`: bind environment variable  whose name is derived from the corresponding param name
- `$`[_NAME_]: bind environment variable whose name is *NAME*

## description

Plain text, can be multiple lines.

```sh
# @describe Multi-line auto-wrapped help text
#
# Extra lines after the @describe/@cmd/@option/@flag/@arg/@env, 
# which don't start with an @, are treated as the long description.
# A line which is not a comment ends the block.
```

[_short_]: #short
[_long_]: #long
[_modifier_]: #modifier
[_param-value_]: #param-value
[_choices_]: #choices
[_notations_]: #notations
[_notation_]: #notation
[_notation-last_]: #notation-last
[_notation-modifier_]: #notation-modifier
[_short-char_]: #short-char
[_separated-char_]: #separated-char
[_bind-env_]: #bind-env
[_description_]: #description
[_name_]: #name
[_value_]: #value
[_fn-name_]: #fn-name
[_NAME_]: #name