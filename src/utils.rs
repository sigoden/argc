use convert_case::{Boundary, Converter};

pub const VARIABLE_PREFIX: &str = "argc_";
pub const BEFORE_HOOK: &str = "_argc_before";
pub const AFTER_HOOK: &str = "_argc_after";
pub const ROOT_NAME: &str = "prog";
pub const MAIN_NAME: &str = "main";

pub(crate) const META_VERSION: &str = "version";
pub(crate) const META_BINNAME: &str = "binname";
pub(crate) const META_DOTENV: &str = "dotenv";
pub(crate) const META_DEFAULT_SUBCOMMAND: &str = "default-subcommand";
pub(crate) const META_INHERIT_FLAG_OPTIONS: &str = "inherit-flag-options";
pub(crate) const META_SYMBOL: &str = "symbol";
pub(crate) const META_COMBINE_SHORTS: &str = "combine-shorts";
pub(crate) const META_MAN_SECTION: &str = "man-section";
pub(crate) const META_REQUIRE_TOOLS: &str = "require-tools";

pub(crate) const MAX_ARGS: usize = 32767;

#[cfg(any(feature = "build", feature = "eval-bash"))]
pub const ARGC_REQUIRE_TOOLS: &str = include_str!("template/require_tools.sh");

#[cfg(any(feature = "build", feature = "eval-bash"))]
pub const ARGC_REQUIRE_PARAMS: &str = include_str!("template/require_params.sh");

#[cfg(any(feature = "build", feature = "eval-bash"))]
pub const ARGC_LOAD_DOTENV: &str = include_str!("template/load_dotenv.sh");

pub fn to_cobol_case(value: &str) -> String {
    Converter::new()
        .set_pattern(convert_case::Pattern::Uppercase)
        .set_delimiter("-")
        .set_boundaries(&[Boundary::Underscore, Boundary::LowerUpper, Boundary::Hyphen])
        .convert(value)
}

pub fn escape_shell_words(value: &str) -> String {
    shell_words::quote(value).to_string()
}

pub fn is_quote_char(c: char) -> bool {
    c == '\'' || c == '"'
}

pub fn unbalance_quote(value: &str) -> Option<(char, usize)> {
    let mut balance = None;
    for (i, c) in value.chars().enumerate() {
        match balance {
            Some((c_, _)) => {
                if c == c_ {
                    balance = None
                }
            }
            None => {
                if is_quote_char(c) {
                    balance = Some((c, i))
                }
            }
        }
    }
    balance
}

pub fn is_windows_path(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    ('a'..='z').any(|v| {
        if value.len() == 2 {
            value == format!("{v}:")
        } else {
            value.starts_with(&format!("{v}:/"))
        }
    })
}

pub fn is_special_var_char(c: char) -> bool {
    matches!(c, '-' | '.' | ':' | '@')
}

pub fn sanitize_var_name(id: &str) -> String {
    id.replace(is_special_var_char, "_")
}

pub fn argc_var_name(id: &str) -> String {
    format!("{VARIABLE_PREFIX}{}", sanitize_var_name(id))
}

pub fn is_true_value(value: &str) -> bool {
    matches!(value, "true" | "1")
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
