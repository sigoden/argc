use crate::{
    command::Command,
    param::{FlagOptionParam, Param},
    utils::{escape_shell_words, expand_dotenv, META_DOTENV},
    ChoiceValue, DefaultValue,
};
use anyhow::Result;

const UTIL_FNS: [(&str, &str); 5] = [
    (
        "_argc_take_args",
        r#"
_argc_take_args() {
    _argc_take_args_values=()
    _argc_take_args_len=0
    local param="$1" min="$2" max="$3" signs="$4" delimiter="$5"
    local _argc_take_index=$((_argc_index + 1)) _argc_take_value
    if [[ "$_argc_item" == *=* ]]; then
        _argc_take_args_values=("${_argc_item##*=}")
    else
        while [[ $_argc_take_index -lt $_argc_len ]]; do
            _argc_take_value="${argc__args[_argc_take_index]}"
            if [[ -n "$signs" ]] && [[ "$_argc_take_value" =~ ^["$signs"] ]]; then
                break
            fi
            _argc_take_args_values+=("$_argc_take_value")
            _argc_take_args_len=$((_argc_take_args_len + 1))
            if [[ "$_argc_take_args_len" -ge "$max" ]]; then
                break
            fi
            _argc_take_index=$((_argc_take_index + 1))
        done
    fi
    if [[ "${#_argc_take_args_values[@]}" -lt "$min" ]] || [[ "${#_argc_take_args_values[@]}" -gt "$max" ]]; then
        _argc_die "error: invalid value for \`$param\`"
    fi
    if [[ -n "$delimiter" ]] && [[ "${#_argc_take_args_values[@]}" -gt 0 ]]; then
        local item values arr=()
        for item in "${_argc_take_args_values[@]}"; do
            IFS="$delimiter" read -r -a values <<<"$item"
            arr+=("${values[@]}")
        done
        _argc_take_args_values=("${arr[@]}")
    fi
}
"#,
    ),
    (
        "_argc_match_positionals",
        r#"
_argc_match_positionals() {
    _argc_match_positionals_values=()
    _argc_match_positionals_len=0
    local params=("$@")
    local args_len="${#argc__positionals[@]}"
    if [[ $args_len -eq 0 ]]; then
        return
    fi
    local params_len=$# arg_index=0 param_index=0
    while [[ $param_index -lt $params_len && $arg_index -lt $args_len ]]; do
        local takes=0
        if [[ "${params[param_index]}" -eq 1 ]]; then
            if [[ $param_index -eq 0 ]] &&
                [[ $_argc_dash -gt 0 ]] &&
                [[ $params_len -eq 2 ]] &&
                [[ "${params[$((param_index + 1))]}" -eq 1 ]] \
                ; then
                takes=$_argc_dash
            else
                local arg_diff=$((args_len - arg_index)) param_diff=$((params_len - param_index))
                if [[ $arg_diff -gt $param_diff ]]; then
                    takes=$((arg_diff - param_diff + 1))
                else
                    takes=1
                fi
            fi
        else
            takes=1
        fi
        _argc_match_positionals_values+=("$arg_index:$takes")
        arg_index=$((arg_index + takes))
        param_index=$((param_index + 1))
    done
    if [[ $arg_index -lt $args_len ]]; then
        _argc_match_positionals_values+=("$arg_index:$((args_len - arg_index))")
    fi
    _argc_match_positionals_len=${#_argc_match_positionals_values[@]}
    if [[ $params_len -gt 0 ]] && [[ $_argc_match_positionals_len -gt $params_len ]]; then
        local index="${_argc_match_positionals_values[params_len]%%:*}"
        _argc_die "error: unexpected argument \`${argc__positionals[index]}\` found"
    fi
}
"#,
    ),
    (
        "_argc_split_positionals",
        r#"
_argc_split_positionals() {
    _argc_split_positionals_values=()
    local values_index="$1" values_size="$2" delimiter="$3" item values
    local split_values=("${argc__positionals[@]:values_index:values_size}")
    for item in "${split_values[@]}"; do
        IFS="$delimiter" read -r -a values <<<"$item"
        _argc_split_positionals_values+=("${values[@]}")
    done
    local heads=() tails=() tails_index=$((values_index + values_size))
    if [[ $values_index -gt 0 ]]; then
        heads=("${argc__positionals[@]:0:values_index}")
    fi
    if [[ $tails_index -lt ${#argc__positionals[@]} ]]; then
        tails=("${argc__positionals[@]:tails_index}")
    fi
    argc__positionals=("${heads[@]}" "${_argc_split_positionals_values[@]}" "${tails[@]}")
}
"#,
    ),
    (
        "_argc_require_params",
        r#"
_argc_require_params() {
    local message="$1" missed_envs item name render_name
    for item in "${@:2}"; do
        name="${item%%:*}"
        render_name="${item##*:}"
        if [[ -z "${!name}" ]]; then
            missed_envs="$missed_envs"$'\n'"  $render_name"
        fi
    done
    if [[ -n "$missed_envs" ]]; then
        _argc_die "$message$missed_envs"
    fi
}
"#,
    ),
    (
        "_argc_validate_choices",
        r#"
_argc_validate_choices() {
    local render_name="$1" raw_choices="$2" choices item choice concated_choices
    IFS=$'\n' read -d '' -r -a choices <<<"$raw_choices"
    for choice in "${choices[@]}"; do
        if [[ -z "$concated_choices" ]]; then
            concated_choices="$choice"
        else
            concated_choices="$concated_choices, $choice"
        fi
    done
    for item in "${@:3}"; do
        local pass=0 choice
        for choice in "${choices[@]}"; do
            if [[ "$item" == "$choice" ]]; then
                pass=1
            fi
        done
        if [[ $pass -ne 1 ]]; then
            _argc_die "error: invalid value \`$item\` for $render_name"$'\n'"  [possible values: $concated_choices]"
        fi
    done
}
"#,
    ),
];

pub fn build(source: &str, root_name: &str) -> Result<String> {
    let cmd = Command::new(source, root_name)?;
    let output = build_root(&cmd);
    let mut build_block = false;
    let mut insert_at = None;
    let mut newlines = vec![];
    for line in source.split('\n') {
        let trimed_line = line.trim();
        if !build_block && trimed_line.starts_with("eval") && trimed_line.contains(" --argc-eval ")
        {
            insert_at = Some(newlines.len());
        } else if !build_block && trimed_line.contains("# ARGC-BUILD {") {
            build_block = true;
            insert_at = Some(newlines.len());
        } else if build_block {
            if trimed_line.contains("# ARGC-BUILD }") {
                build_block = false;
            }
        } else {
            newlines.push(line.to_string());
        }
    }
    if let Some(insert_at) = insert_at {
        newlines.insert(insert_at, output);
    } else {
        newlines.push(output);
    }
    Ok(newlines.join("\n"))
}

fn build_root(cmd: &Command) -> String {
    let command = build_command(cmd);
    let dotenv = if let Some(value) = cmd.get_metadata(META_DOTENV) {
        format!("\n    {}", expand_dotenv(value))
    } else {
        String::new()
    };
    let (before_hook, after_hook) = cmd.exist_hooks();
    let before_hook = if before_hook {
        "\n    _argc_before"
    } else {
        ""
    };
    let after_hook = if after_hook { "\n    _argc_after" } else { "" };
    let mut util_fns = String::new();
    for (fn_name, util_fn) in UTIL_FNS {
        if command.contains(fn_name) {
            util_fns.push_str(util_fn);
        }
    }

    format!(
        r#"# ARGC-BUILD {{
# This block was generated by argc (https://github.com/sigoden/argc)
# Modifying it manually is not recommended

_argc_run() {{
    if [[ "$1" == "___internal___" ]]; then
        _argc_die "error: no supported param"
    fi
    argc__args=("$(basename "$0" .sh)" "$@")
    argc__positionals=()
    _argc_index=1
    _argc_len="${{#argc__args[@]}}"
    _argc_parse{dotenv}{before_hook}
    if [ -n "$argc__fn" ]; then
        $argc__fn "${{argc__positionals[@]}}"{after_hook}
    fi
}}
{command}{util_fns}
_argc_die() {{
    if [[ $# -eq 0 ]]; then
        cat
    else
        echo "$*" >&2
    fi
    exit 1
}}

_argc_run "$@"

# ARGC-BUILD }}"#
    )
}

fn build_command(cmd: &Command) -> String {
    let suffix = if cmd.paths.is_empty() {
        String::new()
    } else {
        format!("_{}", cmd.paths.join("_"))
    };

    let usage = {
        let usage = cmd.render_help(None);
        let usage = usage.trim();
        format!(
            r#"
_argc_usage{suffix}() {{
    cat <<-'EOF'
{usage}
EOF
    exit
}}
"#
        )
    };

    let mut version = String::new();
    if cmd.exist_version() {
        let version_value = cmd.render_version();
        version = format!(
            r#"
_argc_version{suffix}() {{
    echo {version_value}
    exit
}}
"#
        );
    }

    let parse = build_parse(cmd, &suffix);

    let subcmds = cmd
        .subcommands
        .iter()
        .map(build_command)
        .collect::<Vec<String>>()
        .join("");

    format!(r#"{usage}{version}{parse}{subcmds}"#)
}

fn build_parse(cmd: &Command, suffix: &str) -> String {
    let mut parse_help = {
        let help_flags = cmd.help_flags().join(" | ");
        format!(
            r#"
        {help_flags})
            _argc_usage{suffix}
            ;;"#
        )
    };
    let parse_version = if cmd.exist_version() {
        let version_flags = cmd.version_flags().join(" | ");
        format!(
            r#"
        {version_flags})
            _argc_version{suffix}
            ;;"#
        )
    } else {
        String::new()
    };
    let mut parse_dash = r#"
        --)
            _argc_dash="${#argc__positionals[@]}"
            argc__positionals+=("${argc__args[@]:$((_argc_index + 1))}")
            _argc_index=$_argc_len
            break
            ;;"#
    .to_string();
    let flag_option_signs = cmd.flag_option_signs();
    let parse_flag_options = if !cmd.flag_option_params.is_empty() {
        let parses: Vec<String> = cmd
            .flag_option_params
            .iter()
            .map(|param| build_parse_flag_option(param, &flag_option_signs))
            .collect();
        parses.join("")
    } else {
        String::new()
    };
    let parse_unknown_flag_options = if !cmd.flag_option_params.is_empty() {
        let unknown_flags = flag_option_signs
            .chars()
            .map(|v| format!("{v}?*"))
            .collect::<Vec<String>>()
            .join(" | ");
        format!(
            r#"
        {unknown_flags})
            _argc_die "error: unexpected argument \`$_argc_key\` found"
            ;;"#
        )
    } else {
        String::new()
    };
    let parse_subcommands = if !cmd.subcommands.is_empty() {
        let mut parses: Vec<String> = cmd
            .subcommands
            .iter()
            .map(|subcmd| {
                let paths = subcmd.paths.join("_");
                let names = subcmd.list_names().join(" | ");
                format!(
                    r#"
        {names})
            _argc_index=$((_argc_index + 1))
            _argc_action=_argc_parse_{paths}
            break
            ;;"#
                )
            })
            .collect();

        let subcmd_usages = cmd
            .subcommands
            .iter()
            .map(|subcmd| {
                let names = subcmd.list_names().join(" | ");
                let paths = subcmd.paths.join("_");
                format!(
                    r#"
            {names})
                _argc_usage_{paths}
                ;;"#
                )
            })
            .collect::<Vec<String>>()
            .join("");

        parses.push(format!(
                r#"
        help)
            local help_arg="${{argc__args[$((_argc_index + 1))]}}"
            case "$help_arg" in{subcmd_usages}
            "")
                _argc_usage{suffix}
                ;;
            *)
                _argc_die "error: invalid value \`$help_arg\` for \`<command>\`"$'\n'"  [possible values: $_argc_subcmds]"
                ;;
            esac
            ;;"#
            ));
        parses.join("")
    } else {
        String::new()
    };

    let parse_fallback = if !cmd.subcommands.is_empty() && cmd.positional_params.is_empty() {
        let cmd_paths = cmd.cmd_paths().join("-");
        if let Some(subcmd) = cmd.find_default_subcommand() {
            let paths = subcmd.paths.join("_");
            format!(
                r#"
        *)
            if [[ "${{#argc__positionals[@]}}" -eq 0 ]]; then
                _argc_action=_argc_parse_{paths}
                break
            fi
            ;;"#
            )
        } else {
            format!(
                r#"
        *)
            _argc_die "error: \`{cmd_paths}\` requires a subcommand but one was not provided"$'\n'"  [subcommands: $_argc_subcmds]"
            ;;"#
            )
        }
    } else {
        let terminated = if cmd.positional_params.last().map(|v| v.terminated()) == Some(true) {
            let min = cmd.positional_params.len() - 1;
            format!(
                r#"
            if [[ "${{#argc__positionals[@]}}" -ge {min} ]]; then
                argc__positionals+=("${{argc__args[@]:_argc_index}}")
                _argc_index=$_argc_len
            fi"#
            )
        } else {
            String::new()
        };
        format!(
            r#"
        *)
            argc__positionals+=("$_argc_item")
            _argc_index=$((_argc_index + 1)){terminated}
            ;;"#
        )
    };

    let required_flag_options = build_required_flag_options(cmd);

    let handle = build_handle(cmd, suffix);

    if cmd.delegated() {
        parse_help = String::new();
        parse_dash = String::new();
    }

    let joined_subcmd_names = cmd.list_subcommand_names().join(", ");

    let combined_case = [
        parse_help,
        parse_version,
        parse_dash,
        parse_flag_options,
        parse_subcommands,
        parse_unknown_flag_options,
        parse_fallback,
    ]
    .join("");

    format!(
        r#"
_argc_parse{suffix}() {{
    local _argc_key _argc_action
    local _argc_subcmds="{joined_subcmd_names}"
    while [[ $_argc_index -lt $_argc_len ]]; do
        _argc_item="${{argc__args[_argc_index]}}"
        _argc_key="${{_argc_item%%=*}}"
        case "$_argc_key" in{combined_case}
        esac
    done{required_flag_options}
    if [[ -n "$_argc_action" ]]; then
        $_argc_action
    else{handle}
    fi
}}
"#
    )
}

