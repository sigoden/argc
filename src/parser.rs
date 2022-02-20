use crate::{bail, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alphanumeric1, char, satisfy, space0, space1},
    combinator::{eof, map, opt, rest, success},
    multi::many0,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
    value: TokenData<'a>,
    position: Position,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenData<'a> {
    /// App info. e.g. `@title A demo cli`
    AppTag(&'a str),
    /// App subcommand, e.g. `@cmd A sub command`
    CmdTag(&'a str),
    /// Option for app or subommand, e.g. `@option {string} str - A string option`
    OptionTag(Arg<'a>),
    /// Positionl parameter for app or subommand, e.g. `@param {string} str - A string option`
    ParamTag(Arg<'a>),
    /// A shell function. e.g `function cmd()` or `cmd()`
    Func(&'a str),
    /// Palaceholder for unrecognized tag
    UnknownTag(&'a str),
}

/// The line number of
pub type Position = usize;

#[derive(Debug, PartialEq, Clone)]
pub struct Arg<'a> {
    pub name: &'a str,
    pub title: Option<&'a str>,
    pub arg_type: ArgType,
    pub short: Option<char>,
    pub choices: Option<Vec<&'a str>>,
    pub multiple: bool,
    pub required: bool,
    pub default: Option<&'a str>,
}

impl<'a> Arg<'a> {
    pub fn new(name: &'a str) -> Self {
        Arg {
            name,
            title: None,
            arg_type: ArgType::String,
            short: None,
            choices: None,
            multiple: false,
            required: false,
            default: None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArgType {
    String,
    Boolean,
    Number,
}

/// Tokenize shell script
pub fn parse(source: &str) -> Result<Vec<Token>> {
    let mut result = vec![];
    for (position, line) in source.lines().enumerate() {
        match parse_line(line) {
            Ok((_, maybe_token)) => {
                if let Some(value) = maybe_token {
                    result.push(Token { position, value })
                }
            }
            Err(err) => {
                bail!("Parse fail at {}, {}", line, err)
            }
        }
    }
    Ok(result)
}

fn parse_line(line: &str) -> nom::IResult<&str, Option<TokenData>> {
    alt((map(alt((parse_tag, parse_fn)), |v| Some(v)), success(None)))(line)
}

fn parse_tag(input: &str) -> nom::IResult<&str, TokenData> {
    preceded(
        tuple((char('#'), space0, char('@'))),
        alt((
            map(preceded(tag("app"), parse_tail), |v| TokenData::AppTag(v)),
            map(preceded(tag("cmd"), parse_tail), |v| TokenData::CmdTag(v)),
            map(preceded(tag("option"), parse_arg), |v| {
                TokenData::OptionTag(v)
            }),
            map(preceded(tag("param"), parse_arg), |v| {
                TokenData::ParamTag(v)
            }),
            map(parse_name, |v| TokenData::UnknownTag(v)),
        )),
    )(input)
}

fn parse_fn(input: &str) -> nom::IResult<&str, TokenData> {
    map(alt((parse_fn_keyword, parse_fn_elision)), |v| {
        TokenData::Func(v)
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

fn parse_arg(input: &str) -> nom::IResult<&str, Arg> {
    let (i, (mut arg, short, title)) = preceded(
        space0,
        tuple((parse_arg_quote, opt(parse_arg_short), parse_tail)),
    )(input)?;
    arg.short = short;
    arg.title = Some(title);
    Ok((i, arg))
}

// Parse arg  likes `<??>`
fn parse_arg_quote(input: &str) -> nom::IResult<&str, Arg> {
    alt((
        map(
            delimited(char('<'), parse_arg_general, char('>')),
            |mut arg| {
                arg.required = true;
                arg
            },
        ),
        parse_arg_general,
    ))(input)
}

fn parse_arg_general(input: &str) -> nom::IResult<&str, Arg> {
    alt((parse_arg_multiple, parse_arg_assign, parse_arg_base))(input)
}

// Parse `str...`
fn parse_arg_multiple(input: &str) -> nom::IResult<&str, Arg> {
    map(terminated(parse_arg_base, tag("...")), |mut arg| {
        arg.multiple = true;
        arg
    })(input)
}

// Parse `str=?`
fn parse_arg_assign(input: &str) -> nom::IResult<&str, Arg> {
    map(
        separated_pair(parse_arg_base, char('='), parse_choices),
        |(mut arg, choices)| {
            let value = choices[0].clone();
            if value == "true" || value == "false" {
                arg.arg_type = ArgType::Boolean;
            } else if value.chars().all(|v| v.is_numeric()) {
                arg.arg_type = ArgType::Number;
            }
            arg.default = Some(value);
            arg.choices = Some(choices);
            arg
        },
    )(input)
}

// Parse `str`
fn parse_arg_base(input: &str) -> nom::IResult<&str, Arg> {
    map(parse_name, |v| Arg::new(v))(input)
}

// Parse `-s`
fn parse_arg_short(input: &str) -> nom::IResult<&str, char> {
    preceded(pair(space1, char('-')), satisfy(|c| c.is_alphabetic()))(input)
}

// Parse `1|2|3` or `1`
fn parse_choices(input: &str) -> nom::IResult<&str, Vec<&str>> {
    let (input, (value, other_values)) =
        pair(parse_value, many0(preceded(char('|'), parse_value)))(input)?;
    let mut result = vec![value];
    result.extend(other_values);
    Ok((input, result))
}

fn parse_value(input: &str) -> nom::IResult<&str, &str> {
    alt((parse_name, parse_quote))(input)
}

fn parse_tail(input: &str) -> nom::IResult<&str, &str> {
    alt((eof, preceded(space1, rest)))(input)
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
        ($comment:literal, $value:expr) => {
            assert_eq!(parse_line($comment).unwrap().1, $value)
        };
        ($comment:literal, $kind:ident, $text:literal) => {
            assert_eq!(
                parse_line($comment).unwrap().1,
                Some(TokenData::$kind($text))
            )
        };
    }

    macro_rules! assert_arg {
        ($text:literal, $($k:ident : $v:expr),+ $(,)?) => {
            {
                let (_, arg) = parse_arg($text).unwrap();
                $(assert_eq!(arg.$k, $v);)+
            }
        };
    }

    #[test]
    fn test_parse_line() {
        assert_token!("# @app A demo cli", AppTag, "A demo cli");
        assert_token!("# @cmd A sub command", CmdTag, "A sub command");
        assert_token!("# @cmd", CmdTag, "");
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
    fn test_parse_arg() {
        assert_arg!("str", name: "str");
        assert_arg!("<str>  required", name: "str", required: true);
        assert_arg!("str...  multiple", name: "str", multiple: true);
        assert_arg!("str=hello  default", name: "str", default: Some("hello"));
        assert_arg!("str=a|b|c  choice", name: "str", default: Some("a"), choices: Some(vec!["a", "b", "c"]));
        assert_arg!("num=3   type: integer", arg_type: ArgType::Number, default: Some("3"));
        assert_arg!("bool=true  type: boolean", arg_type: ArgType::Boolean, default: Some("true"));
        assert_arg!("str -s short", short: Some('s'));
    }
}
