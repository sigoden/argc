use crate::param::{
    ChoiceValue, DefaultValue, EnvParam, FlagOptionParam, Modifier, Param, ParamData,
    PositionalParam,
};
use crate::utils::is_special_var_char;
use crate::Result;
use anyhow::bail;
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_till, take_while1},
    character::{
        complete::{anychar, char, one_of, satisfy, space0, space1},
        streaming::none_of,
    },
    combinator::{eof, fail, map, not, opt, peek, rest, success},
    error::ErrorKind,
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Event {
    pub(crate) data: EventData,
    pub(crate) position: Position,
}

pub(crate) type Position = usize;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum EventData {
    /// Description
    Describe(String),
    /// Version info
    Version(String),
    /// Author info
    Author(String),
    /// Metadata
    Meta(String, String),
    /// Define a subcommand, e.g. `@cmd A sub command`
    Cmd(String),
    /// Define alias for a subcommand, e.g. `@alias t,tst`
    Aliases(Vec<String>),
    /// Define a env param
    Env(EnvParam),
    /// Define a flag or option parameter
    FlagOption(FlagOptionParam),
    /// Define a positional parameter
    Positional(PositionalParam),
    /// A shell function. e.g `function cmd()` or `cmd()`
    Func(String),
    /// Placeholder for unknown or invalid tag
    Unknown(String),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EventScope {
    Root,
    CmdStart,
    FnEnd,
}

impl Default for EventScope {
    fn default() -> Self {
        Self::Root
    }
}

/// Tokenize shell script
pub(crate) fn parse(source: &str) -> Result<Vec<Event>> {
    let mut result = vec![];
    let lines: Vec<&str> = source.lines().collect();
    let mut line_idx = 0;
    while line_idx < lines.len() {
        let line = lines[line_idx];
        let position = line_idx + 1;
        match parse_line(line) {
            Ok((_, maybe_token)) => {
                if let Some(maybe_data) = maybe_token {
                    if let Some(data) = maybe_data {
                        let data = match data {
                            EventData::Describe(mut text) => {
                                line_idx += take_comment_lines(&lines, line_idx + 1, &mut text);
                                EventData::Describe(text)
                            }
                            EventData::Cmd(mut text) => {
                                line_idx += take_comment_lines(&lines, line_idx + 1, &mut text);
                                EventData::Cmd(text)
                            }
                            EventData::Env(mut param) => {
                                line_idx +=
                                    take_comment_lines(&lines, line_idx + 1, param.describe_mut());
                                EventData::Env(param)
                            }
                            EventData::FlagOption(mut param) => {
                                line_idx +=
                                    take_comment_lines(&lines, line_idx + 1, param.describe_mut());
                                EventData::FlagOption(param)
                            }
                            EventData::Positional(mut param) => {
                                line_idx +=
                                    take_comment_lines(&lines, line_idx + 1, param.describe_mut());
                                EventData::Positional(param)
                            }
                            v => v,
                        };
                        result.push(Event { position, data });
                    } else {
                        bail!("syntax error at line {}", position)
                    }
                }
            }
            Err(err) => {
                bail!("fail to parse at line {}, {}", position, err)
            }
        }
        line_idx += 1;
    }
    Ok(result)
}

pub(crate) fn parse_symbol(input: &str) -> Option<(char, &str, Option<&str>)> {
    let input = input.trim();
    parse_symbol_data(input).map(|(_, v)| v).ok()
}

fn parse_line(line: &str) -> nom::IResult<&str, Option<Option<EventData>>> {
    alt((map(alt((parse_tag, parse_fn)), Some), success(None)))(line)
}

fn parse_fn(input: &str) -> nom::IResult<&str, Option<EventData>> {
    map(alt((parse_fn_keyword, parse_fn_no_keyword)), |v| {
        Some(EventData::Func(v.to_string()))
    })(input)
}

// Parse fn likes `function foo`
fn parse_fn_keyword(input: &str) -> nom::IResult<&str, &str> {
    preceded(terminated(tag("function"), space1), parse_fn_name)(input)
}

// Parse fn likes `foo ()`
fn parse_fn_no_keyword(input: &str) -> nom::IResult<&str, &str> {
    terminated(parse_fn_name, tuple((space0, char('('), space0, char(')'))))(input)
}

fn parse_tag(input: &str) -> nom::IResult<&str, Option<EventData>> {
    preceded(
        tuple((many1(char('#')), space0, char('@'))),
        alt((
            parse_tag_text,
            parse_tag_meta,
            parse_tag_param,
            parse_tag_alias,
            parse_tag_unknown,
        )),
    )(input)
}

fn parse_tag_text(input: &str) -> nom::IResult<&str, Option<EventData>> {
    map(
        pair(
            alt((tag("describe"), tag("version"), tag("author"), tag("cmd"))),
            parse_tail,
        ),
        |(tag, text)| {
            let text = text.to_string();
            Some(match tag {
                "describe" => EventData::Describe(text),
                "version" => EventData::Version(text),
                "author" => EventData::Author(text),
                "cmd" => EventData::Cmd(text),
                _ => unreachable!(),
            })
        },
    )(input)
}

fn parse_tag_meta(input: &str) -> nom::IResult<&str, Option<EventData>> {
    preceded(
        tag("meta"),
        map(preceded(space1, parse_key_value), |kv| {
            kv.map(|(k, v)| EventData::Meta(k.to_string(), v.to_string()))
        }),
    )(input)
}

fn parse_tag_param(input: &str) -> nom::IResult<&str, Option<EventData>> {
    let check = peek(alt((tag("option"), tag("flag"), tag("env"), tag("arg"))));
    let arg = alt((
        map(
            preceded(pair(tag("flag"), space1), parse_flag_param),
            |param| Some(EventData::FlagOption(param)),
        ),
        map(
            preceded(pair(tag("option"), space1), parse_option_param),
            |param| Some(EventData::FlagOption(param)),
        ),
        map(
            preceded(pair(tag("env"), space1), parse_env_param),
            |param| Some(EventData::Env(param)),
        ),
        map(
            preceded(pair(tag("arg"), space1), parse_positional_param),
            |param| Some(EventData::Positional(param)),
        ),
    ));
    preceded(check, alt((arg, success(None))))(input)
}

fn parse_tag_alias(input: &str) -> nom::IResult<&str, Option<EventData>> {
    map(
        pair(tag("alias"), preceded(space1, parse_name_list)),
        |(tag, list)| {
            Some(match tag {
                "alias" => EventData::Aliases(list.iter().map(|v| v.to_string()).collect()),
                _ => unreachable!(),
            })
        },
    )(input)
}

fn parse_tag_unknown(input: &str) -> nom::IResult<&str, Option<EventData>> {
    map(parse_name, |v| Some(EventData::Unknown(v.to_string())))(input)
}

// Parse `@option`
fn parse_option_param(input: &str) -> nom::IResult<&str, FlagOptionParam> {
    alt((parse_with_long_option_param, parse_no_long_option_param))(input)
}

// Parse `@option` with long name
fn parse_with_long_option_param(input: &str) -> nom::IResult<&str, FlagOptionParam> {
    map(
        tuple((
            parse_with_long_head,
            alt((
                parse_param_modifier_choices_default,
                parse_param_modifier_choices_fn,
                parse_param_modifier_choices,
                parse_param_assign_fn,
                parse_param_assign,
                parse_param_modifier,
            )),
            parse_zero_or_one_bind_env,
            parse_zero_or_many_value_notations,
            parse_tail,
        )),
        |((short, long_prefix), mut arg, env, value_names, describe)| {
            arg.env = env;
            arg.describe = describe.to_string();
            FlagOptionParam::new(arg, false, short, long_prefix, &value_names)
        },
    )(input)
}

// Parse `@option` without long name
fn parse_no_long_option_param(input: &str) -> nom::IResult<&str, FlagOptionParam> {
    map(
        tuple((
            preceded(space0, alt((tag("-"), tag("+")))),
            preceded(
                verify_single_char,
                alt((
                    parse_param_modifier_choices_default,
                    parse_param_modifier_choices_fn,
                    parse_param_modifier_choices,
                    parse_param_assign_fn,
                    parse_param_assign,
                    parse_param_modifier,
                )),
            ),
            parse_zero_or_one_bind_env,
            parse_zero_or_many_value_notations,
            parse_tail,
        )),
        |(long_prefix, mut arg, env, value_names, describe)| {
            arg.env = env;
            arg.describe = describe.to_string();
            FlagOptionParam::new(arg, false, None, long_prefix, &value_names)
        },
    )(input)
}

// Parse `@env`
fn parse_env_param(input: &str) -> nom::IResult<&str, EnvParam> {
    map(
        pair(
            alt((
                parse_param_modifier_choices_default,
                parse_param_modifier_choices_fn,
                parse_param_modifier_choices,
                parse_param_assign_fn,
                parse_param_assign,
                parse_param_modifier,
            )),
            parse_tail,
        ),
        |(mut arg, describe)| {
            arg.describe = describe.to_string();
            EnvParam::new(arg)
        },
    )(input)
}

// Parse `@option`, positional only
fn parse_positional_param(input: &str) -> nom::IResult<&str, PositionalParam> {
    map(
        tuple((
            alt((
                parse_param_modifier_choices_default,
                parse_param_modifier_choices_fn,
                parse_param_modifier_choices,
                parse_param_assign_fn,
                parse_param_assign,
                parse_param_modifier,
            )),
            parse_zero_or_one_bind_env,
            parse_zero_or_one_value_notation,
            parse_tail,
        )),
        |(mut arg, env, value_name, describe)| {
            arg.env = env;
            arg.describe = describe.to_string();
            PositionalParam::new(arg, value_name)
        },
    )(input)
}

// Parse `@flag`
fn parse_flag_param(input: &str) -> nom::IResult<&str, FlagOptionParam> {
    alt((parse_with_long_flag_param, parse_no_long_flag_param))(input)
}

// Parse `@flag`
fn parse_with_long_flag_param(input: &str) -> nom::IResult<&str, FlagOptionParam> {
    map(
        tuple((
            parse_with_long_head,
            parse_with_long_flag_name,
            parse_zero_or_one_bind_env,
            parse_tail,
        )),
        |((short, long_prefix), mut arg, env, describe)| {
            arg.env = env;
            arg.describe = describe.to_string();
            FlagOptionParam::new(arg, true, short, long_prefix, &[])
        },
    )(input)
}

// Parse `@flag` without long name
fn parse_no_long_flag_param(input: &str) -> nom::IResult<&str, FlagOptionParam> {
    map(
        tuple((
            preceded(space0, alt((tag("-"), tag("+")))),
            parse_no_long_flag_name,
            parse_zero_or_one_bind_env,
            parse_tail,
        )),
        |(long_prefix, mut arg, env, describe)| {
            arg.env = env;
            arg.describe = describe.to_string();
            FlagOptionParam::new(arg, true, None, long_prefix, &[])
        },
    )(input)
}

fn parse_with_long_flag_name(input: &str) -> nom::IResult<&str, ParamData> {
    alt((
        map(terminated(parse_param_name, tag("*")), |mut arg| {
            arg.modifier = Modifier::MultipleOptional;
            arg
        }),
        parse_param_name,
    ))(input)
}

fn parse_no_long_flag_name(input: &str) -> nom::IResult<&str, ParamData> {
    fn parser(input: &str) -> nom::IResult<&str, ParamData> {
        map(satisfy(is_short_char), |ch| {
            ParamData::new(&format!("{}", ch))
        })(input)
    }
    map(pair(parser, opt(tag("*"))), |(mut arg, multiple)| {
        if multiple.is_some() {
            arg.modifier = Modifier::MultipleOptional;
        }
        arg
    })(input)
}

fn parse_with_long_head(input: &str) -> nom::IResult<&str, (Option<&str>, &str)> {
    map(
        alt((pair(
            opt(terminated(
                pair(
                    opt(alt((char::<&str, _>('-'), char('+')))),
                    satisfy(is_short_char),
                ),
                peek(space1),
            )),
            preceded(
                space0,
                alt((
                    terminated(tag("--"), peek(not(char('-')))),
                    terminated(tag("-"), peek(not(char('-')))),
                    terminated(tag("+"), peek(not(char('+')))),
                )),
            ),
        ),)),
        |(short, long_prefix)| (short.map(|_| &input[0..2]), long_prefix),
    )(input)
}

// Parse `str!` `str~` `str*` `str+` `str`
fn parse_param_modifier(input: &str) -> nom::IResult<&str, ParamData> {
    alt((
        map(terminated(parse_param_name, tag("!")), |mut arg| {
            arg.modifier = Modifier::Required;
            arg
        }),
        map(terminated(parse_param_name, tag("~")), |mut arg| {
            arg.modifier = Modifier::Terminated;
            arg
        }),
        map(
            pair(parse_param_name, preceded(tag("*"), opt(parse_multi_char))),
            |(mut arg, multi_char)| {
                match multi_char {
                    Some(c) => arg.modifier = Modifier::DelimiterOptional(c),
                    None => arg.modifier = Modifier::MultipleOptional,
                };
                arg
            },
        ),
        map(
            pair(parse_param_name, preceded(tag("+"), opt(parse_multi_char))),
            |(mut arg, multi_char)| {
                let modifier = match multi_char {
                    Some(c) => Modifier::DelimiterRequired(c),
                    None => Modifier::MultipleRequired,
                };
                arg.modifier = modifier;
                arg
            },
        ),
        parse_param_name,
    ))(input)
}

// Parse `str=value`
fn parse_param_assign(input: &str) -> nom::IResult<&str, ParamData> {
    map(
        separated_pair(parse_param_name, char('='), parse_default_value),
        |(mut arg, value)| {
            arg.default = Some(DefaultValue::Value(value.to_string()));
            arg
        },
    )(input)
}

// Parse str=`value`
fn parse_param_assign_fn(input: &str) -> nom::IResult<&str, ParamData> {
    map(
        separated_pair(parse_param_name, char('='), parse_value_fn),
        |(mut arg, f)| {
            arg.default = Some(DefaultValue::Fn(f.to_string()));
            arg
        },
    )(input)
}

fn parse_param_modifier_choices_default(input: &str) -> nom::IResult<&str, ParamData> {
    map(
        pair(
            parse_param_modifier,
            delimited(char('['), parse_choices_default, char(']')),
        ),
        |(mut arg, (values, default))| {
            arg.choice = Some(ChoiceValue::Values(
                values.iter().map(|v| v.to_string()).collect(),
            ));
            if let Some(value) = default {
                arg.default = Some(DefaultValue::Value(value.to_string()));
            }
            arg
        },
    )(input)
}

fn parse_param_modifier_choices(input: &str) -> nom::IResult<&str, ParamData> {
    map(
        pair(
            parse_param_modifier,
            delimited(char('['), parse_choices, char(']')),
        ),
        |(mut arg, values)| {
            arg.choice = Some(ChoiceValue::Values(
                values.iter().map(|v| v.to_string()).collect(),
            ));
            arg
        },
    )(input)
}

fn parse_param_modifier_choices_fn(input: &str) -> nom::IResult<&str, ParamData> {
    map(
        pair(
            parse_param_modifier,
            delimited(char('['), pair(opt(char('?')), parse_value_fn), char(']')),
        ),
        |(mut arg, (validate, f))| {
            arg.choice = Some(ChoiceValue::Fn(f.into(), validate.is_none()));
            arg
        },
    )(input)
}

fn parse_param_name(input: &str) -> nom::IResult<&str, ParamData> {
    map(parse_name, ParamData::new)(input)
}

// Zero or many '<FOO>'
fn parse_zero_or_many_value_notations(input: &str) -> nom::IResult<&str, Vec<&str>> {
    many0(parse_value_notation)(input)
}

// Zero or one '<FOO>'
fn parse_zero_or_one_value_notation(input: &str) -> nom::IResult<&str, Option<&str>> {
    opt(parse_value_notation)(input)
}

// Parse '<FOO>'
fn parse_value_notation(input: &str) -> nom::IResult<&str, &str> {
    preceded(
        char(' '),
        delimited(char('<'), parse_notation_text, char('>')),
    )(input)
}

fn parse_bind_env_name(input: &str) -> nom::IResult<&str, &str> {
    take_while1(is_env_name_char)(input)
}

// Parse `a|b|c`
fn parse_choices(input: &str) -> nom::IResult<&str, Vec<&str>> {
    map(separated_list1(char('|'), parse_choice_value), |choices| {
        choices
    })(input)
}

// Parse `=a|b|c`
fn parse_choices_default(input: &str) -> nom::IResult<&str, (Vec<&str>, Option<&str>)> {
    map(
        tuple((
            char('='),
            parse_choice_value,
            many1(preceded(char('|'), parse_choice_value)),
        )),
        |(_, head, tail)| {
            let mut choices = vec![head];
            choices.extend(tail);
            (choices, Some(head))
        },
    )(input)
}

fn parse_tail(input: &str) -> nom::IResult<&str, &str> {
    alt((
        eof,
        preceded(space1, alt((eof, map(rest, |v: &str| v.trim())))),
    ))(input)
}

fn parse_key_value(input: &str) -> nom::IResult<&str, Option<(&str, &str)>> {
    let input = input.trim_end();
    let key_value = map(pair(parse_name, parse_tail), |(key, value)| {
        Some((key, value))
    });

    alt((key_value, success(None)))(input)
}

fn parse_name_list(input: &str) -> nom::IResult<&str, Vec<&str>> {
    separated_list1(char(','), delimited(space0, parse_name, space0))(input)
}

fn parse_fn_name(input: &str) -> nom::IResult<&str, &str> {
    take_while1(|c| {
        !matches!(
            c,
            ' ' | '\t'
                | '"'
                | '\''
                | '`'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '<'
                | '>'
                | '$'
                | '&'
                | '\\'
                | ';'
                | '|'
        )
    })(input)
}

fn parse_name(input: &str) -> nom::IResult<&str, &str> {
    take_while1(is_name_char)(input)
}

fn parse_multi_char(input: &str) -> nom::IResult<&str, char> {
    one_of(",:;@|/")(input)
}

fn parse_default_value(input: &str) -> nom::IResult<&str, &str> {
    alt((parse_quoted_string, take_till(is_default_value_terminate)))(input)
}

fn parse_value_fn(input: &str) -> nom::IResult<&str, &str> {
    delimited(char('`'), parse_fn_name, char('`'))(input)
}

fn parse_choice_value(input: &str) -> nom::IResult<&str, &str> {
    if input.starts_with('=') || input.starts_with('`') {
        return fail(input);
    }
    alt((parse_quoted_string, take_till(is_choice_value_terminate)))(input)
}

fn parse_quoted_string(input: &str) -> nom::IResult<&str, &str> {
    let single = delimited(
        char('\''),
        alt((escaped(none_of("\\\'"), '\\', char('\'')), tag(""))),
        char('\''),
    );
    let double = delimited(
        char('"'),
        alt((escaped(none_of("\\\""), '\\', char('"')), tag(""))),
        char('"'),
    );
    alt((single, double))(input)
}

fn parse_notation_text(input: &str) -> nom::IResult<&str, &str> {
    let (_, size) = notation_text(input, 1)?;
    let (offset, _) = input
        .char_indices()
        .nth(size - 1)
        .ok_or_else(|| create_err(input, ErrorKind::Eof))?;
    let (x, y) = input.split_at(offset);
    Ok((y, x))
}

fn parse_normal_comment(input: &str) -> nom::IResult<&str, &str> {
    alt((
        map(tuple((many1(char('#')), space0, eof)), |_| ""),
        map(
            tuple((
                many1(char('#')),
                opt(one_of(" \t")),
                not(pair(space0, char('@'))),
            )),
            |_| "",
        ),
    ))(input)
}

fn parse_symbol_data(input: &str) -> nom::IResult<&str, (char, &str, Option<&str>)> {
    map(
        terminated(
            tuple((
                alt((char('@'), char('+'))),
                parse_name,
                opt(delimited(char('['), parse_value_fn, char(']'))),
            )),
            eof,
        ),
        |(symbol, name, choice_fn)| (symbol, name, choice_fn),
    )(input)
}

fn parse_zero_or_one_bind_env(input: &str) -> nom::IResult<&str, Option<Option<String>>> {
    opt(parse_bind_env)(input)
}

fn parse_bind_env(input: &str) -> nom::IResult<&str, Option<String>> {
    map(
        preceded(tag(" $"), alt((tag("$"), parse_bind_env_name))),
        |v| {
            if v == "$" {
                None
            } else {
                Some(v.to_string())
            }
        },
    )(input)
}

fn notation_text(input: &str, balances: usize) -> nom::IResult<&str, usize> {
    let (i1, c1) = anychar(input)?;
    match c1 {
        '<' => {
            let (i2, count) = notation_text(i1, balances + 1)?;
            Ok((i2, count + 1))
        }
        '>' => {
            if balances == 1 {
                Ok((i1, 1))
            } else {
                let (i2, count) = notation_text(i1, balances - 1)?;
                Ok((i2, count + 1))
            }
        }
        _ => notation_text(i1, balances).map(|(i3, v)| (i3, 1 + v)),
    }
}

fn verify_single_char(input: &str) -> nom::IResult<&str, &str> {
    if input
        .chars()
        .take_while(|v| v.is_ascii_alphanumeric())
        .count()
        > 1
    {
        return Err(create_err(input, ErrorKind::Verify));
    }
    Ok((input, ""))
}

fn create_err(input: &str, kind: ErrorKind) -> nom::Err<nom::error::Error<&str>> {
    nom::Err::Error(nom::error::Error::new(input, kind))
}

pub(crate) fn is_choice_value_terminate(c: char) -> bool {
    c == '|' || c == ']'
}

pub(crate) fn is_default_value_terminate(c: char) -> bool {
    c.is_whitespace()
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || is_special_var_char(c)
}

fn is_env_name_char(c: char) -> bool {
    c.is_ascii_uppercase() || c == '_'
}

fn is_short_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '!' | '#'
                | '$'
                | '%'
                | '*'
                | '+'
                | ','
                | '.'
                | '/'
                | ':'
                | '='
                | '?'
                | '@'
                | '['
                | ']'
                | '^'
                | '_'
                | '{'
                | '}'
                | '~'
        )
}

