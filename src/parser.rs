use crate::Result;
use anyhow::bail;
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_till, take_while},
    character::{
        complete::{char, satisfy, space0, space1},
        streaming::none_of,
    },
    combinator::{eof, map, opt, peek, rest, success},
    multi::many1,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub struct Event<'a> {
    pub data: EventData<'a>,
    pub position: Position,
}

pub type Position = usize;

#[derive(Debug, PartialEq, Clone)]
pub enum EventData<'a> {
    /// Description
    Describe(&'a str),
    /// Version info
    Version(&'a str),
    /// Author info
    Author(&'a str),
    /// Define a subcommand, e.g. `@cmd A sub command`
    Cmd(&'a str),
    /// Define a arguement
    Arg(ArgData<'a>),
    /// A shell function. e.g `function cmd()` or `cmd()`
    Func(&'a str),
    /// Palaceholder for unknown or invalid tag
    Unknown(&'a str),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArgData<'a> {
    pub name: &'a str,
    pub kind: ArgKind,
    pub summary: Option<&'a str>,
    pub value_name: Option<&'a str>,
    pub short: Option<char>,
    pub choices: Option<Vec<&'a str>>,
    pub multiple: bool,
    pub required: bool,
    pub default: Option<&'a str>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArgKind {
    Flag,
    Option,
    Positional,
}

impl Display for ArgKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ArgKind::Flag => "@flag",
                ArgKind::Option => "@option",
                ArgKind::Positional => "@arg",
            }
        )
    }
}

impl<'a> Display for ArgData<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut segments: Vec<String> = vec![];
        match self.kind {
            ArgKind::Flag => {
                if let Some(s) = self.short {
                    segments.push(format!("-{}", s));
                }
                segments.push(format!("--{}", self.name));
            }
            ArgKind::Option => {
                if let Some(s) = self.short {
                    segments.push(format!("-{}", s));
                }
                let mut name = self.name.to_string();
                if let Some(choices) = &self.choices {
                    let mut prefix = String::new();
                    if self.default.is_some() {
                        prefix.push('=');
                    }
                    let values: Vec<String> = choices
                        .iter()
                        .map(|value| {
                            if value.chars().any(is_terminate_char_choice_value) {
                                format!("\"{}\"", value)
                            } else {
                                value.to_string()
                            }
                        })
                        .collect();
                    name.push_str(&format!("[{}{}]", prefix, values.join("|")))
                } else {
                    if let Some(default) = self.default {
                        let value = if default.chars().any(is_terminate_char_default_value) {
                            format!("\"{}\"", default)
                        } else {
                            default.to_string()
                        };
                        name.push_str(&format!("={}", value));
                    } else if let Some(c) = self.name_suffix() {
                        name.push(c)
                    }
                }
                segments.push(format!("--{}", name));
                if let Some(value_name) = self.value_name {
                    segments.push(format!("<{}>", value_name));
                }
            }
            ArgKind::Positional => {
                let mut name = self.name.to_string();
                if let Some(c) = self.name_suffix() {
                    name.push(c)
                }
                segments.push(name);
            }
        }
        if let Some(summary) = self.summary {
            segments.push(summary.to_string());
        }
        write!(f, "{}", segments.join(" "))
    }
}

impl<'a> ArgData<'a> {
    pub fn new(name: &'a str) -> Self {
        ArgData {
            name,
            summary: None,
            kind: ArgKind::Option,
            value_name: None,
            short: None,
            choices: None,
            multiple: false,
            required: false,
            default: None,
        }
    }
    pub fn is_positional(&self) -> bool {
        self.kind == ArgKind::Positional
    }
    fn name_suffix(&self) -> Option<char> {
        if self.multiple {
            return Some(match self.required {
                true => '+',
                false => '*',
            });
        }
        if self.required {
            return Some('!');
        }
        None
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
    alt((map(alt((parse_tag, parse_fn)), |v| Some(v)), success(None)))(line)
}

fn parse_fn(input: &str) -> nom::IResult<&str, Option<EventData>> {
    map(alt((parse_fn_keyword, parse_fn_no_keyword)), |v| {
        Some(EventData::Func(v))
    })(input)
}

// Parse fn likes `function foo`
fn parse_fn_keyword(input: &str) -> nom::IResult<&str, &str> {
    preceded(tuple((space0, tag("function"), space1)), parse_name)(input)
}

// Parse fn likes `foo ()`
fn parse_fn_no_keyword(input: &str) -> nom::IResult<&str, &str> {
    preceded(
        space0,
        terminated(parse_name, tuple((space0, char('('), space0, char(')')))),
    )(input)
}

fn parse_tag(input: &str) -> nom::IResult<&str, Option<EventData>> {
    preceded(
        tuple((many1(char('#')), space0, char('@'))),
        alt((parse_tag_text, parse_tag_arg, parse_tag_unknown)),
    )(input)
}

fn parse_tag_text(input: &str) -> nom::IResult<&str, Option<EventData>> {
    map(
        pair(
            alt((tag("describe"), tag("version"), tag("author"), tag("cmd"))),
            parse_tail,
        ),
        |(tag, text)| {
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

fn parse_tag_arg(input: &str) -> nom::IResult<&str, Option<EventData>> {
    let check = peek(alt((tag("option"), tag("flag"), tag("arg"))));
    let arg = map(
        alt((
            tuple((tag("option"), space1, parse_option_arg)),
            tuple((tag("flag"), space1, parse_flag_arg)),
            tuple((tag("arg"), space1, parse_positional_arg)),
        )),
        |(_, _, data)| Some(EventData::Arg(data)),
    );
    preceded(check, alt((arg, success(None))))(input)
}

fn parse_tag_unknown(input: &str) -> nom::IResult<&str, Option<EventData>> {
    map(parse_name, |v| Some(EventData::Unknown(v)))(input)
}

// Parse `@option`
fn parse_option_arg(input: &str) -> nom::IResult<&str, ArgData> {
    map(
        tuple((
            parse_short,
            preceded(
                pair(space0, tag("--")),
                alt((parse_arg_choices, parse_arg_assign, parse_arg_mark)),
            ),
            parse_value_notation,
            parse_tail,
        )),
        |(short, mut arg, value_name, summary)| {
            arg.short = short;
            if summary.len() > 0 {
                arg.summary = Some(summary);
            }
            arg.value_name = value_name;
            arg
        },
    )(input)
}

// Parse `@option`, positional only
fn parse_positional_arg(input: &str) -> nom::IResult<&str, ArgData> {
    map(pair(parse_arg_mark, parse_tail), |(mut arg, summary)| {
        arg.kind = ArgKind::Positional;
        if summary.len() > 0 {
            arg.summary = Some(summary);
        }
        arg
    })(input)
}

// Parse `@flag`
fn parse_flag_arg(input: &str) -> nom::IResult<&str, ArgData> {
    map(
        tuple((
            parse_short,
            preceded(pair(space0, tag("--")), parse_arg_name),
            parse_tail,
        )),
        |(short, mut arg, summary)| {
            arg.short = short;
            if summary.len() > 0 {
                arg.summary = Some(summary);
            }
            arg.kind = ArgKind::Flag;
            arg
        },
    )(input)
}

// Parse `str!` `str*` `str+` `str`
fn parse_arg_mark(input: &str) -> nom::IResult<&str, ArgData> {
    alt((
        map(terminated(parse_arg_name, tag("!")), |mut arg| {
            arg.required = true;
            arg
        }),
        map(terminated(parse_arg_name, tag("*")), |mut arg| {
            arg.multiple = true;
            arg
        }),
        map(terminated(parse_arg_name, tag("+")), |mut arg| {
            arg.required = true;
            arg.multiple = true;
            arg
        }),
        parse_arg_name,
    ))(input)
}

// Parse `str=value`
fn parse_arg_assign(input: &str) -> nom::IResult<&str, ArgData> {
    map(
        separated_pair(parse_arg_name, char('='), parse_default_value),
        |(mut arg, value)| {
            arg.default = Some(value);
            arg
        },
    )(input)
}

// Parse `str[a|b|c]` or `str[=a|b|c]`
fn parse_arg_choices(input: &str) -> nom::IResult<&str, ArgData> {
    map(
        pair(
            parse_arg_name,
            delimited(char('['), parse_choices, char(']')),
        ),
        |(mut arg, (choices, default))| {
            arg.choices = Some(choices);
            arg.default = default;
            arg
        },
    )(input)
}

// Parse `str`
fn parse_arg_name(input: &str) -> nom::IResult<&str, ArgData> {
    map(parse_name, |v| ArgData::new(v))(input)
}

// Parse `-s`
fn parse_short(input: &str) -> nom::IResult<&str, Option<char>> {
    let short = delimited(
        char('-'),
        satisfy(|c| c.is_ascii_alphabetic()),
        peek(space1),
    );
    opt(short)(input)
}

// Parse '<FOO>'
fn parse_value_notation(input: &str) -> nom::IResult<&str, Option<&str>> {
    let main = delimited(
        char('<'),
        take_while(|c: char| c.is_ascii_uppercase() || c == '-'),
        char('>'),
    );
    opt(preceded(space0, main))(input)
}

// Parse `a|b|c`, `=a|b|c`
fn parse_choices(input: &str) -> nom::IResult<&str, (Vec<&str>, Option<&str>)> {
    map(
        tuple((
            opt(char('=')),
            parse_choice_value,
            many1(preceded(char('|'), parse_choice_value)),
        )),
        |(default, head, tail)| {
            let mut choices = vec![head];
            choices.extend(tail);
            let default_choice = default.map(|_| head);
            (choices, default_choice)
        },
    )(input)
}

fn parse_tail(input: &str) -> nom::IResult<&str, &str> {
    alt((
        eof,
        preceded(space1, alt((eof, map(rest, |v: &str| v.trim())))),
    ))(input)
}

fn parse_name(input: &str) -> nom::IResult<&str, &str> {
    take_while(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-')(input)
}

fn parse_default_value(input: &str) -> nom::IResult<&str, &str> {
    alt((
        parse_quoted_string,
        take_till(is_terminate_char_default_value),
    ))(input)
}

fn is_terminate_char_default_value(c: char) -> bool {
    c.is_whitespace()
}

fn parse_choice_value(input: &str) -> nom::IResult<&str, &str> {
    alt((
        parse_quoted_string,
        take_till(is_terminate_char_choice_value),
    ))(input)
}

fn is_terminate_char_choice_value(c: char) -> bool {
    return c == '|' || c == ']';
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
        ($comment:literal, $kind:ident, $text:literal) => {
            assert_eq!(
                parse_line($comment).unwrap().1,
                Some(Some(EventData::$kind($text)))
            )
        };
    }

    macro_rules! assert_parse_option_arg {
        ($data:literal, &expect:literal) => {
            assert_eq!(
                parse_option_arg($data).unwrap().1.to_string().as_str(),
                $expect
            );
        };
        ($data:literal) => {
            assert_eq!(
                parse_option_arg($data).unwrap().1.to_string().as_str(),
                $data
            );
        };
    }

    macro_rules! assert_parse_flag_arg {
        ($data:literal, &expect:literal) => {
            assert_eq!(
                parse_flag_arg($data).unwrap().1.to_string().as_str(),
                $expect
            );
        };
        ($data:literal) => {
            assert_eq!(parse_flag_arg($data).unwrap().1.to_string().as_str(), $data);
        };
    }

    macro_rules! assert_parse_positional_arg {
        ($data:literal, &expect:literal) => {
            assert_eq!(
                parse_positional_arg($data).unwrap().1.to_string().as_str(),
                $expect
            );
        };
        ($data:literal) => {
            assert_eq!(
                parse_positional_arg($data).unwrap().1.to_string().as_str(),
                $data
            );
        };
    }

    #[test]
    fn test_parse_option_arg() {
        assert_parse_option_arg!("-f --foo=a <FOO> A foo option");
        assert_parse_option_arg!("--foo!");
        assert_parse_option_arg!("--foo+");
        assert_parse_option_arg!("--foo*");
        assert_parse_option_arg!("--foo!");
        assert_parse_option_arg!("--foo=a");
        assert_parse_option_arg!("--foo[a|b]");
        assert_parse_option_arg!("--foo[=a|b]");
        assert_parse_option_arg!("--foo <FOO>");
        assert_parse_option_arg!("--foo-abc <FOO>");
        assert_parse_option_arg!("--foo=\"a b\"");
        assert_parse_option_arg!("--foo[\"a|b\"|\"c]d\"]");
    }

    #[test]
    fn test_parse_flag_arg() {
        assert_parse_flag_arg!("-f --foo A foo flag");
        assert_parse_flag_arg!("--foo A foo flag");
        assert_parse_flag_arg!("--foo");
    }

    #[test]
    fn test_parse_positional_arg() {
        assert_parse_positional_arg!("foo A foo arg");
        assert_parse_positional_arg!("foo");
        assert_parse_positional_arg!("foo!");
        assert_parse_positional_arg!("foo+");
        assert_parse_positional_arg!("foo*");
    }

    #[test]
    fn test_parse_line() {
        assert_token!("# @describe A demo cli", Describe, "A demo cli");
        assert_token!("# @version 1.0.0", Version, "1.0.0");
        assert_token!("# @author Somebody", Author, "Somebody");
        assert_token!("# @cmd A subcommand", Cmd, "A subcommand");
        assert_token!("# @flag -f --foo", Arg);
        assert_token!("# @option -f --foo", Arg);
        assert_token!("# @arg foo", Arg);
        assert_token!("foo()", Func, "foo");
        assert_token!("foo ()", Func, "foo");
        assert_token!("foo  ()", Func, "foo");
        assert_token!("foo ( )", Func, "foo");
        assert_token!(" foo ()", Func, "foo");
        assert_token!("function foo", Func, "foo");
        assert_token!("function  foo", Func, "foo");
        assert_token!(" function foo", Func, "foo");
        assert_token!("foo=bar", Ignore);
        assert_token!("#!/bin/bash", Ignore);
        assert_token!("# @flag -f", Error);
    }
}