fn build_parse_flag_option(param: &FlagOptionParam, signs: &str) -> String {
    let names = param.list_names().join(" | ");
    let long_name = param.render_long_name();
    let var_name = param.var_name();
    if param.is_flag {
        if param.id() == "help" || param.id() == "version" {
            return String::new();
        }
        let variant = if param.multiple_occurs() {
            format!("{var_name}=$(({var_name} + 1))")
        } else {
            format!(
                r#"_argc_die "error: the argument \`{long_name}\` cannot be used multiple times""#
            )
        };

        format!(
            r#"
        {names})
            if [[ "$_argc_item" == *=* ]]; then
                _argc_die "error: unexpected value for \`{long_name}\` flag"
            fi
            _argc_index=$((_argc_index + 1))
            if [[ -n "${var_name}" ]]; then
                {variant}
            else
                {var_name}=1
            fi
            ;;"#
        )
    } else {
        let signs = if param.terminated() { "" } else { signs };
        let delimiter = match param.args_delimiter() {
            Some(v) => v.to_string(),
            None => String::new(),
        };
        let render_name_notations = param.render_name_notations();
        let render_first_notation = param.render_first_notation();
        let choice = build_choice(
            "{_argc_take_args_values[@]}",
            &format!("`{render_first_notation}`"),
            param.choice(),
            3,
        );
        let variant = if param.multiple_values() {
            format!(
                r#"
            {var_name}+=("${{_argc_take_args_values[@]}}")"#
            )
        } else {
            format!(
                r#"
            if [[ -z "${var_name}" ]]; then
                {var_name}="${{_argc_take_args_values[0]}}"
            else
                _argc_die "error: the argument \`{long_name}\` cannot be used multiple times"
            fi"#
            )
        };
        let (min, max) = param.args_range();
        format!(
            r#"
        {names})
            _argc_take_args "{render_name_notations}" {min} {max} "{signs}" "{delimiter}"
            _argc_index=$((_argc_index + _argc_take_args_len + 1)){choice}{variant}
            ;;"#
        )
    }
}

