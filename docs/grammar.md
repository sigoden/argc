# Grammar

## @cmd

> **<sup>Syntax</sup>**\
> `@cmd` description<sup>?</sup>


## @alias

> **<sup>Syntax</sup>**\
> cmd-name (`,` cmd-name)<sup>\*</sup>

## @arg

> **<sup>Syntax</sup>**\
> `@arg` arg-name [_modifier_]<sup>?</sup>[_param-value_]<sup>?</sup>
>   [_notation_]<sup>?</sup>
>   description<sup>?</sup>

## @option

> **<sup>Syntax</sup>**\
> `@option` [_short_]<sup>?</sup> [_long_] [_modifier_]<sup>?</sup>[_param-value_]<sup>?</sup>
>   [_notations_]<sup>?</sup>
>   description<sup>?</sup>

## @flag

> **<sup>Syntax</sup>**\
> `@flag` [_short_]<sup>?</sup>[_long_] `*`<sup>?</sup>
>   description<sup>?</sup>

## @env
> **<sup>Syntax</sup>**\
> `@arg` env-name [_modifier_]<sup>?</sup>[_param-value_]<sup>?</sup>
>   [_notation_]<sup>?</sup>
>   description<sup>?</sup>

## @meta

> **<sup>Syntax</sup>**\
> `@meta` meta-name meta-value<sup>?</sup>

## @describe

> **<sup>Syntax</sup>**\
> `@describe` string

## @version

> **<sup>Syntax</sup>**\
> `@version` string

## @author

> **<sup>Syntax</sup>**\
> `@author` string

## short

> **<sup>Syntax</sup>**\
> &nbsp;&nbsp; -[_short-char_] \
> | +[_short-char_]

## long

> **<sup>Syntax</sup>**\
> &nbsp; -- [_long-name_] \
> | -[_long-name_] \
> | +[_long-name_]

## modifier

> **<sup>Syntax</sup>**\
> &nbsp; `!` \
> | `*` \
> | `+` \
> | `*` [_separate-char_] \
> | `+` [_separate-char_]

## param-value

> **<sup>Syntax</sup>**\
> &nbsp; =value \
> | =``` `default-fn` ``` \
> | [[_choices_]] \
> | [=[_choices_]] \
> | [``` `choice-fn` ```] \
> | [?``` `choice-fn` ```]

## choices

> **<sup>Syntax</sup>**\
> _value_ (`|` _value_)<sup>\*</sup>

## notations

> **<sup>Syntax</sup>**\
> ([_notation_] )<sup>\*</sup>  [_notation-last_] 


## notation

> **<sup>Syntax</sup>**\
> `<` value `>`

## notation-last

> **<sup>Syntax</sup>**\
> `<` value [_notation-modifier_]<sup>?</sup> `>`

## notation-modifier

> **<sup>Syntax</sup>**\
> &nbsp; `*` \
> | `+` \
> | `?`

## short-char

A-Z a-z 0-9 `!` `#` `$` `%` `*` `+` `,` `.` `/` `:` `=` `?` `@` `[` `]` `^` `_` `{` `}` `~`

## separate-char

`,` `:` `@` `|` `/`


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
[_separate-char_]: #separate-char
[_long-name_]: #long-name 