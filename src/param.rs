use crate::utils::{
    escape_shell_words, is_choice_value_terminate, is_default_value_terminate, to_cobol_case,
};
use crate::ArgcValue;

use serde::Serialize;
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct FlagOptionParam {
    pub(crate) describe: String,
    pub(crate) short: Option<char>,
    pub(crate) flag: bool,
    pub(crate) sign: char,
    pub(crate) single: bool,
    pub(crate) data: ParamData,
    pub(crate) value_names: Vec<String>,
    pub(crate) arg_value_names: Vec<String>,
    pub(crate) inherit: bool,
}

impl FlagOptionParam {
    pub(crate) fn new(
        param: ParamData,
        describe: &str,
        short: Option<char>,
        flag: bool,
        sign: char,
        single: bool,
        value_names: &[&str],
    ) -> Self {
        let name = param.name.clone();
        let value_names: Vec<String> = value_names.iter().map(|v| v.to_string()).collect();
        let mut arg_value_names = if flag {
            vec![]
        } else if value_names.is_empty() {
            vec![to_cobol_case(&name)]
        } else {
            value_names.iter().map(|v| to_cobol_case(v)).collect()
        };
        if param.terminated() {
            let last_arg = arg_value_names.last_mut().unwrap();
            last_arg.push('~')
        }
        Self {
            describe: describe.to_string(),
            short,
            flag,
            sign,
            single,
            data: param,
            value_names,
            arg_value_names,
            inherit: false,
        }
    }

    pub(crate) fn name(&self) -> &str {
        self.data.name.as_str()
    }

    pub(crate) fn is_flag(&self) -> bool {
        self.flag
    }

    pub(crate) fn is_option(&self) -> bool {
        !self.is_flag()
    }

    pub(crate) fn tag_name(&self) -> &str {
        if self.is_flag() {
            "@flag"
        } else {
            "@option"
        }
    }

    pub(crate) fn multiple(&self) -> bool {
        self.multi_occurs() || self.multi_values()
    }

    pub(crate) fn multi_occurs(&self) -> bool {
        self.data.multi_occurs()
    }

    pub(crate) fn multi_values(&self) -> bool {
        self.unlimited_args() || self.arg_value_names.len() > 1
    }

    pub(crate) fn unlimited_args(&self) -> bool {
        self.terminated() || !self.notation_modifer().is_none()
    }

    pub(crate) fn validate_args_len(&self, num: usize) -> bool {
        let len = self.arg_value_names.len();
        if self.unlimited_args() {
            let min = if len > 1 && self.notation_modifer() == NotationModifier::Asterisk {
                len - 1
            } else {
                len
            };
            num >= min
        } else {
            num == len
        }
    }

    pub(crate) fn notation_modifer(&self) -> NotationModifier {
        if let Some(notation) = self.arg_value_names.last() {
            if notation.ends_with('*') {
                return NotationModifier::Asterisk;
            } else if notation.ends_with('+') {
                return NotationModifier::Plus;
            }
        }
        NotationModifier::None
    }

    pub(crate) fn required(&self) -> bool {
        self.data.required()
    }

    pub(crate) fn terminated(&self) -> bool {
        self.data.terminated()
    }

    pub(crate) fn multi_char(&self) -> Option<char> {
        self.data.multi_char()
    }

    pub(crate) fn choice_fn(&self) -> Option<(&String, &bool)> {
        self.data.choice_fn()
    }

    pub(crate) fn default_fn(&self) -> Option<&String> {
        self.data.default_fn()
    }

    #[allow(unused)]
    pub(crate) fn render_source(&self) -> String {
        let mut output = vec![];
        if let Some(ch) = self.short {
            output.push(format!("{}{}", self.sign, ch));
        };
        output.push(format!(
            "{}{}",
            self.render_long_prefix(),
            self.data.render_name_value()
        ));
        for value_name in &self.value_names {
            output.push(format!("<{}>", value_name));
        }
        if !self.describe.is_empty() {
            output.push(self.describe.clone());
        }
        output.join(" ")
    }

    pub(crate) fn render_long_prefix(&self) -> &str {
        if self.single {
            if self.sign == '+' {
                "+"
            } else {
                "-"
            }
        } else {
            "--"
        }
    }

    pub(crate) fn render_name(&self) -> String {
        format!("{}{}", self.render_long_prefix(), self.name())
    }

    pub(crate) fn render_first_notation(&self) -> String {
        format!("<{}>", self.arg_value_names[0])
    }