fn build_handle(cmd: &Command, suffix: &str) -> String {
    if !cmd.subcommands.is_empty() && cmd.command_fn.is_none() && cmd.positional_params.is_empty() {
        return format!(
            r#"
        _argc_usage{suffix}"#
        );
    }
    let set_argc_fn = match &cmd.command_fn {
        Some(fn_name) => format!(
            r#"
        argc__fn={fn_name}"#
        ),
        None => String::new(),
    };
    let run_help = format!(
        r#"
        if [[ "${{argc__positionals[0]}}" == "help" ]] && [[ "${{#argc__positionals[@]}}" -eq 1 ]]; then
            _argc_usage{suffix}
        fi"#
    );
    let positionals = build_positionals(cmd);

    let default_flag_options = build_default_flag_options(cmd);

    let required_envs = build_required_envs(cmd);

    let envs = build_envs(cmd);

    let output =
        format!("{set_argc_fn}{run_help}{positionals}{default_flag_options}{required_envs}{envs}");
    if output.is_empty() {
        r#"
        :;"#
        .to_string()
    } else {
        output
    }
}

fn build_positionals(cmd: &Command) -> String {
    if cmd.positional_params.is_empty() {
        return String::new();
    }
    let split_args = cmd
        .positional_params
        .iter()
        .map(|param| if param.multiple_values() { "1" } else { "0" })
        .collect::<Vec<&str>>()
        .join(" ");
    let positionals = cmd
        .positional_params
        .iter()
        .enumerate()
        .map(|(index, param)| {
            let var_name = param.var_name();
            let render_value = param.render_value();
            let multiple = param.multiple_values();
            let variant = if multiple {
                match param.args_delimiter() {
                    Some(delimiter) => format!(
                        r#"
            _argc_split_positionals "$values_index" "$values_size" "{delimiter}"
            {var_name}=("${{_argc_split_positionals_values[@]}}")"#
                    ),
                    None => format!(
                        r#"
            {var_name}=("${{argc__positionals[@]:values_index:values_size}}")"#
                    ),
                }
            } else {
                format!(
                    r#"
            {var_name}="${{argc__positionals[values_index]}}""#
                )
            };

            let choice_variable = if multiple {
                format!("{{{var_name}[@]}}")
            } else {
                var_name.clone()
            };
            let choice = build_choice(
                &choice_variable,
                &format!("`{render_value}`"),
                param.choice(),
                3,
            );
            let default = if param.default().is_some() {
                let default = build_default(&var_name, param.default(), 3);
                format!(
                    r#"{default}
            argc__positionals+=("${var_name}")"#
                )
            } else {
                String::new()
            };

            let required = if param.required() {
                format!(
                    r#"
            _argc_die "error: the required environments \`{render_value}\` were not provided""#
                )
            } else {
                String::new()
            };
            let handle_nonexist = format!("{default}{required}");
            let handle_nonexist = if !handle_nonexist.is_empty() {
                format!(
                    r#"
        else{handle_nonexist}"#
                )
            } else {
                String::new()
            };
            format!(
                r#"
        IFS=: read -r values_index values_size <<<"${{_argc_match_positionals_values[{index}]}}"
        if [[ -n "$values_index" ]]; then{variant}{choice}{handle_nonexist}
        fi"#
            )
        })
        .collect::<Vec<String>>()
        .join("");
    format!(
        r#"
        _argc_match_positionals {split_args}
        local values_index values_size{positionals}"#
    )
}

