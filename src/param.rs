use crate::parser::{is_choice_value_terminate, is_default_value_terminate};
use crate::utils::{argc_var_name, escape_shell_words, sanitize_var_name, to_cobol_case, MAX_ARGS};
use crate::ArgcValue;

use anyhow::{bail, Result};
use indexmap::IndexMap;
use serde::Serialize;

pub(crate) trait Param {
    fn data(&self) -> &ParamData;
    fn data_mut(&mut self) -> &mut ParamData;
    fn id(&self) -> &str;
    fn var_name(&self) -> String;
    fn tag_name(&self) -> &str;
    fn guard(&self) -> Result<()>;
    fn multiple_values(&self) -> bool;
    fn render_source(&self) -> String;
    fn describe_oneline(&self) -> &str {
        match self.describe().split_once('\n') {
            Some((v, _)) => v,
            None => self.describe(),
        }
    }
    fn render_describe(&self) -> String {
        self.data().render_describe(self.describe(), self.id())
    }
    fn describe(&self) -> &str {
        &self.data().describe
    }

    fn describe_mut(&mut self) -> &mut String {
        &mut self.data_mut().describe
    }

    fn required(&self) -> bool {
        self.data().required()
    }
    fn multiple_occurs(&self) -> bool {
        self.data().multiple_occurs()
    }
    fn args_delimiter(&self) -> Option<char> {
        self.data().args_delimiter()
    }
    fn terminated(&self) -> bool {
        self.data().terminated()
    }
    fn choice(&self) -> Option<&ChoiceValue> {
        self.data().choice.as_ref()
    }
    fn choice_fn(&self) -> Option<(&String, &bool)> {
        self.data().choice_fn()
    }
    fn choice_values(&self) -> Option<&Vec<String>> {
        self.data().choice_values()
    }
    fn default(&self) -> Option<&DefaultValue> {
        self.data().default.as_ref()
    }
    fn default_fn(&self) -> Option<&String> {
        self.data().default_fn()
    }
    fn default_value(&self) -> Option<&String> {
        self.data().default_value()
    }
    fn bind_env(&self) -> Option<String> {
        self.data().normalize_bind_env(self.id())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct FlagOptionParam {
    data: ParamData,
    id: String,
    is_flag: bool,
    short: Option<String>,
    long_prefix: String,
    prefixed: bool,
    assigned: bool,
    raw_notations: Vec<String>,
    notations: Vec<String>,
    inherit: bool,
}

impl Param for FlagOptionParam {
    fn data(&self) -> &ParamData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut ParamData {
        &mut self.data
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn var_name(&self) -> String {
        argc_var_name(self.id())
    }

    fn tag_name(&self) -> &str {
        if self.is_flag() {
            "@flag"
        } else {
            "@option"
        }
    }

    fn guard(&self) -> Result<()> {
        if self.notations.len() > 1 {
            if self.assigned {
                bail!("cannot combine assign and multiple notations")
            }
            if self.prefixed {
                bail!("cannot combine prefix and multiple notations")
            }
            if self.args_delimiter().is_some() {
                bail!("cannot combine delmiter and multiple notations")
            }
        }
        if self.prefixed && self.bind_env().is_some() {
            bail!("cannot bind env with prefixed options")
        }
        Ok(())
    }

    fn multiple_values(&self) -> bool {
        self.multiple_occurs() || self.args_range().1 > 1
    }

    fn render_source(&self) -> String {
        let mut output = vec![];
        if let Some(short) = &self.short {
            output.push(short.to_string());
        };
        let mut name_suffix = String::new();
        if self.prefixed || self.data.name.ends_with('-') {
            name_suffix.push('-');
        }
        if self.assigned || self.data.name.ends_with(':') {
            name_suffix.push(':');
        }
        output.push(format!(
            "{}{}",
            self.long_prefix,
            self.data.render_source_of_name_value(&name_suffix)
        ));

        if let Some(bind_env) = &self.data.bind_env {
            match bind_env {
                Some(v) => output.push(format!("${v}")),
                None => output.push("$$".into()),
            }
        }

        for raw_notation in &self.raw_notations {
            output.push(format!("<{}>", raw_notation));
        }

        if !self.data.describe.is_empty() {
            output.push(self.data.describe.clone());
        }

        output.join(" ")
    }
}

impl FlagOptionParam {
    pub(crate) fn new(
        mut data: ParamData,
        is_flag: bool,
        short: Option<&str>,
        long_prefix: &str,
        row_notations: &[&str],
    ) -> Self {
        let (mut prefixed, mut assigned) = (false, false);
        if let Some(new_name) = data.name.strip_suffix("::") {
            data.name = format!("{new_name}:");
        } else if let Some(new_name) = data.name.strip_suffix(':') {
            data.name = new_name.to_string();
            assigned = true;
        }
        if let Some(new_name) = data.name.strip_suffix("--") {
            data.name = format!("{new_name}-");
        } else if let Some(new_name) = data.name.strip_suffix('-') {
            data.name = new_name.to_string();
            prefixed = true;
        }
        let name = data.name.clone();
        let id = if long_prefix.starts_with('+') {
            format!("plus_{}", name)
        } else {
            name.clone()
        };
        let raw_notations: Vec<String> = row_notations.iter().map(|v| v.to_string()).collect();
        let mut notations = if is_flag {
            vec![]
        } else if raw_notations.is_empty() {
            vec![to_cobol_case(&name)]
        } else {
            raw_notations.iter().map(|v| to_cobol_case(v)).collect()
        };
        if data.terminated() {
            let last_arg = notations.last_mut().unwrap();
            last_arg.push('~')
        }
        Self {
            is_flag,
            short: short.map(|v| v.to_string()),
            long_prefix: long_prefix.to_string(),
            prefixed,
            assigned,
            data,
            id,
            raw_notations,
            notations,
            inherit: false,
        }
    }

    pub(crate) fn export(&self) -> FlagOptionValue {
        FlagOptionValue {
            id: self.id().to_string(),
            long_name: self.render_long_name(),
            short_name: self.short.clone(),
            describe: self.describe().to_string(),
            is_flag: self.is_flag,
            notations: self.notations.clone(),
            required: self.required(),
            multiple_values: self.multiple_values(),
            multiple_occurs: self.multiple_occurs(),
            args_range: self.args_range(),
            args_delimiter: self.args_delimiter(),
            terminated: self.terminated(),
            prefixed: self.prefixed,
            assigned: self.assigned,
            default: self.data().default.clone(),
            choice: self.data().choice.clone(),
            env: self.bind_env(),
            inherit: self.inherit,
        }
    }

    pub(crate) fn is_flag(&self) -> bool {
        self.is_flag
    }

    pub(crate) fn is_option(&self) -> bool {
        !self.is_flag()
    }

    pub(crate) fn is_assigned(&self) -> bool {
        self.assigned
    }

    pub(crate) fn is_prefixed(&self) -> bool {
        self.prefixed
    }

    pub(crate) fn short(&self) -> &Option<String> {
        &self.short
    }

    pub(crate) fn long_prefix(&self) -> &str {
        &self.long_prefix
    }

    pub(crate) fn notations(&self) -> &[String] {
        &self.notations
    }

    pub(crate) fn args_range(&self) -> (usize, usize) {
        let len = self.notations.len();
        if self.terminated()
            || self
                .notation_modifer()
                .map(|v| matches!(v, '*' | '+'))
                .unwrap_or_default()
        {
            let min = if self.notation_modifer() == Some('*') {
                len - 1
            } else {
                len
            };
            (min, MAX_ARGS)
        } else if self.notation_modifer() == Some('?') {
            (len - 1, len)
        } else {
            (len, len)
        }
    }

    pub(crate) fn notation_modifer(&self) -> Option<char> {
        self.notations
            .last()
            .and_then(|name| ['*', '+', '?'].into_iter().find(|v| name.ends_with(*v)))
    }

    pub(crate) fn render_long_name(&self) -> String {
        format!("{}{}", self.long_prefix, self.data.name)
    }

    pub(crate) fn render_first_notation(&self) -> String {
        format!("<{}>", self.notations[0])
    }

    pub(crate) fn render_name_notations(&self) -> String {
        let mut output = self.render_long_name();
        if !self.is_flag() {
            let ch = if self.assigned { '=' } else { ' ' };
            output.push(ch);
            output.push_str(&self.render_notations());
        }
        output
    }

    pub(crate) fn render_notations(&self) -> String {
        let mut list = vec![];
        if self.notations.len() == 1 {
            let name: &String = &self.notations[0];
            let value = match (self.required(), self.multiple_occurs()) {
                (true, true) => format!("<{name}>..."),
                (false, true) => format!("[{name}]..."),
                (_, false) => format!("<{name}>"),
            };
            list.push(value);
        } else {
            let values = self
                .notations
                .iter()
                .map(|v| format!("<{v}>"))
                .collect::<Vec<String>>();
            list.extend(values);
        }
        list.join(" ")
    }

    pub(crate) fn render_body(&self) -> String {
        let mut output = String::new();
        if self.short.is_none() && self.long_prefix.len() == 1 && self.data.name.len() == 1 {
            output.push_str(&self.render_long_name());
        } else {
            if let Some(short) = &self.short {
                output.push_str(&format!("{short}, "))
            } else {
                output.push_str("    ")
            };
            output.push_str(&format!("{:>2}", self.long_prefix));
            output.push_str(&self.data.name);
        }

        if self.is_flag() {
            if self.multiple_occurs() {
                output.push_str("...")
            }
        } else {
            let ch = if self.assigned { '=' } else { ' ' };
            output.push(ch);
            output.push_str(&self.render_notations());
        }
        output
    }

    pub(crate) fn to_argc_value(&self, args: &[(&str, &[&str])]) -> Option<ArgcValue> {
        let id = self.id().to_string();
        if self.prefixed {
            let mut map: IndexMap<String, Vec<String>> = IndexMap::new();
            for (arg, value) in args {
                if let Some(arg_suffix) = self
                    .list_names()
                    .into_iter()
                    .find_map(|v| arg.strip_prefix(&v))
                {
                    let key = match arg_suffix.split_once('=') {
                        Some((arg_suffix, _)) => arg_suffix,
                        None => arg_suffix,
                    };
                    if let Some(values) = map.get_mut(key) {
                        values.extend(value.iter().map(|v| v.to_string()));
                    } else {
                        map.insert(
                            key.to_string(),
                            value.iter().map(|v| v.to_string()).collect(),
                        );
                    }
                }
            }
            if map.is_empty() {
                None
            } else {
                Some(ArgcValue::Map(id, map))
            }
        } else {
            let values: Vec<&[&str]> = args.iter().map(|(_, value)| *value).collect();
            if self.is_flag {
                if values.is_empty() {
                    None
                } else {
                    Some(ArgcValue::Single(id, values.len().to_string()))
                }
            } else {
                if values.is_empty() {
                    match &self.data.default {
                        Some(DefaultValue::Value(value)) => {
                            return Some(ArgcValue::Single(id, value.clone()));
                        }
                        Some(DefaultValue::Fn(f)) => {
                            return Some(ArgcValue::SingleFn(id, f.clone()));
                        }
                        None => return None,
                    }
                }
                if self.multiple_values() {
                    let values: Vec<String> = values
                        .iter()
                        .flat_map(|v| v.iter().map(|v| v.to_string()))
                        .collect();
                    Some(ArgcValue::Multiple(id, values))
                } else if self.notations.len() > 1 {
                    Some(ArgcValue::Multiple(
                        id,
                        values[0].iter().map(|v| v.to_string()).collect(),
                    ))
                } else {
                    Some(ArgcValue::Single(id, must_get_first(values[0])))
                }
            }
        }
    }

    pub(crate) fn set_inherit(&mut self) {
        self.inherit = true;
    }

    pub(crate) fn match_prefix<'a>(&self, arg: &'a str) -> Option<&'a str> {
        if self.prefixed {
            self.list_names().iter().find_map(|v| {
                if arg.starts_with(v) {
                    Some(&arg[..v.len()])
                } else {
                    None
                }
            })
        } else {
            None
        }
    }

    pub(crate) fn is_match(&self, name: &str) -> bool {
        self.id() == name
            || self.list_names().iter().any(|v| {
                if self.prefixed {
                    name.starts_with(v)
                } else {
                    v == name
                }
            })
    }

    pub(crate) fn list_names(&self) -> Vec<String> {
        let mut output = vec![];
        output.push(self.render_long_name());
        if let Some(short) = &self.short {
            output.push(short.clone());
        }
        output
    }
}

#[derive(Debug, Serialize)]
pub struct FlagOptionValue {
    pub id: String,
    pub long_name: String,
    pub short_name: Option<String>,
    pub describe: String,
    pub is_flag: bool,
    pub notations: Vec<String>,
    pub required: bool,
    pub multiple_values: bool,
    pub multiple_occurs: bool,
    pub args_range: (usize, usize),
    pub args_delimiter: Option<char>,
    pub terminated: bool,
    pub prefixed: bool,
    pub assigned: bool,
    pub default: Option<DefaultValue>,
    pub choice: Option<ChoiceValue>,
    pub env: Option<String>,
    pub inherit: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct PositionalParam {
    data: ParamData,
    raw_notation: Option<String>,
    notation: String,
}

impl Param for PositionalParam {
    fn data(&self) -> &ParamData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut ParamData {
        &mut self.data
    }