    pub(crate) fn render_name_notations(&self) -> String {
        let mut output = self.render_name();
        if !self.is_flag() {
            output.push(' ');
            output.push_str(&self.render_notations());
        }
        output
    }

    pub(crate) fn render_body(&self) -> String {
        let mut output = String::new();
        let sign = self.sign;
        if self.single && self.short.is_none() && self.name().len() == 1 {
            output.push_str(&format!("{sign}{}", self.name()));
        } else {
            if let Some(ch) = self.short {
                output.push_str(&format!("{sign}{ch}, "))
            } else {
                output.push_str("    ")
            };
            output.push_str(&format!("{:>2}", self.render_long_prefix()));
            output.push_str(self.name());
        }

        if self.is_flag() {
            if self.multi_occurs() {
                output.push_str("...")
            }
        } else {
            output.push(' ');
            output.push_str(&self.render_notations());
        }
        output
    }

    pub(crate) fn render_notations(&self) -> String {
        if self.is_flag() {
            return String::new();
        }
        let mut output = String::new();
        if self.arg_value_names.len() == 1 {
            let name: &String = &self.arg_value_names[0];
            let value = match (self.required(), self.multi_occurs()) {
                (true, true) => format!("<{name}>..."),
                (false, true) => format!("[{name}]..."),
                (_, false) => format!("<{name}>"),
            };
            output.push_str(&value);
        } else {
            let values = self
                .arg_value_names
                .iter()
                .map(|v| format!("<{v}>"))
                .collect::<Vec<String>>();
            output.push_str(&values.join(" "));
        }
        output
    }

    pub(crate) fn render_describe(&self) -> String {
        self.data.render_describe(&self.describe)
    }

    pub(crate) fn get_arg_value(&self, values: &[&[&str]]) -> Option<ArgcValue> {
        let mut name = self.name().to_string();
        if self.sign == '+' {
            name = format!("plus_{name}")
        }
        if self.flag {
            if values.is_empty() {
                None
            } else {
                Some(ArgcValue::Single(name, values.len().to_string()))
            }
        } else {
            if values.is_empty() {
                match &self.data.default {
                    Some(DefaultData::Value(value)) => {
                        return Some(ArgcValue::Single(name, value.clone()));
                    }
                    Some(DefaultData::Fn(f)) => {
                        return Some(ArgcValue::SingleFn(name, f.clone()));
                    }
                    None => return None,
                }
            }
            if self.multiple() {
                let mut values: Vec<String> = values
                    .iter()
                    .flat_map(|v| v.iter().map(|v| v.to_string()))
                    .collect();
                if let Some(c) = self.multi_char() {
                    values = values
                        .into_iter()
                        .flat_map(|v| {
                            v.split(c)
                                .filter(|v| !v.is_empty())
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                        })
                        .collect()
                }
                Some(ArgcValue::Multiple(name, values))
            } else if self.arg_value_names.len() > 1 {
                Some(ArgcValue::Multiple(
                    name,
                    values[0].iter().map(|v| v.to_string()).collect(),
                ))
            } else {
                Some(ArgcValue::Single(name, must_get_first(values[0])))
            }
        }
    }

    pub(crate) fn is_match(&self, name: &str) -> bool {
        self.list_names().iter().any(|v| v == name)
    }

    pub(crate) fn prefixed(&self) -> Option<String> {
        if !matches!(
            self.data.modifer,
            Modifier::Prefixed | Modifier::MultiPrefixed
        ) {
            return None;
        }

        if let Some(ch) = self.short {
            return Some(format!("{}{ch}", self.sign));
        }

        Some(self.render_name())
    }

    pub(crate) fn list_names(&self) -> Vec<String> {
        let mut output = vec![];
        output.push(format!("{}{}", self.render_long_prefix(), self.name()));
        if let Some(short) = self.short {
            output.push(format!("{}{}", self.sign, short));
        }
        output
    }

