# Export Formats

`argc --argc-export` writes the parsed command definition as JSON. This page documents the object shapes produced by that command.

## command

Each command object, including the root object, has the following fields.

| Field | Type | Description |
| --- | --- | --- |
| `name` | string | Root program name or subcommand name. The root name honors `@meta binname`. |
| `describe` | string | Command description. It may contain newlines. |
| `version` | string or `null` | Version declared for this command. |
| `aliases` | string[] | Explicit aliases plus automatic hyphenated aliases. |
| `flag_options` | [flag_option_param](#flag_option_param)[] | Flags and options, including the generated help entry and any generated version entry. |
| `positionals` | [arg_param](#arg_param)[] | Positional arguments in declaration order. |
| `envs` | [env_param](#env_param)[] | Environment declarations, including inherited entries. |
| `subcommands` | [command](#command)[] | Nested commands. Empty for leaf commands. |
| `command_fn` | string or `null` | Shell function invoked for the command, or `null` when none is selected. |
| `metadata` | map<string, string or `null`> (optional) | Raw `@meta` entries. Omitted when no metadata exists. |
| `require_tools` | string[] (optional) | Tools declared directly on this command. Omitted when empty. |
| `combine_shorts` | `true` (optional) | Root only. Present when `@meta combine-shorts` is enabled. |
| `dotenv` | string (optional) | Root only. Present when `@meta dotenv` is configured. The value is `.env` when no path is given. |
| `default_subcommand` | integer (optional) | Zero-based index into `subcommands` identifying the default child. |
| `before_hook` | string (optional) | Root only. Present when `_argc_before` exists. |
| `after_hook` | string (optional) | Root only. Present when `_argc_after` exists. |

## flag_option_param

Each entry in `flag_options` has the following fields.

| Field | Type | Description |
| --- | --- | --- |
| `id` | string | Raw parameter id. Plus-prefixed options use a `plus_` prefix. |
| `long_name` | string | Long name with its prefix, such as `--foo`, `-c`, or `+foo`. |
| `short_name` | string or `null` | Explicit short name, such as `-f`. A short-only declaration such as `@option -c` has `long_name: "-c"` and `short_name: null`. |
| `describe` | string | Parameter description. |
| `flag` | boolean | `true` for a flag, `false` for an option. |
| `notations` | string[] | Value placeholders, uppercased. Empty for flags. |
| `required` | boolean | Whether the parameter is required after defaults are considered. |
| `multiple_values` | boolean | Whether the parameter is multi-valued, either because it may occur multiple times or because one occurrence consumes multiple values. `true` for parameter modifiers `*`/`+`, multiple notation placeholders, `~`, and notation modifiers `*`/`+`; use `num_args` for the per-occurrence value count. |
| `multiple_occurs` | boolean | Whether the parameter may occur multiple times (`*` or `+`). |
| `num_args` | `[integer, integer]` | Minimum and maximum values per occurrence. A maximum of `32767` means unbounded. |
| `delimiter` | string or `null` | Value separator, such as `,`. |
| `terminated` | boolean | Whether `~` captures all remaining arguments. |
| `prefixed` | boolean | Whether this is a prefix option, such as `-X-*`. |
| `assigned` | boolean | Whether the assigned form (`--name=value`) is required. |
| `default` | [default_value](#default_value) or `null` | Default value. |
| `choice` | [choice_value](#choice_value) or `null` | Completion and validation choices. |
| `env` | string or `null` | Bound environment variable name. `$$` bindings use the derived name. |
| `inherited` | boolean | Whether the parameter was inherited via `@meta inherit-flag-options`. |

## arg_param

Each entry in `positionals` has the following fields.

| Field | Type | Description |
| --- | --- | --- |
| `id` | string | Parameter id. |
| `describe` | string | Parameter description. |
| `notation` | string | Placeholder shown in help, usually uppercased. It comes from the parameter name or an explicit `<...>` notation. |
| `required` | boolean | Whether the argument is required after defaults are considered. |
| `multiple` | boolean | Whether the argument accepts multiple values. `*`, `+`, and `~` all set this to `true`. |
| `delimiter` | string or `null` | Value separator, such as `,`. |
| `terminated` | boolean | Whether `~` captures all remaining arguments. |
| `default` | [default_value](#default_value) or `null` | Default value. |
| `choice` | [choice_value](#choice_value) or `null` | Completion and validation choices. |
| `env` | string or `null` | Bound environment variable name. |

## env_param

Each entry in `envs` has the following fields.

| Field | Type | Description |
| --- | --- | --- |
| `id` | string | Environment variable name. |
| `describe` | string | Description. |
| `required` | boolean | Whether the environment variable must be provided. |
| `default` | [default_value](#default_value) or `null` | Default value. |
| `choice` | [choice_value](#choice_value) or `null` | Validation choices. |
| `inherited` | boolean | Whether the declaration was inherited from a parent command. |

## default_value

The `default` field uses one of these shapes.

| Type | JSON shape | Description |
| --- | --- | --- |
| `Value` | `{"type":"Value","value":"value"}` | Literal default value. |
| `Fn` | `{"type":"Fn","value":"fn_name"}` | Default value produced by a shell function. |

## choice_value

The `choice` field uses one of these shapes.

| Type | JSON shape | Description |
| --- | --- | --- |
| `Values` | `{"type":"Values","data":["a","b"]}` | Fixed choices. With the equals form (`[=...]`), the first value is also emitted as `default`. |
| `Fn` | `{"type":"Fn","data":["fn_name",true]}` | Choices produced by a shell function. The second element is the validation flag; it is `false` for the `?` form. |