fn take_comment_lines(lines: &[&str], idx: usize, output: &mut String) -> usize {
    let mut count = 0;
    for line in lines.iter().skip(idx) {
        if let Ok((text, _)) = parse_normal_comment(line) {
            output.push('\n');
            let text = match text.starts_with("\\@") {
                true => &text[1..],
                false => text,
            };
            output.push_str(text);
            count += 1;
        } else {
            break;
        }
    }
    *output = output.trim().to_string();
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::param::Param;

    macro_rules! assert_token {
        ($comment:literal, Ignore) => {
            assert_eq!(parse_line($comment).unwrap().1, None)
        };
        ($comment:literal, Error) => {
            assert_eq!(parse_line($comment).unwrap().1.unwrap(), None)
        };
        ($comment:literal, $kind:ident) => {
            assert!(
                if let Some(Some(EventData::$kind(_))) = parse_line($comment).unwrap().1 {
                    true
                } else {
                    false
                }
            );
        };
        ($comment:literal, Aliases, $text:expr) => {
            assert_eq!(
                parse_line($comment).unwrap().1,
                Some(Some(EventData::Aliases(
                    $text.iter().map(|v| v.to_string()).collect()
                )))
            )
        };
        ($comment:literal, $kind:ident, $text:expr) => {
            assert_eq!(
                parse_line($comment).unwrap().1,
                Some(Some(EventData::$kind($text.to_string())))
            )
        };
        ($comment:literal, Meta, $key:expr, $value:expr) => {
            assert_eq!(
                parse_line($comment).unwrap().1,
                Some(Some(EventData::Meta($key.to_string(), $value.to_string())))
            )
        };
    }

    macro_rules! assert_parse_option_arg {
        ($data:literal, $expect:literal) => {
            assert_eq!(
                parse_option_param($data)
                    .unwrap()
                    .1
                    .render_source()
                    .as_str(),
                $expect
            );
        };
        ($data:literal) => {
            assert_eq!(
                parse_option_param($data)
                    .unwrap()
                    .1
                    .render_source()
                    .as_str(),
                $data
            );
        };
    }

    macro_rules! assert_parse_flag_arg {
        ($data:literal, $expect:literal) => {
            assert_eq!(
                parse_flag_arg($data).unwrap().1.render_source().as_str(),
                $expect
            );
        };
        ($data:literal) => {
            assert_eq!(
                parse_flag_param($data).unwrap().1.render_source().as_str(),
                $data
            );
        };
    }

    macro_rules! assert_parse_positional_arg {
        ($data:literal, $expect:literal) => {
            assert_eq!(
                parse_positional_param($data)
                    .unwrap()
                    .1
                    .render_source()
                    .as_str(),
                $expect
            );
        };
        ($data:literal) => {
            assert_eq!(
                parse_positional_param($data)
                    .unwrap()
                    .1
                    .render_source()
                    .as_str(),
                $data
            );
        };
    }

    #[test]
    fn test_parse_with_long_option_arg() {
        assert_parse_option_arg!("-f --foo=a <FOO> A foo option");
        assert_parse_option_arg!("--foo:bar");
        assert_parse_option_arg!("--foo.bar");
        assert_parse_option_arg!("--foo!");
        assert_parse_option_arg!("--foo+");
        assert_parse_option_arg!("--foo*");
        assert_parse_option_arg!("--foo+,");
        assert_parse_option_arg!("--foo*,");
        assert_parse_option_arg!("--foo!");
        assert_parse_option_arg!("--foo-*");
        assert_parse_option_arg!("--foo-");
        assert_parse_option_arg!("--foo--");
        assert_parse_option_arg!("--foo:*");
        assert_parse_option_arg!("--foo:");
        assert_parse_option_arg!("--foo::");
        assert_parse_option_arg!("--foo=a");
        assert_parse_option_arg!("--foo=`_foo`");
        assert_parse_option_arg!("--foo[a|b]");
        assert_parse_option_arg!("--foo[=a|b]");
        assert_parse_option_arg!("--foo[`_foo`]");
        assert_parse_option_arg!("--foo![a|b]");
        assert_parse_option_arg!("--foo![`_foo`]");
        assert_parse_option_arg!("--foo![=a|b]");
        assert_parse_option_arg!("--foo+[a|b]");
        assert_parse_option_arg!("--foo+[`_foo`]");
        assert_parse_option_arg!("--foo+[=a|b]");
        assert_parse_option_arg!("--foo+,[a|b]");
        assert_parse_option_arg!("--foo*[a|b]");
        assert_parse_option_arg!("--foo*[=a|b]");
        assert_parse_option_arg!("--foo*[`_foo`]");
        assert_parse_option_arg!("--foo*,[a|b]");
        assert_parse_option_arg!("--foo <FOO>");
        assert_parse_option_arg!("--foo-abc <FOO>");
        assert_parse_option_arg!("--foo=\"a b\"");
        assert_parse_option_arg!("--foo[\"a|b\"|\"c]d\"]");
        assert_parse_option_arg!("--foo <abc>");
        assert_parse_option_arg!("--foo <abc> <def>");
        assert_parse_option_arg!("--foo <>");
        assert_parse_option_arg!("--foo <abc def>");
        assert_parse_option_arg!("--foo <<abc def>>");
        assert_parse_option_arg!("--foo $$");
        assert_parse_option_arg!("--foo $FOO");
        assert_parse_option_arg!("--foo $FOO <FOO>");
    }

    #[test]
    fn test_parse_with_long_option_arg_single_dash() {
        assert_parse_option_arg!("-f -foo=a <FOO> A foo option");
        assert_parse_option_arg!("-foo:bar");
        assert_parse_option_arg!("-foo.bar");
        assert_parse_option_arg!("-foo!");
        assert_parse_option_arg!("-foo+");
        assert_parse_option_arg!("-foo*");
        assert_parse_option_arg!("-foo+,");
        assert_parse_option_arg!("-foo*,");
        assert_parse_option_arg!("-foo!");
        assert_parse_option_arg!("-foo-*");
        assert_parse_option_arg!("-foo-");
        assert_parse_option_arg!("-foo--");
        assert_parse_option_arg!("-foo:*");
        assert_parse_option_arg!("-foo:");
        assert_parse_option_arg!("-foo::");
        assert_parse_option_arg!("-foo=a");
        assert_parse_option_arg!("-foo=`_foo`");
        assert_parse_option_arg!("-foo[a|b]");
        assert_parse_option_arg!("-foo[=a|b]");
        assert_parse_option_arg!("-foo[`_foo`]");
        assert_parse_option_arg!("-foo![a|b]");
        assert_parse_option_arg!("-foo![`_foo`]");
        assert_parse_option_arg!("-foo![=a|b]");
        assert_parse_option_arg!("-foo+[a|b]");
        assert_parse_option_arg!("-foo+[`_foo`]");
        assert_parse_option_arg!("-foo+[=a|b]");
        assert_parse_option_arg!("-foo*[a|b]");
        assert_parse_option_arg!("-foo*[=a|b]");
        assert_parse_option_arg!("-foo*[`_foo`]");
        assert_parse_option_arg!("-foo <FOO>");
        assert_parse_option_arg!("-foo-abc <FOO>");
        assert_parse_option_arg!("-foo=\"a b\"");
        assert_parse_option_arg!("-foo[\"a|b\"|\"c]d\"]");
        assert_parse_option_arg!("-foo <abc>");
        assert_parse_option_arg!("-foo <abc> <def>");
        assert_parse_option_arg!("-foo <>");
        assert_parse_option_arg!("-foo <abc def>");
        assert_parse_option_arg!("-foo <<abc def>>");
        assert_parse_option_arg!("-foo $$");
        assert_parse_option_arg!("-foo $FOO");
        assert_parse_option_arg!("-foo $FOO <FOO>");
    }

    #[test]
    fn test_parse_no_long_option_arg() {
        assert_parse_option_arg!("-f");
        assert_parse_option_arg!("-f!");
        assert_parse_option_arg!("-f=a");
        assert_parse_option_arg!("-f=`_foo`");
        assert_parse_option_arg!("-f[a|b]");
        assert_parse_option_arg!("-f[=a|b]");
        assert_parse_option_arg!("-f[`_foo`]");
        assert_parse_option_arg!("-f![a|b]");
        assert_parse_option_arg!("-f![`_foo`]");
        assert_parse_option_arg!("-f![=a|b]");
        assert_parse_option_arg!("-D-");
        assert_parse_option_arg!("-D--");
    }

    #[test]
    fn test_parse_with_long_flag_arg() {
        assert_parse_flag_arg!("-f --foo A foo flag");
        assert_parse_flag_arg!("-. --hidden");
        assert_parse_flag_arg!("--http1.1");
        assert_parse_flag_arg!("--foo A foo flag");
        assert_parse_flag_arg!("--foo");
        assert_parse_flag_arg!("--foo*");
    }

    #[test]
    fn test_parse_with_long_flag_arg_single_dash() {
        assert_parse_flag_arg!("-f -foo A foo flag");
        assert_parse_flag_arg!("-. -hidden");
        assert_parse_flag_arg!("-http1.1");
        assert_parse_flag_arg!("-foo A foo flag");
        assert_parse_flag_arg!("-foo");
        assert_parse_flag_arg!("-foo*");
    }

    #[test]
    fn test_parse_no_long_flag_arg() {
        assert_parse_flag_arg!("-f A foo flag");
        assert_parse_flag_arg!("-f");
        assert_parse_flag_arg!("-.");
        assert_parse_flag_arg!("-0");
        assert_parse_flag_arg!("-#");
        assert_parse_flag_arg!("-:");
        assert_parse_flag_arg!("-f*");
    }

    #[test]
    fn test_parse_with_long_head() {
        assert_eq!(
            parse_with_long_head("-f --foo"),
            Ok(("foo", (Some("-f"), "--")))
        );
        assert_eq!(
            parse_with_long_head("-f -foo"),
            Ok(("foo", (Some("-f"), "-")))
        );
        assert_eq!(parse_with_long_head("--foo"), Ok(("foo", (None, "--"))));
        assert_eq!(parse_with_long_head("-foo"), Ok(("foo", (None, "-"))));
        assert_eq!(
            parse_with_long_head("+f +foo"),
            Ok(("foo", (Some("+f"), "+")))
        );
        assert_eq!(parse_with_long_head("+foo"), Ok(("foo", (None, "+"))));
        assert!(parse_with_long_head("-f ---foo").is_err());
        assert!(parse_with_long_head("+f ++foo").is_err());
    }

    #[test]
    fn test_parse_positional_arg() {
        assert_parse_positional_arg!("foo <FOO> A foo arg");
        assert_parse_positional_arg!("a.b");
        assert_parse_positional_arg!("foo");
        assert_parse_positional_arg!("foo!");
        assert_parse_positional_arg!("foo+");
        assert_parse_positional_arg!("foo*");
        assert_parse_positional_arg!("foo <FOO>");
        assert_parse_positional_arg!("foo=a");
        assert_parse_positional_arg!("foo=`_foo`");
        assert_parse_positional_arg!("foo[a|b]");
        assert_parse_positional_arg!("foo[`_foo`]");
        assert_parse_positional_arg!("foo[=a|b]");
        assert_parse_positional_arg!("foo![a|b]");
        assert_parse_positional_arg!("foo![`_foo`]");
        assert_parse_positional_arg!("foo![=a|b]");
        assert_parse_positional_arg!("foo+[a|b]");
        assert_parse_positional_arg!("foo+[`_foo`]");
        assert_parse_positional_arg!("foo+[=a|b]");
        assert_parse_positional_arg!("foo*[a|b]");
        assert_parse_positional_arg!("foo*[`_foo`]");
        assert_parse_positional_arg!("foo*[=a|b]");
        assert_parse_positional_arg!("foo $$");
        assert_parse_positional_arg!("foo $FOO");
    }

    #[test]
    fn test_special_notation() {
        assert_parse_option_arg!("--foo <◉>");
    }

    #[test]
    fn test_parse_line() {
        assert_token!("# @describe A demo cli", Describe, "A demo cli");
        assert_token!("# @version 1.0.0", Version, "1.0.0");
        assert_token!("# @author Somebody", Author, "Somebody");
        assert_token!("# @meta key", Meta, "key", "");
        assert_token!("# @meta key value", Meta, "key", "value");
        assert_token!("# @cmd A subcommand", Cmd, "A subcommand");
        assert_token!("# @alias tst", Aliases, ["tst"]);
        assert_token!("# @alias t,tst", Aliases, ["t", "tst"]);
        assert_token!("# @flag -f --foo", FlagOption);
        assert_token!("# @option -f --foo", FlagOption);
        assert_token!("# @arg foo", Positional);
        assert_token!("foo()", Func, "foo");
        assert_token!("foo ()", Func, "foo");
        assert_token!("foo  ()", Func, "foo");
        assert_token!("foo ( )", Func, "foo");
        assert_token!(" foo ()", Ignore);
        assert_token!("foo_bar ()", Func, "foo_bar");
        assert_token!("foo-bar ()", Func, "foo-bar");
        assert_token!("foo:bar ()", Func, "foo:bar");
        assert_token!("foo.bar ()", Func, "foo.bar");
        assert_token!("foo@bar ()", Func, "foo@bar");
        assert_token!("function foo", Func, "foo");
        assert_token!("function  foo", Func, "foo");
        assert_token!(" function foo", Ignore);
        assert_token!("function foo_bar", Func, "foo_bar");
        assert_token!("function foo-bar", Func, "foo-bar");
        assert_token!("function foo:bar", Func, "foo:bar");
        assert_token!("function foo.bar", Func, "foo.bar");
        assert_token!("function foo@bar", Func, "foo@bar");
        assert_token!("#!/bin/bash", Ignore);
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(
            parse_symbol("+toolchain").unwrap(),
            ('+', "toolchain", None)
        );
        assert_eq!(
            parse_symbol("+toolchain[`_choice_toolchain`]").unwrap(),
            ('+', "toolchain", Some("_choice_toolchain"))
        );
    }
}