    fn id(&self) -> &str {
        &self.data().name
    }

    fn var_name(&self) -> String {
        argc_var_name(self.id())
    }

    fn tag_name(&self) -> &str {
        "@arg"
    }

    fn guard(&self) -> Result<()> {
        Ok(())
    }

    fn multiple_values(&self) -> bool {
        self.multiple_occurs() || self.terminated()
    }

    fn render_source(&self) -> String {
        let mut output = vec![self.data.render_source_of_name_value("")];

        if let Some(bind_env) = &self.data.bind_env {
            match bind_env {
                Some(v) => output.push(format!("${v}")),
                None => output.push("$$".into()),
            }
        }
        if let Some(raw_notation) = self.raw_notation.as_ref() {
            output.push(format!("<{}>", raw_notation));
        }
        if !self.data.describe.is_empty() {
            output.push(self.data.describe.clone());
        }
        output.join(" ")
    }
}

impl PositionalParam {
    pub(crate) fn new(data: ParamData, raw_notation: Option<&str>) -> Self {
        let name = data.name.clone();
        PositionalParam {
            data,
            raw_notation: raw_notation.map(|v| v.to_string()),
            notation: raw_notation
                .or(Some(&name))
                .map(to_cobol_case)
                .unwrap_or_default(),
        }
    }

