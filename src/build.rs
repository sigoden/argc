use crate::{
    command::Command,
    param::{EnvParam, FlagOptionParam, Param},
    utils::{escape_shell_words, expand_dotenv},
    ChoiceValue, DefaultValue,
};
use anyhow::Result;

const UTIL_FNS: [(&str, &str); 3] = [
    (
        "_argc_take_args",
        r#"
_argc_take_args() {
    _argc_take_args_values=()
    _argc_take_args_len=0
    local param="$1" min="$2" max="$3" signs="$4" delimiter="$5"
    local _argc_take_index=$((_argc_index+1)) _argc_take_value
    if [[ "$_argc_item" == *=* ]]; then
        _argc_take_args_values=( "${_argc_item##*=}" )
    else
        while [[ $_argc_take_index -lt $_argc_len ]]; do
            _argc_take_value="${argc__args[_argc_take_index]}"
            if [[ -n "$signs" ]] && [[ "$_argc_take_value" =~ ^["$signs"] ]]; then
                break
            fi
            _argc_take_args_values+=( "$_argc_take_value" )
            _argc_take_args_len=$((_argc_take_args_len+1))
            if [[ "$_argc_take_args_len" -ge "$max" ]]; then
                break
            fi
            _argc_take_index=$((_argc_take_index+1))
        done
    fi
    if [[ "${#_argc_take_args_values[@]}" -lt "$min" ]] || [[ "${#_argc_take_args_values[@]}" -gt "$max" ]]; then
        _argc_die "error: invalid value for \`$param\`"
    fi
    if [[ -n "$delimiter" ]] && [[ "${#_argc_take_args_values[@]}" -gt 0 ]]; then
        local item values arr=()
        for item in "${_argc_take_args_values[@]}"; do
            IFS="$delimiter" read -r -a values <<< "$item"
            arr+=( "${values[@]}" )
        done
        _argc_take_args_values=( "${arr[@]}" )
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
    local params=( "$@" )
    local args_len="${#argc__positionals[@]}" 
    if [[ $args_len -eq 0 ]]; then
        return
    fi
    local params_len=$# arg_index=0 param_index=0
    while [[ $param_index -lt $params_len && $arg_index -lt $args_len ]]; do
        local takes=0
        if [[ "${params[param_index]}" -eq 1 ]]; then
            if [[ $param_index -eq 0 ]] \
            && [[ $_argc_dash -gt 0 ]] \
            && [[ $params_len -eq 2 ]] \
            && [[ "${params[$((param_index+1))]}" -eq 1 ]] \
            ; then
                takes=$_argc_dash
            else
                local arg_diff=$((args_len-arg_index)) param_diff=$((params_len-param_index))
                if [[ $arg_diff -gt $param_diff ]]; then
                    takes=$((arg_diff - param_diff + 1))
                else
                    takes=1
                fi
            fi
        else
            takes=1
        fi
        _argc_match_positionals_values+=( "$arg_index:$takes" )
        arg_index=$((arg_index+takes))
        param_index=$((param_index+1))
    done
    if [[ $arg_index -lt $args_len ]]; then
        _argc_match_positionals_values+=( "$arg_index:$((args_len-arg_index))" )
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
    local split_values=( "${argc__positionals[@]:values_index:values_size}" )
    for item in "${split_values[@]}"; do
        IFS="$delimiter" read -r -a values <<< "$item"
        _argc_split_positionals_values+=( "${values[@]}" )
    done
    local heads=() tails=() tails_index=$((values_index+values_size))
    if [[ $values_index -gt 0 ]]; then
        heads=( "${argc__positionals[@]:0:values_index}" )
    fi
    if [[ $tails_index -lt ${#argc__positionals[@]} ]]; then
        tails=( "${argc__positionals[@]:tails_index}" )
    fi
    argc__positionals=( "${heads[@]}" "${_argc_split_positionals_values[@]}" "${tails[@]}" )
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
        if !build_block
            && trimed_line.starts_with("eval")
            && trimed_line.contains("argc --argc-eval")
        {
            insert_at = Some(newlines.len());
        } else if !build_block && trimed_line.contains("#ARGC-BUILD {") {
            build_block = true;
            insert_at = Some(newlines.len());
        } else if build_block {
            if trimed_line.contains("#ARGC-BUILD }") {
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
    let dotenv = if let Some(value) = cmd.get_metadata("dotenv") {
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
        r#"#ARGC-BUILD {{
# This block was generated by argc (https://github.com/sigoden/argc)
# Modifying it manually is not recommended

_argc_run() {{
    if [[ "$1" == "___internal___" ]]; then
        _argc_die "error: no supported param"
    fi
    argc__args=( "$(basename "$0" .sh)" "$@" )
    argc__cmd_arg_index=0
    argc__positionals=()
    _argc_index=1
    _argc_len="${{#argc__args[@]}}"
    _argc_parse{dotenv}{before_hook}
    if [ -n "$argc__fn" ]; then
        $argc__fn "${{argc__positionals[@]}}"{after_hook}
    fi
}}{command}{util_fns}
_argc_die() {{
    if [[ $# -eq 0 ]]; then
        cat
    else
        echo "$*" >&2
    fi
    exit 1
}}

_argc_run "$@"
#ARGC-BUILD }}"#
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
}}
"#
        )
    };
    let exist_version = cmd.exist_version();
    let version = if exist_version {
        let version = cmd.render_version();
        format!(
            r#"
_argc_version{suffix}() {{
    echo {version}
}}
"#
        )
    } else {
        String::new()
    };

    let parse = {
        let mut parse_help = {
            let help_flags = cmd.help_flags().join("|");
            format!(
                r#"
        {help_flags})
            _argc_usage{suffix}
            exit
            ;;"#
            )
        };
        let parse_version = if exist_version {
            let version_flags = cmd.version_flags().join("|");
            format!(
                r#"
        {version_flags})
            _argc_version{suffix}
            exit
            ;;"#
            )
        } else {
            String::new()
        };
        let mut parse_dash = r#"
        --)
            _argc_dash="${#argc__positionals[@]}"
            argc__positionals+=( "${argc__args[@]:$((_argc_index+1))}" )
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
                .join("|");
            format!(
                r#"
        {unknown_flags})
            _argc_die "error: unexpected argument \`$_argc_key\` found"
            ;;"#
            )
        } else {
            String::new()
        };
        let subcmd_names = cmd.list_subcommand_names().join(", ");
        let parse_subcommands = if !cmd.subcommands.is_empty() {
            let mut parses: Vec<String> = cmd
                .subcommands
                .iter()
                .map(|subcmd| {
                    let paths = subcmd.paths.join("_");
                    let names = subcmd.list_names().join("|");
                    format!(
                        r#"
        {names})
            argc__cmd_arg_index=$_argc_index
            _argc_index=$((_argc_index+1))
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
                    let names = subcmd.list_names().join("|");
                    let paths = subcmd.paths.join("_");
                    format!(
                        r#"
            {names})
                _argc_usage_{paths}
                exit
                ;;"#
                    )
                })
                .collect::<Vec<String>>()
                .join("");

            parses.push(format!(
                r#"
        help)
            local help_arg="${{argc__args[$((_argc_index+1))]}}"
            case "$help_arg" in{subcmd_usages}
            "")
                _argc_usage{suffix}
                exit
                ;;
            *)
                _argc_die "error: invalid value \`$help_arg\` for \`<command>\`"$'\n'"  [possible values: $_argc_subcmd_names]"
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
            format!(
                r#"
        *)
            _argc_die "error: \`{cmd_paths}\` requires a subcommand but one was not provided"$'\n'"  [subcommands: $_argc_subcmd_names]"
            ;;"#
            )
        } else {
            let terminated = if cmd.positional_params.last().map(|v| v.terminated()) == Some(true) {
                let min = cmd.positional_params.len() - 1;
                format!(
                    r#"
            if [[ "${{#argc__positionals[@]}}" -ge {min} ]]; then
                argc__positionals+=( "${{argc__args[@]:_argc_index}}" )
                _argc_index=$_argc_len
            fi"#
                )
            } else {
                String::new()
            };
            format!(
                r#"
        *)
            argc__positionals+=( "$_argc_item" )
            _argc_index=$((_argc_index+1)){terminated}
            ;;"#
            )
        };

        let required_flag_options: Vec<_> = cmd
            .flag_option_params
            .iter()
            .filter(|v| v.required())
            .collect();
        let required_flag_options = if !required_flag_options.is_empty() {
            let validates: Vec<_> = required_flag_options
                .iter()
                .map(|param| {
                    let var_name = param.var_name();
                    let render_name = param.render_name_notations();
                    format!(
                        r#"
    if [[ -z "${var_name}" ]]; then
        _argc_die "error: the required argument \`{render_name}\` were not provided"
    fi"#
                    )
                })
                .collect();
            validates.join("")
        } else {
            String::new()
        };

        let mut handle = if !cmd.subcommands.is_empty()
            && cmd.command_fn.is_none()
            && cmd.positional_params.is_empty()
        {
            format!(
                r#"
        _argc_usage{suffix}
        exit"#
            )
        } else {
            let set_fn = match &cmd.command_fn {
                Some(fn_name) => format!(
                    r#"
        argc__fn={fn_name}"#
                ),
                None => String::new(),
            };
            let help_only_positional = format!(
                r#"
        if [[ "${{argc__positionals[0]}}" == "help" ]] && [[ "${{#argc__positionals[@]}}" -eq 1 ]]; then
            _argc_usage{suffix}
            exit
        fi"#
            );
            let positionals = build_positionals(cmd);
            let default_flag_options: Vec<_> = cmd
                .flag_option_params
                .iter()
                .filter(|param| param.default().is_some())
                .collect();
            let default_flag_options = if !default_flag_options.is_empty() {
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
            } else {
                String::new()
            };
            let envs = if !cmd.env_params.is_empty() && cmd.command_fn.is_some() {
                cmd.env_params
                    .iter()
                    .map(build_env)
                    .collect::<Vec<String>>()
                    .join("")
            } else {
                String::new()
            };

            format!("{set_fn}{help_only_positional}{positionals}{default_flag_options}{envs}")
        };
        if handle.is_empty() {
            handle = r#"
        :;"#
            .to_string()
        }

        if cmd.delegated() {
            parse_help = String::new();
            parse_dash = String::new();
        }

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
    local _argc_key
    local _argc_action _argc_subcmd_names="{subcmd_names}"
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
    };

    let subcmds = cmd
        .subcommands
        .iter()
        .map(build_command)
        .collect::<Vec<String>>()
        .join("");

    format!(
        r#"
{usage}{version}{parse}{subcmds}"#
    )
}