fn build_required_flag_options(cmd: &Command) -> String {
    let required_flag_options: Vec<_> = cmd
        .flag_option_params
        .iter()
        .filter(|v| v.required())
        .collect();
    if required_flag_options.is_empty() {
        return String::new();
    }
    let values = required_flag_options
        .iter()
        .map(|param| {
            let var_name = param.var_name();
            let render_name = param.render_name_notations();
            format!("'{var_name}:{render_name}'")
        })
        .collect::<Vec<String>>()
        .join(" ");
    format!(
        r#"
    _argc_require_params "error: the following required arguments were not provided:" \
        {values}"#
    )
}

fn build_default_flag_options(cmd: &Command) -> String {
    let default_flag_options: Vec<_> = cmd
        .flag_option_params
        .iter()
        .filter(|param| param.default().is_some())
        .collect();
    if default_flag_options.is_empty() {
        return String::new();
    }

    default_flag_options
        .into_iter()
        .map(|param| {
            let var_name = param.var_name();
            let default = build_default(&var_name, param.default(), 3);
            format!(
                r#"
        if [[ -z "${var_name}" ]]; then{default}
        fi"#
            )
        })
        .collect::<Vec<String>>()
        .join("")
}

fn build_required_envs(cmd: &Command) -> String {
    let required_envs: Vec<_> = cmd
        .env_params
        .iter()
        .filter(|param| param.required())
        .collect();
    if required_envs.is_empty() {
        return String::new();
    }
    let values = required_envs
        .into_iter()
        .map(|param| {
            let name = param.var_name();
            format!("{name}:{name}")
        })
        .collect::<Vec<String>>()
        .join(" ");
    format!(
        r#"
        _argc_require_params "error: the following required environments were not provided:" \
            {values}"#
    )
}