    pub(crate) fn export(&self) -> PositionalValue {
        PositionalValue {
            id: self.id().to_string(),
            describe: self.describe().to_string(),
            notation: self.notation.clone(),
            required: self.required(),
            multiple_values: self.multiple_values(),
            multiple_occurs: self.multiple_occurs(),
            args_delimiter: self.args_delimiter(),
            terminated: self.terminated(),
            default: self.data().default.clone(),
            choice: self.data().choice.clone(),
            env: self.bind_env(),
        }
    }

    pub(crate) fn notation(&self) -> &str {
        &self.notation
    }

    pub(crate) fn render_notation(&self) -> String {
        let name: &String = &self.notation;
        match (self.required(), self.multiple_values()) {
            (true, true) => format!("<{name}>..."),
            (true, false) => format!("<{name}>"),
            (false, true) => format!("[{name}]..."),
            (false, false) => format!("[{name}]"),
        }
    }

    pub(crate) fn to_argc_value(&self, values: &[&str]) -> Option<ArgcValue> {
        let id = self.id().to_string();
        if values.is_empty() {
            match &self.data.default {
                Some(DefaultValue::Value(value)) => {
                    return Some(ArgcValue::PositionalSingle(id, value.clone()));
                }
                Some(DefaultValue::Fn(f)) => {
                    return Some(ArgcValue::PositionalSingleFn(id, f.clone()));
                }
                None => return None,
            }
        }
        if self.multiple_values() {
            let values: Vec<String> = values.iter().map(|v| v.to_string()).collect();
            Some(ArgcValue::PositionalMultiple(id, values))
        } else {
            Some(ArgcValue::PositionalSingle(id, must_get_first(values)))
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PositionalValue {
    pub id: String,
    pub describe: String,
    pub notation: String,
    pub required: bool,
    pub multiple_values: bool,
    pub multiple_occurs: bool,
    pub args_delimiter: Option<char>,
    pub terminated: bool,
    pub default: Option<DefaultValue>,
    pub choice: Option<ChoiceValue>,
    pub env: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct EnvParam {
    data: ParamData,
    describe: String,
    inherit: bool,
}

impl Param for EnvParam {
    fn data(&self) -> &ParamData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut ParamData {
        &mut self.data
    }

    fn id(&self) -> &str {
        &self.data().name
    }

    fn var_name(&self) -> String {
        self.id().to_string()
    }

    fn describe(&self) -> &str {
        &self.describe
    }

    fn tag_name(&self) -> &str {
        "@env"
    }

    fn guard(&self) -> Result<()> {
        if !matches!(self.data().modifer, Modifier::Optional | Modifier::Required) {
            bail!("can only be a single value")
        }
        if self.data.bind_env.is_some() {
            bail!("cannot bind another env")
        }
        Ok(())
    }

    fn multiple_values(&self) -> bool {
        false
    }

    fn render_source(&self) -> String {
        let mut output = vec![self.data.render_source_of_name_value("")];
        if let Some(bind_env) = &self.data.bind_env {
            match bind_env {
                Some(v) => output.push(format!("${v}")),
                None => output.push("$$".into()),
            }
        }
        if !self.data.describe.is_empty() {
            output.push(self.data.describe.clone());
        }
        output.join(" ")
    }
}

impl EnvParam {
    pub(crate) fn new(data: ParamData, describe: &str) -> Self {
        Self {
            describe: describe.to_string(),
            data,
            inherit: false,
        }
    }

    pub(crate) fn export(&self) -> EnvValue {
        EnvValue {
            id: self.id().to_string(),
            describe: self.describe.clone(),
            required: self.required(),
            default: self.data().default.clone(),
            choice: self.data().choice.clone(),
            inherit: self.inherit,
        }
    }

    pub(crate) fn render_body(&self) -> String {
        let marker = if self.required() { "*" } else { "" };
        format!("{}{}", self.id(), marker)
    }

    pub(crate) fn get_env_value(&self) -> Option<ArgcValue> {
        let id = self.id().to_string();
        let default = self.data.default.clone()?;
        let value = match default {
            DefaultValue::Value(value) => ArgcValue::Env(id, value),
            DefaultValue::Fn(value) => ArgcValue::EnvFn(id, value),
        };
        Some(value)
    }

    pub(crate) fn set_inherit(&mut self) {
        self.inherit = true;
    }
}

#[derive(Debug, Serialize)]
pub struct EnvValue {
    pub id: String,
    pub describe: String,
    pub required: bool,
    pub default: Option<DefaultValue>,
    pub choice: Option<ChoiceValue>,
    pub inherit: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct ParamData {
    pub(crate) name: String,
    pub(crate) describe: String,
    pub(crate) choice: Option<ChoiceValue>,
    pub(crate) default: Option<DefaultValue>,
    pub(crate) modifer: Modifier,
    pub(crate) bind_env: Option<Option<String>>,
}

impl ParamData {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            describe: String::new(),
            choice: None,
            default: None,
            modifer: Modifier::Optional,
            bind_env: None,
        }
    }

    pub(crate) fn required(&self) -> bool {
        self.modifer.required() && self.default.is_none()
    }

    pub(crate) fn multiple_occurs(&self) -> bool {
        self.modifer.multiple_occurs()
    }

    pub(crate) fn args_delimiter(&self) -> Option<char> {
        match &self.modifer {
            Modifier::DelimiterRequired(c) | Modifier::DelimiterOptional(c) => Some(*c),
            _ => None,
        }
    }

    pub(crate) fn terminated(&self) -> bool {
        matches!(self.modifer, Modifier::Terminated)
    }

    pub(crate) fn choice_fn(&self) -> Option<(&String, &bool)> {
        match &self.choice {
            Some(ChoiceValue::Fn(f, v)) => Some((f, v)),
            _ => None,
        }
    }

    pub(crate) fn choice_values(&self) -> Option<&Vec<String>> {
        match &self.choice {
            Some(ChoiceValue::Values(v)) => Some(v),
            _ => None,
        }
    }

    pub(crate) fn default_fn(&self) -> Option<&String> {
        match &self.default {
            Some(DefaultValue::Fn(f)) => Some(f),
            _ => None,
        }
    }

    pub(crate) fn default_value(&self) -> Option<&String> {
        match &self.default {
            Some(DefaultValue::Value(v)) => Some(v),
            _ => None,
        }
    }

    pub(crate) fn normalize_bind_env(&self, id: &str) -> Option<String> {
        let bind_env = match self.bind_env.as_ref()? {
            Some(v) => v.clone(),
            None => sanitize_var_name(id).to_uppercase(),
        };
        Some(bind_env)
    }

    pub(crate) fn render_source_of_name_value(&self, name_suffix: &str) -> String {
        let mut output = format!("{}{name_suffix}", self.name);
        output.push_str(&self.modifer.render());
        match (&self.choice, &self.default) {
            (Some(ChoiceValue::Values(values)), None) => {
                output.push_str(&format!("[{}]", Self::render_choice_values(values)));
            }
            (Some(ChoiceValue::Values(values)), Some(DefaultValue::Value(_))) => {
                output.push_str(&format!("[={}]", Self::render_choice_values(values)));
            }
            (Some(ChoiceValue::Fn(f, validate)), _) => {
                let prefix = if *validate { "" } else { "?" };
                output.push_str(&format!("[{prefix}`{f}`]"));
            }
            (None, Some(DefaultValue::Value(value))) => {
                output.push_str(&format!("={}", Self::render_default_value(value)));
            }
            (None, Some(DefaultValue::Fn(f))) => {
                output.push_str(&format!("=`{f}`"));
            }
            _ => {}
        }
        output
    }

    pub(crate) fn render_describe(&self, describe: &str, id: &str) -> String {
        let mut output = describe.to_string();
        let multiline = output.contains('\n');
        let sep = if multiline { '\n' } else { ' ' };
        if let Some(DefaultValue::Value(value)) = &self.default {
            if !output.is_empty() {
                output.push(sep)
            }
            output.push_str(&format!("[default: {}]", escape_shell_words(value)));
        }
        if let Some(ChoiceValue::Values(values)) = &self.choice {
            if !output.is_empty() {
                output.push(sep)
            }
            let values: Vec<String> = values.iter().map(|v| escape_shell_words(v)).collect();
            output.push_str(&format!("[possible values: {}]", values.join(", ")));
        }
        if let Some(bind_env) = self.normalize_bind_env(id) {
            if !output.is_empty() {
                output.push(sep)
            }
            output.push_str(&format!("[env: {bind_env}]"));
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
pub(crate) enum Modifier {
    Optional,
    Required,
    MultipleOptional,
    MultipleRequired,
    DelimiterOptional(char),
    DelimiterRequired(char),
    Terminated,
}

impl Modifier {
    pub(crate) fn multiple_occurs(&self) -> bool {
        match self {
            Self::Optional => false,
            Self::Required => false,
            Self::MultipleOptional => true,
            Self::MultipleRequired => true,
            Self::DelimiterOptional(_) => true,
            Self::DelimiterRequired(_) => true,
            Self::Terminated => false,
        }
    }

    pub(crate) fn required(&self) -> bool {
        match self {
            Self::Optional => false,
            Self::Required => true,
            Self::MultipleOptional => false,
            Self::MultipleRequired => true,
            Self::DelimiterOptional(_) => false,
            Self::DelimiterRequired(_) => true,
            Self::Terminated => false,
        }
    }

    pub(crate) fn render(&self) -> String {
        match self {
            Self::Optional => "".into(),
            Self::Required => "!".into(),
            Self::MultipleOptional => "*".into(),
            Self::MultipleRequired => "+".into(),
            Self::DelimiterOptional(c) => format!("*{c}"),
            Self::DelimiterRequired(c) => format!("+{c}"),
            Self::Terminated => "~".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ChoiceValue {
    Values(Vec<String>),
    Fn(String, bool),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum DefaultValue {
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
