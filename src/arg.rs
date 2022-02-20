use crate::Result;

#[derive(Debug, PartialEq, Clone)]
pub struct Arg {
    pub name: String,
    pub title: Option<String>,
    pub kind: ArgType,
    pub array: bool,
    pub choices: Vec<String>,
    pub short: String,
    pub options: Vec<Arg>,
    pub params: Vec<Arg>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArgType {
    String,
    Boolean,
    Number,
}

impl Arg {
    pub fn parse(value: &str) -> Result<Self> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! assert_arg {
        ($text:literal, $name:literal, $kind:ident, $($k:ident : $v:expr),+ $(,)?) => {
            {
                let arg = Arg::parse($text).unwrap();
                assert_eq!(&arg.name, $name);
                $(assert_eq!(arg.$k, $v);)+
            }
        };
    }

    #[test]
    fn test_parse() {
        assert_arg!("{string} str - A string option", "str", String, title: Some("A string option".into()))
    }
}
