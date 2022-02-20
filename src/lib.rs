#[derive(Debug)]
pub struct App {
    pub title: Option<String>,
    pub version: Option<String>,
    pub options: Vec<AppOption>,
    pub commands: Vec<AppOption>,
}

#[derive(Debug)]
pub struct AppOption {
    pub name: String,
    pub description: Option<String>,
    pub kind: AppOptionType,
    pub array: bool,
    pub choices: Vec<String>,
    pub short: String,
    pub options: Vec<AppOption>,
    pub params: Vec<AppOption>,
}

impl AppOption {
    pub fn parse() -> Self {
        todo!()
    }
}


#[derive(Debug)]
pub enum AppOptionType {
    String,
    Boolean,
    Number,
}

pub struct Settings {}
