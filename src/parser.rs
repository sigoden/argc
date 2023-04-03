use crate::param::{FlagParam, OptionParam, ParamData, PositionalParam};
use crate::utils::{is_choice_value_terminate, is_default_value_terminate};
use crate::Result;
use anyhow::bail;
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_till, take_while1},
    character::{
        complete::{anychar, char, satisfy, space0, space1},
        streaming::none_of,
    },
    combinator::{eof, fail, map, opt, peek, rest, success},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Event {
    pub data: EventData,
    pub position: Position,
}

pub type Position = usize;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EventData {
    /// Description
    Describe(String),
    /// Version info
    Version(String),
    /// Author info
    Author(String),
    /// Define a subcommand, e.g. `@cmd A sub command`
    Cmd(String),
    /// Define alias for a subcommand, e.g. `@alias t,tst`
    Aliases(Vec<String>),
    /// Define a option parameter
    Option(OptionParam),
    /// Define a positional parameter
    Positional(PositionalParam),
    /// Define a flag
    Flag(FlagParam),
    /// A shell function. e.g `function cmd()` or `cmd()`
    Func(String),
    /// Placeholder for unknown or invalid tag
    Unknown(String),
}

#[derive(PartialEq, Eq)]
pub enum EventScope {
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
pub fn parse(source: &str) -> Result<Vec<Event>> {
    let mut result = vec![];
    for (line_idx, line) in source.lines().enumerate() {
        let position = line_idx + 1;
        match parse_line(line) {
            Ok((_, maybe_token)) => {
                if let Some(maybe_data) = maybe_token {
                    if let Some(data) = maybe_data {
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
    }
    Ok(result)
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
    preceded(tuple((space0, tag("function"), space1)), parse_fn_name)(input)
}

// Parse fn likes `foo ()`
fn parse_fn_no_keyword(input: &str) -> nom::IResult<&str, &str> {
    preceded(
        space0,
        terminated(parse_fn_name, tuple((space0, char('('), space0, char(')')))),
    )(input)
}

fn parse_tag(input: &str) -> nom::IResult<&str, Option<EventData>> {
    preceded(
        tuple((many1(char('#')), space0, char('@'))),
        alt((
            parse_tag_text,
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

fn parse_tag_param(input: &str) -> nom::IResult<&str, Option<EventData>> {
    let check = peek(alt((tag("option"), tag("flag"), tag("arg"))));
    let arg = alt((
        map(
            preceded(pair(tag("flag"), space1), parse_flag_param),
            |param| Some(EventData::Flag(param)),
        ),
        map(
            preceded(pair(tag("option"), space1), parse_option_param),
            |param| Some(EventData::Option(param)),
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
fn parse_option_param(input: &str) -> nom::IResult<&str, OptionParam> {
    alt((parse_with_long_option_param, parse_no_long_option_param))(input)
}

// Parse `@option` with long name
fn parse_with_long_option_param(input: &str) -> nom::IResult<&str, OptionParam> {
    map(
        tuple((
            parse_short,
            preceded(
                pair(space0, tag("--")),
                alt((
                    parse_param_modifer_choices_default,
                    parse_param_modifer_choices_fn,
                    parse_param_modifer_choices,
                    parse_param_assign_fn,
                    parse_param_assign,
                    parse_param_modifer,
                )),
            ),
            parse_zero_or_many_value_notations,
            parse_tail,
        )),
        |(short, arg, value_names, summary)| {
            OptionParam::new(arg, summary, short, false, &value_names)
        },
    )(input)
}

// Parse `@option` without long name
fn parse_no_long_option_param(input: &str) -> nom::IResult<&str, OptionParam> {
    map(
        tuple((
            preceded(
                pair(space0, tag("-")),
                preceded(
                    verify_single_char,
                    alt((
                        parse_param_modifer_choices_default,
                        parse_param_modifer_choices_fn,
                        parse_param_modifer_choices,
                        parse_param_assign_fn,
                        parse_param_assign,
                        parse_param_modifer,
                    )),
                ),
            ),
            parse_zero_or_many_value_notations,
            parse_tail,
        )),
        |(arg, value_names, summary)| {
            let short = arg.name.chars().next();
            OptionParam::new(arg, summary, short, true, &value_names)
        },
    )(input)
}

// Parse `@option`, positional only
fn parse_positional_param(input: &str) -> nom::IResult<&str, PositionalParam> {
    map(
        tuple((
            alt((
                parse_param_modifer_choices_default,
                parse_param_modifer_choices_fn,
                parse_param_modifer_choices,
                parse_param_assign_fn,
                parse_param_assign,
                parse_param_modifer,
            )),
            parse_zero_or_one_value_notation,
            parse_tail,
        )),
        |(arg, value_name, summary)| PositionalParam::new(arg, summary, value_name),
    )(input)
}

// Parse `@flag`
fn parse_flag_param(input: &str) -> nom::IResult<&str, FlagParam> {
    alt((parse_with_long_flag_param, parse_no_long_flag_param))(input)
}
// Parse `@flag`
fn parse_with_long_flag_param(input: &str) -> nom::IResult<&str, FlagParam> {
    map(
        tuple((
            parse_short,
            preceded(pair(space0, tag("--")), parse_long_flag_and_asterisk),
            parse_tail,
        )),
        |(short, arg, summary)| FlagParam::new(arg, summary, short, false),
    )(input)
}

// Parse `@flag` without long name
fn parse_no_long_flag_param(input: &str) -> nom::IResult<&str, FlagParam> {
    map(
        tuple((
            preceded(pair(space0, tag("-")), parse_short_flag_and_asterisk),
            parse_tail,
        )),
        |(arg, summary)| {
            let short = arg.name.chars().next();
            FlagParam::new(arg, summary, short, true)
        },
    )(input)
}

// Parse `str*` `str`
fn parse_long_flag_and_asterisk(input: &str) -> nom::IResult<&str, ParamData> {
    alt((
        map(terminated(parse_param_name, tag("*")), |mut arg| {
            arg.multiple = true;
            arg
        }),
        parse_param_name,
    ))(input)
}

// Parse ':' or '#' or '0'
fn parse_short_flag_and_asterisk(input: &str) -> nom::IResult<&str, ParamData> {
    fn parser(input: &str) -> nom::IResult<&str, ParamData> {
        map(satisfy(is_short_char), |ch| {
            ParamData::new(&format!("{}", ch))
        })(input)
    }
    map(pair(parser, opt(tag("*"))), |(mut arg, multiple)| {
        arg.multiple = multiple.is_some();
        arg
    })(input)
}

// Parse `str!` `str*` `str+` `str`
fn parse_param_modifer(input: &str) -> nom::IResult<&str, ParamData> {
    alt((
        map(terminated(parse_param_name, tag("!")), |mut arg| {
            arg.required = true;
            arg
        }),
        map(terminated(parse_param_name, tag("*")), |mut arg| {
            arg.multiple = true;
            arg
        }),
        map(terminated(parse_param_name, tag("+")), |mut arg| {
            arg.required = true;
            arg.multiple = true;
            arg
        }),
        parse_param_name,
    ))(input)
}

// Parse `str=value`
fn parse_param_assign(input: &str) -> nom::IResult<&str, ParamData> {
    map(
        separated_pair(parse_param_name, char('='), parse_default_value),
        |(mut arg, value)| {
            arg.default = Some(value.to_string());
            arg
        },
    )(input)
}

// Parse str=`value`
fn parse_param_assign_fn(input: &str) -> nom::IResult<&str, ParamData> {
    map(
        separated_pair(parse_param_name, char('='), parse_value_fn),
        |(mut arg, value)| {
            arg.default_fn = Some(value.to_string());
            arg
        },
    )(input)
}

fn parse_param_modifer_choices_default(input: &str) -> nom::IResult<&str, ParamData> {
    map(
        pair(
            parse_param_modifer,
            delimited(char('['), parse_choices_default, char(']')),
        ),
        |(mut arg, (choices, default))| {
            arg.choices = Some(choices.iter().map(|v| v.to_string()).collect());
            arg.required = false;
            arg.default = default.map(|v| v.to_string());
            arg
        },
    )(input)
}

fn parse_param_modifer_choices(input: &str) -> nom::IResult<&str, ParamData> {
    map(
        pair(
            parse_param_modifer,
            delimited(char('['), parse_choices, char(']')),
        ),
        |(mut arg, choices)| {
            arg.choices = Some(choices.iter().map(|v| v.to_string()).collect());
            arg
        },
    )(input)
}

fn parse_param_modifer_choices_fn(input: &str) -> nom::IResult<&str, ParamData> {
    map(
        pair(
            parse_param_modifer,
            delimited(char('['), parse_value_fn, char(']')),
        ),
        |(mut arg, choices_fn)| {
            arg.choices_fn = Some(choices_fn.into());
            arg
        },
    )(input)
}

// Parse `str`
fn parse_param_name(input: &str) -> nom::IResult<&str, ParamData> {
    map(parse_name, ParamData::new)(input)
}

// Parse `-s`
fn parse_short(input: &str) -> nom::IResult<&str, Option<char>> {
    let short = delimited(char('-'), satisfy(is_short_char), peek(space1));
    opt(short)(input)
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
    preceded(space0, delimited(char('<'), parse_notation_text, char('>')))(input)
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

fn parse_name_list(input: &str) -> nom::IResult<&str, Vec<&str>> {
    separated_list1(char(','), delimited(space0, parse_name, space0))(input)
}

fn parse_fn_name(input: &str) -> nom::IResult<&str, &str> {
    take_while1(is_not_fn_name_char)(input)
}

fn parse_name(input: &str) -> nom::IResult<&str, &str> {
    take_while1(is_name_char)(input)
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
    Ok((&input[size - 1..], &input[0..size - 1]))
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
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }
    Ok((input, ""))
}

fn is_not_fn_name_char(c: char) -> bool {
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
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
}

fn is_short_char(c: char) -> bool {
    c.is_ascii() && is_not_fn_name_char(c) && !matches!(c, '-')
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    macro_rules! assert_parse_option_arg {
        ($data:literal, $expect:literal) => {
            assert_eq!(
                parse_option_param($data).unwrap().1.render().as_str(),
                $expect
            );
        };
        ($data:literal) => {
            assert_eq!(
                parse_option_param($data).unwrap().1.render().as_str(),
                $data
            );
        };
    }

    macro_rules! assert_parse_flag_arg {
        ($data:literal, $expect:literal) => {
            assert_eq!(parse_flag_arg($data).unwrap().1.render().as_str(), $expect);
        };
        ($data:literal) => {
            assert_eq!(parse_flag_param($data).unwrap().1.render().as_str(), $data);
        };
    }

    macro_rules! assert_parse_positional_arg {
        ($data:literal, $expect:literal) => {
            assert_eq!(
                parse_positional_param($data).unwrap().1.render().as_str(),
                $expect
            );
        };
        ($data:literal) => {
            assert_eq!(
                parse_positional_param($data).unwrap().1.render().as_str(),
                $data
            );
        };
    }

    #[test]
    fn test_parse_with_long_option_arg() {
        assert_parse_option_arg!("-f --foo=a <FOO> A foo option");
        assert_parse_option_arg!("--foo!");
        assert_parse_option_arg!("--foo+");
        assert_parse_option_arg!("--foo*");
        assert_parse_option_arg!("--foo!");
        assert_parse_option_arg!("--foo=a");
        assert_parse_option_arg!("--foo=`_foo`");
        assert_parse_option_arg!("--foo[a|b]");
        assert_parse_option_arg!("--foo[=a|b]");
        assert_parse_option_arg!("--foo[`_foo`]");
        assert_parse_option_arg!("--foo![a|b]");
        assert_parse_option_arg!("--foo![`_foo`]");
        assert_parse_option_arg!("--foo![=a|b]", "--foo[=a|b]");
        assert_parse_option_arg!("--foo+[a|b]");
        assert_parse_option_arg!("--foo+[`_foo`]");
        assert_parse_option_arg!("--foo+[=a|b]", "--foo*[=a|b]");
        assert_parse_option_arg!("--foo*[a|b]");
        assert_parse_option_arg!("--foo*[=a|b]");
        assert_parse_option_arg!("--foo*[`_foo`]");
        assert_parse_option_arg!("--foo <FOO>");
        assert_parse_option_arg!("--foo-abc <FOO>");
        assert_parse_option_arg!("--foo=\"a b\"");
        assert_parse_option_arg!("--foo[\"a|b\"|\"c]d\"]");
        assert_parse_option_arg!("--foo <abc>");
        assert_parse_option_arg!("--foo <abc> <def>");
        assert_parse_option_arg!("--foo <>");
        assert_parse_option_arg!("--foo <abc def>");
        assert_parse_option_arg!("--foo <<abc def>>");
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
        assert_parse_option_arg!("-f![=a|b]", "-f[=a|b]");
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
        assert_parse_positional_arg!("foo![=a|b]", "foo[=a|b]");
        assert_parse_positional_arg!("foo+[a|b]");
        assert_parse_positional_arg!("foo+[`_foo`]");
        assert_parse_positional_arg!("foo+[=a|b]", "foo*[=a|b]");
        assert_parse_positional_arg!("foo*[a|b]");
        assert_parse_positional_arg!("foo*[`_foo`]");
        assert_parse_positional_arg!("foo*[=a|b]");
    }

    #[test]
    fn test_parse_line() {
        assert_token!("# @describe A demo cli", Describe, "A demo cli");
        assert_token!("# @version 1.0.0", Version, "1.0.0");
        assert_token!("# @author Somebody", Author, "Somebody");
        assert_token!("# @cmd A subcommand", Cmd, "A subcommand");
        assert_token!("# @alias tst", Aliases, vec!["tst"]);
        assert_token!("# @alias t,tst", Aliases, vec!["t", "tst"]);
        assert_token!("# @flag -f --foo", Flag);
        assert_token!("# @option -f --foo", Option);
        assert_token!("# @arg foo", Positional);
        assert_token!("foo()", Func, "foo");
        assert_token!("foo ()", Func, "foo");
        assert_token!("foo  ()", Func, "foo");
        assert_token!("foo ( )", Func, "foo");
        assert_token!(" foo ()", Func, "foo");
        assert_token!("foo_bar ()", Func, "foo_bar");
        assert_token!("foo-bar ()", Func, "foo-bar");
        assert_token!("foo:bar ()", Func, "foo:bar");
        assert_token!("foo.bar ()", Func, "foo.bar");
        assert_token!("foo@bar ()", Func, "foo@bar");
        assert_token!("function foo", Func, "foo");
        assert_token!("function  foo", Func, "foo");
        assert_token!(" function foo", Func, "foo");
        assert_token!("function foo_bar", Func, "foo_bar");
        assert_token!("function foo-bar", Func, "foo-bar");
        assert_token!("function foo:bar", Func, "foo:bar");
        assert_token!("function foo.bar", Func, "foo.bar");
        assert_token!("function foo@bar", Func, "foo@bar");
        assert_token!("foo=bar", Ignore);
        assert_token!("#!/bin/bash", Ignore);
        assert_token!("# @flag -foo", Error);
        assert_token!("# @option -foo![=a|b]", Error);
    }
}