    pub(crate) fn describe_oneline(&self) -> &str {
        match self.describe.split_once('\n') {
            Some((v, _)) => v,
            None => self.describe.as_str(),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        let option_names = self.list_names();
        json!({
            "name": self.name(),
            "describe": self.describe,
            "flag": self.flag,
            "option_names": option_names,
            "value_names": self.value_names,
            "modifier": self.data.modifer,
            "choices": self.data.choice_values(),
            "default": self.data.default_value(),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct PositionalParam {
    pub(crate) describe: String,
    pub(crate) data: ParamData,
    pub(crate) value_name: Option<String>,
    pub(crate) arg_value_name: String,
}

impl PositionalParam {
    pub(crate) fn new(param: ParamData, describe: &str, value_name: Option<&str>) -> Self {
        let name = param.name.clone();
        PositionalParam {
            describe: describe.to_string(),
            data: param,
            value_name: value_name.map(|v| v.to_string()),
            arg_value_name: value_name
                .or(Some(&name))
                .map(to_cobol_case)
                .unwrap_or_default(),
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.data.name
    }

    pub(crate) fn tag_name(&self) -> &str {
        "@arg"
    }

    pub(crate) fn multiple(&self) -> bool {
        self.data.multi_occurs() || self.terminated()
    }

    pub(crate) fn required(&self) -> bool {
        self.data.required()
    }

    pub(crate) fn terminated(&self) -> bool {
        self.data.terminated()
    }

    pub(crate) fn multi_char(&self) -> Option<char> {
        self.data.multi_char()
    }

    pub(crate) fn choice_fn(&self) -> Option<(&String, &bool)> {
        self.data.choice_fn()
    }

    pub(crate) fn default_fn(&self) -> Option<&String> {
        self.data.default_fn()
    }

    #[allow(unused)]
    pub(crate) fn render_source(&self) -> String {
        let mut output = vec![];
        output.push(self.data.render_name_value());
        if let Some(value_name) = self.value_name.as_ref() {
            output.push(format!("<{}>", value_name));
        }
        if !self.describe.is_empty() {
            output.push(self.describe.clone());
        }
        output.join(" ")
    }

    pub(crate) fn render_value(&self) -> String {
        let name: &String = &self.arg_value_name;
        match (self.required(), self.multiple()) {
            (true, true) => format!("<{name}>..."),
            (true, false) => format!("<{name}>"),
            (false, true) => format!("[{name}]..."),
            (false, false) => format!("[{name}]"),
        }
    }

    pub(crate) fn render_describe(&self) -> String {
        self.data.render_describe(&self.describe)
    }

    pub(crate) fn get_arg_value(&self, values: &[&str]) -> Option<ArgcValue> {
        let name = self.name().to_string();
        if values.is_empty() {
            match &self.data.default {
                Some(DefaultData::Value(value)) => {
                    return Some(ArgcValue::PositionalSingle(name, value.clone()));
                }
                Some(DefaultData::Fn(f)) => {
                    return Some(ArgcValue::PositionalSingleFn(name, f.clone()));
                }
                None => return None,
            }
        }
        if self.multiple() {
            let mut values: Vec<String> = values.iter().map(|v| v.to_string()).collect();
            if let Some(c) = self.multi_char() {
                values = values
                    .into_iter()
                    .flat_map(|v| {
                        v.split(c)
                            .filter(|v| !v.is_empty())
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                    })
                    .collect()
            }
            Some(ArgcValue::PositionalMultiple(name, values))
        } else {
            Some(ArgcValue::PositionalSingle(name, must_get_first(values)))
        }
    }

    pub(crate) fn describe_oneline(&self) -> &str {
        match self.describe.split_once('\n') {
            Some((v, _)) => v,
            None => self.describe.as_str(),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "name": self.name(),
            "describe": self.describe,
            "modifier": self.data.modifer,
            "choices": self.data.choice_values(),
            "default": self.data.default_value(),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct ParamData {
    pub(crate) name: String,
    pub(crate) choice: Option<ChoiceData>,
    pub(crate) default: Option<DefaultData>,
    pub(crate) modifer: Modifier,
}

impl ParamData {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            choice: None,
            default: None,
            modifer: Modifier::Optional,
        }
    }

    pub(crate) fn multi_occurs(&self) -> bool {
        self.modifer.multi_occurs()
    }

    pub(crate) fn required(&self) -> bool {
        self.modifer.required() && self.default.is_none()
    }

    pub(crate) fn multi_char(&self) -> Option<char> {
        match &self.modifer {
            Modifier::MultiCharRequired(c) | Modifier::MultiCharOptional(c) => Some(*c),
            _ => None,
        }
    }

    pub(crate) fn terminated(&self) -> bool {
        matches!(self.modifer, Modifier::Terminated)
    }

    pub(crate) fn choice_fn(&self) -> Option<(&String, &bool)> {
        match &self.choice {
            Some(ChoiceData::Fn(f, v)) => Some((f, v)),
            _ => None,
        }
    }

    pub(crate) fn choice_values(&self) -> Option<&Vec<String>> {
        match &self.choice {
            Some(ChoiceData::Values(v)) => Some(v),
            _ => None,
        }
    }

    pub(crate) fn default_fn(&self) -> Option<&String> {
        match &self.default {
            Some(DefaultData::Fn(f)) => Some(f),
            _ => None,
        }
    }

    pub(crate) fn default_value(&self) -> Option<&String> {
        match &self.default {
            Some(DefaultData::Value(v)) => Some(v),
            _ => None,
        }
    }

    pub(crate) fn render_name_value(&self) -> String {
        let mut result = self.name.to_string();
        result.push_str(&self.modifer.render());
        match (&self.choice, &self.default) {
            (Some(ChoiceData::Values(values)), None) => {
                result.push_str(&format!("[{}]", Self::render_choice_values(values)));
            }
            (Some(ChoiceData::Values(values)), Some(DefaultData::Value(_))) => {
                result.push_str(&format!("[={}]", Self::render_choice_values(values)));
            }
            (Some(ChoiceData::Fn(f, validate)), _) => {
                let prefix = if *validate { "" } else { "?" };
                result.push_str(&format!("[{prefix}`{f}`]"));
            }
            (None, Some(DefaultData::Value(value))) => {
                result.push_str(&format!("={}", Self::render_default_value(value)));
            }
            (None, Some(DefaultData::Fn(f))) => {
                result.push_str(&format!("=`{f}`"));
            }
            _ => {}
        }
        result
    }

    pub(crate) fn render_describe(&self, describe: &str) -> String {
        let mut output = describe.to_string();
        if let Some(DefaultData::Value(value)) = &self.default {
            if !output.is_empty() {
                output.push(' ')
            }
            output.push_str(&format!("[default: {}]", escape_shell_words(value)));
        }
        if let Some(ChoiceData::Values(values)) = &self.choice {
            if !output.is_empty() {
                output.push(' ')
            }
            let values: Vec<String> = values.iter().map(|v| escape_shell_words(v)).collect();
            output.push_str(&format!("[possible values: {}]", values.join(", ")));
        }
        output
    }

    fn render_choice_values(values: &[String]) -> String {
        let values: Vec<String> = values
            .iter()
            .map(|value| {
                if value.chars().any(is_choice_value_terminate) {
                    format!("\"{}\"", value)
                } else {
                    value.to_string()
                }
            })
            .collect();
        values.join("|")
    }

    fn render_default_value(value: &str) -> String {
        if value.chars().any(is_default_value_terminate) {
            format!("\"{}\"", value)
        } else {
            value.to_string()
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub(crate) enum NotationModifier {
    None,
    Plus,
    Asterisk,
}

impl NotationModifier {
    pub(crate) fn is_none(&self) -> bool {
        self == &NotationModifier::None
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub(crate) enum Modifier {
    Optional,
    Required,
    MultipleOptional,
    MultipleRequired,
    MultiCharOptional(char),
    MultiCharRequired(char),
    Terminated,
    Prefixed,
    MultiPrefixed,
}

impl Modifier {
    pub(crate) fn multi_occurs(&self) -> bool {
        match self {
            Self::Optional => false,
            Self::Required => false,
            Self::MultipleOptional => true,
            Self::MultipleRequired => true,
            Self::MultiCharOptional(_) => true,
            Self::MultiCharRequired(_) => true,
            Self::Terminated => false,
            Self::Prefixed => false,
            Self::MultiPrefixed => true,
        }
    }

    pub(crate) fn required(&self) -> bool {
        match self {
            Self::Optional => false,
            Self::Required => true,
            Self::MultipleOptional => false,
            Self::MultipleRequired => true,
            Self::MultiCharOptional(_) => false,
            Self::MultiCharRequired(_) => true,
            Self::Terminated => false,
            Self::Prefixed => false,
            Self::MultiPrefixed => false,
        }
    }

    pub(crate) fn render(&self) -> String {
        match self {
            Self::Optional => "".into(),
            Self::Required => "!".into(),
            Self::MultipleOptional => "*".into(),
            Self::MultipleRequired => "+".into(),
            Self::MultiCharOptional(c) => format!("*{c}"),
            Self::MultiCharRequired(c) => format!("+{c}"),
            Self::Terminated => "~".to_string(),
            Self::Prefixed => "-".to_string(),
            Self::MultiPrefixed => "-*".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub(crate) enum ChoiceData {
    Values(Vec<String>),
    Fn(String, bool),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub(crate) enum DefaultData {
    Value(String),
    Fn(String),
}

fn must_get_first(value: &[&str]) -> String {
    if value.is_empty() {
        String::new()
    } else {
        value[0].to_string()
    }
}