fn build_envs(cmd: &Command) -> String {
    if cmd.env_params.is_empty() {
        return String::new();
    }
    cmd.env_params
        .iter()
        .map(|param| {
            let var_name = param.var_name();
            let default = build_default(&format!("export {var_name}"), param.default(), 3);
            let choice = build_choice(
                &var_name,
                &format!(r#"environment variable `{var_name}`"#),
                param.choice(),
                3,
            );
            if default.is_empty() && choice.is_empty() {
                String::new()
            } else if default.is_empty() {
                format!(
                    r#"
        if [[ -n "${var_name}" ]]; then{choice}
        fi"#
                )
            } else if choice.is_empty() {
                format!(
                    r#"
        if [[ -z "${var_name}" ]]; then{default}
        fi"#
                )
            } else {
                format!(
                    r#"
        if [[ -z "${var_name}" ]]; then{default}
        else{choice}
        fi"#
                )
            }
        })
        .collect::<Vec<String>>()
        .join("")
}

fn build_default(var_name: &str, value: Option<&DefaultValue>, indent: usize) -> String {
    let indent = build_indent(indent);
    match value {
        Some(value) => match value {
            DefaultValue::Value(value) => {
                let value = escape_shell_words(value);
                format!("\n{indent}{var_name}={value}")
            }
            DefaultValue::Fn(value) => format!("\n{indent}{var_name}=\"$({value})\""),
        },
        None => String::new(),
    }
}

fn build_choice(
    variable: &str,
    target_name: &str,
    choice: Option<&ChoiceValue>,
    indent: usize,
) -> String {
    let indent = build_indent(indent);
    match choice {
        Some(value) => match value {
            ChoiceValue::Values(values) => {
                let values = values
                    .iter()
                    .map(|v| escape_shell_words(v))
                    .collect::<Vec<String>>()
                    .join(" ");
                format!(
                    r#"
{indent}_argc_validate_choices '{target_name}' "$(printf "%s\n" {values})" "${variable}""#
                )
            }
            ChoiceValue::Fn(fn_name, validate) => {
                if *validate {
                    format!(
                        r#"
{indent}_argc_validate_choices '{target_name}' "$({fn_name})" "${variable}""#
                    )
                } else {
                    String::new()
                }
            }
        },
        None => String::new(),
    }
}

fn build_indent(indent: usize) -> String {
    "    ".repeat(indent)
}
