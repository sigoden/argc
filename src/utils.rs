use convert_case::{Boundary, Converter, Pattern};

/// Transform into a lower case string with underscores between words. `foo-bar` => `foo_bar`
pub fn to_snake_case(value: &str) -> String {
    Converter::new()
        .set_pattern(Pattern::Lowercase)
        .set_delim("_")
        .set_boundaries(&[Boundary::Underscore, Boundary::LowerUpper, Boundary::Hyphen])
        .convert(value)
}

/// Transform into a lower cased string with dashes between words. `foo_bar` => `foo-bar`
pub fn to_kebab_case(value: &str) -> String {
    Converter::new()
        .set_pattern(Pattern::Lowercase)
        .set_delim("-")
        .set_boundaries(&[
            Boundary::Underscore,
            Boundary::LowerUpper,
            Boundary::Underscore,
        ])
        .convert(value)
}

/// Transform into upper case string with an underscore between words. `foo-bar` => `FOO-BAR`
pub fn to_cobol_case(value: &str) -> String {
    Converter::new()
        .set_pattern(Pattern::Uppercase)
        .set_delim("-")
        .set_boundaries(&[Boundary::Underscore, Boundary::LowerUpper, Boundary::Hyphen])
        .convert(value)
}

pub fn escape_shell_words(value: &str) -> String {
    let mut output = String::new();
    if value.is_empty() {
        return "''".to_string();
    }
    for ch in value.chars() {
        match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '.' | ',' | ':' | '/' | '@' => {
                output.push(ch)
            }
            '\n' => output.push_str("'\n'"),
            _ => {
                output.push('\\');
                output.push(ch);
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake() {
        assert_eq!("foo_bar".to_string(), to_snake_case("fooBar"));
        assert_eq!("foo_bar".to_string(), to_snake_case("foo-bar"));
        assert_eq!("foo_bar".to_string(), to_snake_case("foo_bar"));
        assert_eq!("foo1".to_string(), to_snake_case("foo1"));
    }

    #[test]
    fn test_kebab() {
        assert_eq!("foo-bar".to_string(), to_kebab_case("fooBar"));
        assert_eq!("foo-bar".to_string(), to_kebab_case("foo_bar"));
        assert_eq!("foo1".to_string(), to_kebab_case("foo1"));
    }

    #[test]
    fn test_cobol() {
        assert_eq!("FOO-BAR".to_string(), to_cobol_case("fooBar"));
        assert_eq!("FOO-BAR".to_string(), to_cobol_case("foo-bar"));
        assert_eq!("FOO1".to_string(), to_cobol_case("foo1"));
    }
}