fn build_parse_flag_option(param: &FlagOptionParam, signs: &str) -> String {
    let names = param.list_names().join("|");
    let long_name = param.render_long_name();
    let var_name = param.var_name();
    if param.is_flag {
        if param.id() == "help" || param.id() == "version" {
            return String::new();
        }
        let variant = if param.multiple_occurs() {
            format!("{var_name}=$(({var_name}+1))")
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
            _argc_index=$((_argc_index+1))
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
            "_argc_take_args_values",
            &render_first_notation,
            param.choice(),
            true,
            3,
        );
        let variant = if param.multiple_values() {
            format!(
                r#"
            {var_name}+=( "${{_argc_take_args_values[@]}}" )"#
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
            _argc_index=$((_argc_index+_argc_take_args_len+1)){choice}{variant}
            ;;"#
        )
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
            {var_name}=( "${{_argc_split_positionals_values[@]}}" )"#
                    ),
                    None => format!(
                        r#"
            {var_name}=( "${{argc__positionals[@]:values_index:values_size}}" )"#
                    ),
                }
            } else {
                format!(
                    r#"
            {var_name}="${{argc__positionals[values_index]}}""#
                )
            };
            let choice = build_choice(&var_name, &render_value, param.choice(), multiple, 3);
            let default = if param.default().is_some() {
                let default = build_default(&var_name, param.default(), 3);
                format!(
                    r#"{default}
            argc__positionals+=( "${var_name}" )"#
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

fn build_env(param: &EnvParam) -> String {
    let var_name = param.var_name();
    let required = if param.required() {
        format!(
            r#"
            argc_die error: the required environments \`{var_name}\` were not provided"#
        )
    } else {
        String::new()
    };
    let default = build_default(&format!("export {var_name}"), param.default(), 3);
    let handle_exist = format!("{required}{default}");
    let handle_nonexist = build_choice(&var_name, &var_name, param.choice(), false, 3);
    if handle_exist.is_empty() && handle_nonexist.is_empty() {
        String::new()
    } else if handle_exist.is_empty() {
        format!(
            r#"
        if [[ -n "${var_name}" ]]; then{handle_nonexist}
        fi"#
        )
    } else if handle_nonexist.is_empty() {
        format!(
            r#"
        if [[ -z "${var_name}" ]]; then{handle_exist}
        fi"#
        )
    } else {
        format!(
            r#"
        if [[ -z "${var_name}" ]]; then{handle_exist}
        else{handle_nonexist}
        fi"#
        )
    }
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
    var_name: &str,
    render_name: &str,
    choice: Option<&ChoiceValue>,
    is_array: bool,
    indent: usize,
) -> String {
    let indent = build_indent(indent);
    match choice {
        Some(value) => match value {
            ChoiceValue::Values(values) => {
                let match_choices = format!("@({})", values.join("|"));
                let render_choices = values.join(", ");
                let possible_values = format!(r#"$'\n'" [possible values: {render_choices}]""#);
                if is_array {
                    format!(
                        r#"
{indent}local _argc_choice_item
{indent}for _argc_choice_item in "${{{var_name}[@]}}"; do
{indent}    if [[ "$_argc_choice_item" != {match_choices} ]]; then
{indent}        _argc_die "error: invalid value \`$_argc_choice_item\` for \`{render_name}\`"{possible_values}
{indent}    fi
{indent}done"#
                    )
                } else {
                    format!(
                        r#"
{indent}if [[ "${var_name}" != {match_choices} ]]; then
{indent}    _argc_die "error: invalid value \`${var_name}\` for \`{render_name}\`"{possible_values}
{indent}fi"#
                    )
                }
            }
            ChoiceValue::Fn(fn_name, validate) => {
                if *validate {
                    let possible_values = r#"$'\n'" [possible values: $(echo "$_argc_choices" | sed ':x {N; s/\n/, /g; bx}')]""#.to_string();
                    if is_array {
                        format!(
                            r#"
{indent}local _argc_choice_item _argc_choices
{indent}_argc_choices="$({fn_name})"
{indent}for _argc_choice_item in "${{{var_name}[@]}}"; do
{indent}    if ! grep -qw "$_argc_choice_item" <<<"$_argc_choices"; then
{indent}        _argc_die "error: invalid value \`$_argc_choice_item\` for \`{render_name}\`"{possible_values}
{indent}    fi
{indent}done"#
                        )
                    } else {
                        format!(
                            r#"
{indent}local _argc_choices
{indent}_argc_choices="$({fn_name})"
{indent}if ! grep -qw "${var_name}" <<<"$_argc_choices"; then
{indent}    _argc_die "error: invalid value \`${var_name}\` for \`{render_name}\`"{possible_values}
{indent}fi"#
                        )
                    }
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
