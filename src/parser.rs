use crate::{bail, ArgData, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alphanumeric1, char, satisfy, space0, space1},
    combinator::{eof, map, opt, rest, success},
    multi::many1,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Event<'a> {
    pub data: EventData<'a>,
    pub position: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum EventData<'a> {
    /// Description of command or subcommand. e.g. `@summary A demo cli`
    Description(&'a str),
    /// Define a subcommand, e.g. `@cmd A sub command`
    Cmd(&'a str),
    /// Define a arguemtn
    Arg(ArgData<'a>),
    /// A shell function. e.g `function cmd()` or `cmd()`
    Func(&'a str),
    /// Palaceholder for unrecognized tag
    Unknown(&'a str),
}

/// Tokenize shell script
pub fn parse(source: &str) -> Result<Vec<Event>> {
    let mut result = vec![];
    for (position, line) in source.lines().enumerate() {
        match parse_line(line) {
            Ok((_, maybe_token)) => {
                if let Some(value) = maybe_token {
                    result.push(Event {
                        position,
                        data: value,
                    })
                }
            }
            Err(err) => {
                bail!("Parse fail code at line {}, {}", line, err)
            }
        }
    }
    Ok(result)
}

fn parse_line(line: &str) -> nom::IResult<&str, Option<EventData>> {
    alt((map(alt((parse_tag, parse_fn)), |v| Some(v)), success(None)))(line)
}

fn parse_tag(input: &str) -> nom::IResult<&str, EventData> {
    preceded(
        tuple((char('#'), space0, char('@'))),
        alt((
            map(
                preceded(tag("description"), alt((parse_tail, parse_empty))),
                |v| EventData::Description(v),
            ),
            map(preceded(tag("cmd"), alt((parse_tail, parse_empty))), |v| {
                EventData::Cmd(v)
            }),
            map(
                alt((
                    preceded(pair(tag("option"), space1), parse_option_arg),
                    preceded(pair(tag("flag"), space1), parse_flag_arg),
                    preceded(pair(tag("arg"), space1), parse_param_arg),
                )),
                |v| EventData::Arg(v),
            ),
            map(parse_name, |v| EventData::Unknown(v)),
        )),
    )(input)
}

fn parse_fn(input: &str) -> nom::IResult<&str, EventData> {
    map(alt((parse_fn_keyword, parse_fn_elision)), |v| {
        EventData::Func(v)
    })(input)
}

// Parse fn likes `function foo`
fn parse_fn_keyword(input: &str) -> nom::IResult<&str, &str> {
    preceded(tuple((space0, tag("function"), space1)), parse_name)(input)
}

// Parse fn likes `foo ()`
fn parse_fn_elision(input: &str) -> nom::IResult<&str, &str> {
    preceded(
        space0,
        terminated(parse_name, tuple((space0, char('('), space0, char(')')))),
    )(input)
}

// Parse `@option`
fn parse_option_arg(input: &str) -> nom::IResult<&str, ArgData> {
    let (input, (short, mut arg, choices, summary)) = tuple((
        opt(parse_arg_short),
        parse_arg_long_suffix,
        opt(parse_arg_choices),
        alt((parse_tail, parse_empty)),
    ))(input)?;
    arg.short = short;
    arg.choices = choices;
    arg.summary = Some(summary);
    Ok((input, arg))
}

// Parse `@flag`
fn parse_flag_arg(input: &str) -> nom::IResult<&str, ArgData> {
    let (input, (short, mut arg, summary)) = tuple((
        opt(parse_arg_short),
        parse_arg_long,
        alt((parse_tail, parse_empty)),
    ))(input)?;
    arg.short = short;
    arg.summary = Some(summary);
    arg.flag = true;
    Ok((input, arg))
}

// Parse `@arg`
fn parse_param_arg(input: &str) -> nom::IResult<&str, ArgData> {
    let (i, (mut arg, summary)) =
        tuple((parse_param_arg_suffix, alt((parse_tail, parse_empty))))(input)?;
    arg.summary = Some(summary);
    Ok((i, arg))
}

// Parse `str!` `str*` `str+`
fn parse_arg_long_suffix(input: &str) -> nom::IResult<&str, ArgData> {
    alt((
        map(terminated(parse_arg_long, tag("!")), |mut arg| {
            arg.required = true;
            arg
        }),
        map(terminated(parse_arg_long, tag("*")), |mut arg| {
            arg.multiple = true;
            arg
        }),
        map(terminated(parse_arg_long, tag("+")), |mut arg| {
            arg.required = true;
            arg.multiple = true;
            arg
        }),
        parse_arg_assign,
        parse_arg_long,
    ))(input)
}

// Parse `str=value`
fn parse_arg_assign(input: &str) -> nom::IResult<&str, ArgData> {
    map(
        separated_pair(parse_arg_long, char('='), parse_value),
        |(mut arg, value)| {
            arg.default = Some(value);
            arg
        },
    )(input)
}

// Parse `--str`
fn parse_arg_long(input: &str) -> nom::IResult<&str, ArgData> {
    map(preceded(pair(space0, tag("--")), parse_name), |v| {
        ArgData::new(v)
    })(input)
}

fn parse_param_arg_suffix(input: &str) -> nom::IResult<&str, ArgData> {
    alt((
        map(terminated(parse_param_arg_name, tag("!")), |mut arg| {
            arg.required = true;
            arg
        }),
        map(terminated(parse_param_arg_name, tag("*")), |mut arg| {
            arg.multiple = true;
            arg
        }),
        map(terminated(parse_param_arg_name, tag("+")), |mut arg| {
            arg.required = true;
            arg.multiple = true;
            arg
        }),
        parse_param_arg_name,
    ))(input)
}

// Parse `str`
fn parse_param_arg_name(input: &str) -> nom::IResult<&str, ArgData> {
    map(preceded(space0, parse_name), |v| {
        let mut arg = ArgData::new(v);
        arg
    })(input)
}

// Parse `-s`
fn parse_arg_short(input: &str) -> nom::IResult<&str, char> {
    preceded(
        pair(space0, char('-')),
        satisfy(|c| c.is_ascii_alphabetic()),
    )(input)
}

// Parse `[1|2|3]`
fn parse_arg_choices(input: &str) -> nom::IResult<&str, Vec<&str>> {
    preceded(space1, delimited(char('['), parse_choices, char(']')))(input)
}

// Parse `1|2|3`
fn parse_choices(input: &str) -> nom::IResult<&str, Vec<&str>> {
    let (input, (value, other_values)) =
        pair(parse_value, many1(preceded(char('|'), parse_value)))(input)?;
    let mut result = vec![value];
    result.extend(other_values);
    Ok((input, result))
}

fn parse_value(input: &str) -> nom::IResult<&str, &str> {
    alt((parse_name, parse_quote))(input)
}

fn parse_tail(input: &str) -> nom::IResult<&str, &str> {
    map(preceded(space1, rest), |v: &str| v.trim())(input)
}

fn parse_empty(input: &str) -> nom::IResult<&str, &str> {
    map(alt((eof, preceded(space1, eof))), |v: &str| v.trim())(input)
}

fn parse_name(input: &str) -> nom::IResult<&str, &str> {
    take_while(|c: char| c.is_ascii_alphanumeric() || c == '_')(input)
}

fn parse_quote(input: &str) -> nom::IResult<&str, &str> {
    preceded(char('\"'), terminated(alphanumeric1, char('\"')))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_token {
        ($comment:literal, None) => {
            assert_eq!(parse_line($comment).unwrap().1, None)
        };
        ($comment:literal, $kind:ident) => {
            assert!(
                if let Some(EventData::$kind(_)) = parse_line($comment).unwrap().1 {
                    true
                } else {
                    false
                }
            );
        };
        ($comment:literal, $kind:ident, $text:literal) => {
            assert_eq!(
                parse_line($comment).unwrap().1,
                Some(EventData::$kind($text))
            )
        };
    }

    macro_rules! assert_arg {
        (option, $text:literal, $($k:ident : $v:expr),+ $(,)?) => {
            {
                let (_, arg) = parse_option_arg($text).unwrap();
                $(assert_eq!(arg.$k, $v);)+
            }
        };
        (flag, $text:literal, $($k:ident : $v:expr),+ $(,)?) => {
            {
                let (_, arg) = parse_flag_arg($text).unwrap();
                $(assert_eq!(arg.$k, $v);)+
            }
        };
        (arg, $text:literal, $($k:ident : $v:expr),+ $(,)?) => {
            {
                let (_, arg) = parse_param_arg($text).unwrap();
                $(assert_eq!(arg.$k, $v);)+
            }
        };
    }

    #[test]
    fn test_parse_line() {
        assert_token!("# @description A demo cli", Description, "A demo cli");
        assert_token!("# @cmd", Cmd, "");
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
        assert_token!("foo=bar", None);
        assert_token!("#!", None);
    }

    #[test]
    fn test_parse_option_arg() {
        assert_arg!(option, "--foo", name: "foo", required: false);
        assert_arg!(option, "--foo!", required: true);
        assert_arg!(option, "--foo+", multiple: true, required: true);
        assert_arg!(option, "--foo*", multiple: true, required: false);
        assert_arg!(option, "-f --foo", short: Some('f'));
        assert_arg!(option, "--foo=a", default: Some("a"), required: false);
        assert_arg!(
            option,
            "--foo=a [a|b|c]",
            choices: Some(vec!["a", "b", "c"])
        );
        assert_arg!(
            option,
            "--foo [a|b|c]",
            choices: Some(vec!["a", "b", "c"]),
            default: None
        );
        assert_arg!(option, "--foo A foo option", summary: Some("A foo option"));
        assert_arg!(option, "--foo", summary: Some(""));
        assert_arg!(option, "--foo ", summary: Some(""));
    }

    #[test]
    fn test_parse_flag_arg() {
        assert_arg!(flag, "--foo", name: "foo", flag: true);
        assert_arg!(flag, "-f --foo", flag: true, short: Some('f'));
        assert_arg!(option, "--foo A foo flag", summary: Some("A foo flag"));
        assert_arg!(option, "--foo", summary: Some(""));
        assert_arg!(option, "--foo ", summary: Some(""));
    }

    #[test]
    fn test_parse_param_arg() {
        assert_arg!(arg, "foo", name: "foo", required: false);
        assert_arg!(arg, "foo!", required: true);
        assert_arg!(arg, "foo+", multiple: true, required: true);
        assert_arg!(arg, "foo*", multiple: true, required: false);
        assert_arg!(arg, "foo A foo argument", summary: Some("A foo argument"));
        assert_arg!(arg, "foo ", summary: Some(""));
        assert_arg!(arg, "foo", summary: Some(""));
    }
}
