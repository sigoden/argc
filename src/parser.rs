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
    /// Describe of command or subcommand. e.g. `@summary A demo cli`
    Describe(&'a str),
    /// Version of command e.g. `@version 1.0.0`
    Version(&'a str),
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
                preceded(tag("describe"), alt((parse_tail, parse_empty))),
                |v| EventData::Describe(v),
            ),
            map(
                preceded(tag("version"), alt((parse_tail, parse_empty))),
                |v| EventData::Version(v),
            ),
            map(preceded(tag("cmd"), alt((parse_tail, parse_empty))), |v| {
                EventData::Cmd(v)
            }),
            map(
                alt((
                    preceded(pair(tag("option"), space1), alt((parse_option_arg, parse_positional_arg))),
                    preceded(pair(tag("flag"), space1), parse_flag_arg),
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
    let (input, (short, mut arg, summary)) = tuple((
        opt(parse_arg_short),
        preceded(
            pair(space0, tag("--")),
            alt((parse_arg_choices, parse_arg_assign, parse_arg_mark)),
        ),
        alt((parse_tail, parse_empty)),
    ))(input)?;
    arg.short = short;
    arg.summary = Some(summary);
    Ok((input, arg))
}

// Parse `@option`, positional only
fn parse_positional_arg(input: &str) -> nom::IResult<&str, ArgData> {
    let (i, (mut arg, summary)) = tuple((
        preceded(space0, parse_arg_mark),
        alt((parse_tail, parse_empty)),
    ))(input)?;
    arg.positional = true;
    arg.summary = Some(summary);
    Ok((i, arg))
}

// Parse `@flag`
fn parse_flag_arg(input: &str) -> nom::IResult<&str, ArgData> {
    let (input, (short, mut arg, summary)) = tuple((
        opt(parse_arg_short),
        preceded(pair(space0, tag("--")), parse_arg_name),
        alt((parse_tail, parse_empty)),
    ))(input)?;
    arg.short = short;
    arg.summary = Some(summary);
    arg.flag = true;
    Ok((input, arg))
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
        separated_pair(parse_arg_name, char('='), parse_value),
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
fn parse_arg_short(input: &str) -> nom::IResult<&str, char> {
    preceded(
        pair(space0, char('-')),
        satisfy(|c| c.is_ascii_alphabetic()),
    )(input)
}

// Parse `a|b|c`, `=a|b|c`
fn parse_choices(input: &str) -> nom::IResult<&str, (Vec<&str>, Option<&str>)> {
    let (input, (equal, value, other_values)) = tuple((
        opt(char('=')),
        parse_value,
        many1(preceded(char('|'), parse_value)),
    ))(input)?;
    let mut choices = vec![value];
    let default_choice = equal.map(|_| value);
    choices.extend(other_values);
    Ok((input, (choices, default_choice)))
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
    take_while(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-')(input)
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
        ($parser:tt, $text:literal, $($k:ident : $v:expr),+ $(,)?) => {
            {
                let (_, arg) = $parser($text).unwrap();
                $(assert_eq!(arg.$k, $v);)+
            }
        };
    }

    #[test]
    fn test_parse_line() {
        assert_token!("# @describe A demo cli", Describe, "A demo cli");
        assert_token!("# @version 1.0.0", Version, "1.0.0");
        assert_token!("# @cmd", Cmd, "");
        assert_token!("# @flag -f --foo", Arg);
        assert_token!("# @option -f --foo", Arg);
        assert_token!("# @positional foo", Arg);
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
        assert_arg!(parse_option_arg, "--foo", name: "foo", required: false);
        assert_arg!(parse_option_arg, "--foo!", required: true);
        assert_arg!(parse_option_arg, "--foo+", multiple: true, required: true);
        assert_arg!(parse_option_arg, "--foo*", multiple: true, required: false);
        assert_arg!(parse_option_arg, "-f --foo", short: Some('f'));
        assert_arg!(parse_option_arg, "--foo=a", default: Some("a"), required: false);
        assert_arg!(
            parse_option_arg,
            "--foo[=a|b|c]",
            choices: Some(vec!["a", "b", "c"]),
            default: Some("a")
        );
        assert_arg!(
            parse_option_arg,
            "--foo[a|b|c]",
            choices: Some(vec!["a", "b", "c"]),
            default: None
        );
        assert_arg!(parse_option_arg, "--foo A foo parse_option_arg", summary: Some("A foo parse_option_arg"));
        assert_arg!(parse_option_arg, "--foo", summary: Some(""));
        assert_arg!(parse_option_arg, "--foo ", summary: Some(""));
        assert_arg!(parse_option_arg, "--max-count ", name: "max-count");
    }

    #[test]
    fn test_parse_flag_arg() {
        assert_arg!(parse_flag_arg, "--foo", name: "foo", flag: true);
        assert_arg!(parse_flag_arg, "-f --foo",  flag: true, short: Some('f'));
        assert_arg!(parse_flag_arg, "--foo A foo parse_flat_arg", summary: Some("A foo parse_flat_arg"));
        assert_arg!(parse_flag_arg, "--foo", summary: Some(""));
        assert_arg!(parse_flag_arg, "--foo ", summary: Some(""));
    }

    #[test]
    fn test_parse_param_arg() {
        assert_arg!(parse_positional_arg, "foo", name: "foo", required: false, positional: true);
        assert_arg!(parse_positional_arg, "foo!", required: true);
        assert_arg!(parse_positional_arg, "foo+", multiple: true, required: true);
        assert_arg!(parse_positional_arg, "foo*", multiple: true, required: false);
        assert_arg!(parse_positional_arg, "foo A foo argument", summary: Some("A foo argument"));
        assert_arg!(parse_positional_arg, "foo ", summary: Some(""));
        assert_arg!(parse_positional_arg, "foo", summary: Some(""));
    }
}
