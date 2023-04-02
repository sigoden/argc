use anyhow::{anyhow, Context, Result};
use convert_case::{Boundary, Converter, Pattern};

/// Transform into upper case string with an underscore between words. `foo-bar` => `FOO-BAR`
pub fn to_cobol_case(value: &str) -> String {
    Converter::new()
        .set_pattern(Pattern::Uppercase)
        .set_delim("-")
        .set_boundaries(&[Boundary::Underscore, Boundary::LowerUpper, Boundary::Hyphen])
        .convert(value)
}

pub fn hyphens_to_underscores(name: &str) -> String {
    name.replace('-', "_")
}

pub fn escape_shell_words(value: &str) -> String {
    shell_words::quote(value).to_string()
}

pub fn split_shell_words(s: &str) -> Result<Vec<String>> {
    shell_words::split(s).with_context(|| anyhow!("Failed to split shell words"))
}

pub fn is_choice_value_terminate(c: char) -> bool {
    c == '|' || c == ']'
}

pub fn is_default_value_terminate(c: char) -> bool {
    c.is_whitespace()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cobol() {
        assert_eq!("FOO-BAR".to_string(), to_cobol_case("fooBar"));
        assert_eq!("FOO-BAR".to_string(), to_cobol_case("foo-bar"));
        assert_eq!("FOO1".to_string(), to_cobol_case("foo1"));
    }
}
