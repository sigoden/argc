use crate::arg::Arg;
use crate::throw;
use crate::Result;
use regex::Regex;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// App info. e.g. `@title A demo cli`
    AppTag(Text),
    /// App subcommand, e.g. `@cmd A sub command`
    CmdTag(Text),
    /// Option for app or subommand, e.g. `@option {string} str - A string option`
    OptionTag(Opt),
    /// Positionl parameter for app or subommand, e.g. `@param {string} str - A string option`
    ParamTag(Opt),
    /// A shell function. e.g `function cmd()` or `cmd()`
    Func(Text),
    /// Palaceholder for unrecognized tag
    UnknownTag(Text),
}

/// The line number of 
pub type Position = usize;

#[derive(Debug, PartialEq, Clone)]
pub struct Text {
    pub position: Position,
    pub text: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Opt {
    pub position: Position,
    pub arg: Arg,
}

/// Tokenize shell script
pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut result = vec![];
    let re_tag = Regex::new(r"^#\s+@(\w+)(.*)$").unwrap();
    let re_func1 = Regex::new(r"^\s*function\s+(\w+)").unwrap();
    let re_func2 = Regex::new(r"^\s*(\w+)\s*\(\s*\)").unwrap();
    for (position, line) in source.lines().enumerate() {
        if line.starts_with("#") {
            if let Some(capture) = re_tag.captures(&line) {
                let name = &capture[1];
                let extra = &capture[2];
                let text = extra.to_owned();
                let parsed: Result<Token> = match name {
                    "app" => Ok(Token::AppTag(Text { position, text })),
                    "cmd" => Ok(Token::CmdTag(Text { position, text })),
                    "option" => {
                        let arg = Arg::parse(&text)?;
                        Ok(Token::OptionTag(Opt { position, arg }))
                    }
                    "param" => {
                        let arg = Arg::parse(&text)?;
                        Ok(Token::ParamTag(Opt { position, arg }))
                    }
                    _ => Ok(Token::UnknownTag(Text {
                        position,
                        text: name.to_owned(),
                    })),
                };
                match parsed {
                    Ok(token) => result.push(token),
                    Err(err) => {
                        throw!("{} at line {}", err, position)
                    }
                }
            }
        } else {
            if let Some(capture) = re_func1.captures(&line).or(re_func2.captures(&line)) {
                let name = &capture[1];
                let text = name.to_owned();
                result.push(Token::Func(Text { position, text }));
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_token {
        ($comment:literal, $tag:ident) => {
            assert!(if let Token::$tag(_) = tokenize($comment).unwrap()[0] {
                true
            } else {
                false
            })
        };
        ($comment:literal, $tag:ident, $text:literal) => {
            assert_eq!(
                tokenize($comment).unwrap()[0],
                Token::$tag(Text {
                    position: 0,
                    text: $text.to_owned()
                })
            )
        };
    }

    #[test]
    fn test_parse() {
        assert_token!("# @app A demo cli", AppTag, " A demo cli");
        assert_token!("# @cmd A sub command", CmdTag, " A sub command");
        assert_token!("foo()", Func, "foo");
        assert_token!("foo ()", Func, "foo");
        assert_token!("foo  ()", Func, "foo");
        assert_token!("foo ( )", Func, "foo");
        assert_token!(" foo ()", Func, "foo");
        assert_token!("function foo", Func, "foo");
        assert_token!("function  foo", Func, "foo");
        assert_token!(" function foo", Func, "foo");
        assert_token!("# @option {string} str - A string option", OptionTag);
        assert_token!("# @param {string} str1_s - A string param option", ParamTag);
    }
}
