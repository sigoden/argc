use crate::{command::Command, param::Param, utils::META_MAN_SECTION};

use anyhow::Result;
use roff::{bold, italic, roman, Inline, Roff};

pub fn mangen(source: &str, root_name: &str) -> Result<Vec<(String, String)>> {
    let mut output = vec![];
    let root_cmd = Command::new(source, root_name)?;
    let section = root_cmd.get_metadata(META_MAN_SECTION).unwrap_or("1");
    manpage_impl(&mut output, &root_cmd, section);
    Ok(output)
}

fn manpage_impl(output: &mut Vec<(String, String)>, cmd: &Command, section: &str) {
    let filename = format!("{}.{}", cmd.full_name(), section);
    let page = render_manpage(cmd, section);
    output.push((filename, page));
    for subcmd in &cmd.subcommands {
        manpage_impl(output, subcmd, section);
    }
}

fn render_manpage(cmd: &Command, section: &str) -> String {
    let mut roff = Roff::default();
    render_title(&mut roff, cmd, section);
    render_name_section(&mut roff, cmd);
    render_synopsis_section(&mut roff, cmd);
    render_description_section(&mut roff, cmd);
    render_options_section(&mut roff, cmd);
    render_subcommands_section(&mut roff, cmd, section);
    render_envs_section(&mut roff, cmd);
    render_version_section(&mut roff, cmd);
    render_author_section(&mut roff, cmd);
    roff.to_roff()
}

fn render_title(roff: &mut Roff, cmd: &Command, section: &str) {
    let title = cmd.full_name().to_uppercase();
    let source = cmd.render_version();
    let title_args = [title.as_str(), section, "", source.as_str(), ""];
    roff.control("TH", title_args);
}

fn render_name_section(roff: &mut Roff, cmd: &Command) {
    roff.control("SH", ["NAME"]);
    let name = cmd.full_name();
    let text = if !cmd.describe.is_empty() {
        format!("{} - {}", name, cmd.describe_oneline())
    } else {
        name
    };
    roff.text([roman(text)]);
}

fn render_synopsis_section(roff: &mut Roff, cmd: &Command) {
    roff.control("SH", ["SYNOPSIS"]);
    let name = cmd.cmd_paths().join(" ");
    let mut line = vec![bold(name)];
    if !cmd.flag_option_params.is_empty() {
        line.push(roman(" [OPTIONS]"))
    }
    if !cmd.subcommands.is_empty() {
        line.push(roman(" <COMMAND>"))
    } else {
        for param in &cmd.positional_params {
            line.push(roman(format!(" {}", param.render_notation())))
        }
    }
    roff.text(line);
}

fn render_description_section(roff: &mut Roff, cmd: &Command) {
    if !cmd.describe.contains('\n') {
        return;
    }
    roff.control("SH", ["DESCRIPTION"]);
    let mut body = vec![];
    render_describe(&mut body, &cmd.describe);
    roff.text(body);
}

fn render_options_section(roff: &mut Roff, cmd: &Command) {
    if cmd.flag_option_params.is_empty() && cmd.positional_params.is_empty() {
        return;
    }
    roff.control("SH", ["OPTIONS"]);
    for param in cmd.all_flag_options() {
        let mut header = vec![];
        if let Some(short) = param.short() {
            header.push(bold(short));
            header.push(roman(", "));
        }
        header.push(bold(param.long_name()));
        let notations = param.notations();
        if notations.len() == 1 {
            header.push(roman("="));
            let notation = &notations[0];
            let parts = match (param.required(), param.multiple_occurs()) {
                (true, true) => vec![roman("<"), italic(notation), roman(">...")],
                (false, true) => vec![roman("["), italic(notation), roman("]...")],
                (true, false) => vec![roman("<"), italic(notation), roman(">")],
                (false, false) => vec![italic(notation)],
            };
            header.extend(parts);
        } else {
            for notation in notations {
                header.push(roman(" "));
                header.push(italic(notation));
            }
        }
        if let Some(value) = param.default_value() {
            header.push(roman(format!(" [default: {value}]")));
        }
        let mut body = vec![];
        let mut has_help_written = false;
        if !param.describe().is_empty() {
            has_help_written = true;
            render_describe(&mut body, param.describe());
        }
        roff.control("TP", []);
        roff.text(header);
        roff.text(body);
        render_choices(roff, param, has_help_written);
    }

    for param in &cmd.positional_params {
        let notation = param.notation();
        let mut header = match (param.required(), param.multiple_values()) {
            (true, true) => vec![roman("<"), italic(notation), roman(">...")],
            (true, false) => vec![roman("<"), italic(notation), roman(">")],
            (false, true) => vec![roman("["), italic(notation), roman("]...")],
            (false, false) => vec![roman("["), italic(notation), roman("]")],
        };
        if let Some(value) = param.default_value() {
            header.push(roman(format!(" [default: {value}]")));
        }
        let mut body = vec![];
        let mut has_help_written = false;
        if !param.describe().is_empty() {
            has_help_written = true;
            render_describe(&mut body, param.describe());
        }
        roff.control("TP", []);
        roff.text(header);
        roff.text(body);
        render_choices(roff, param, has_help_written);
    }
}

fn render_subcommands_section(roff: &mut Roff, cmd: &Command, section: &str) {
    if cmd.subcommands.is_empty() {
        return;
    }
    roff.control("SH", ["SUBCOMMANDS"]);
    for subcmd in &cmd.subcommands {
        roff.control("TP", []);
        let name = subcmd.full_name();
        roff.text([roman(format!("{}({})", name, section))]);
        for line in subcmd.describe.lines() {
            roff.text([roman(line)]);
        }
    }
}

fn render_envs_section(roff: &mut Roff, cmd: &Command) {
    if cmd.env_params.is_empty() {
        return;
    }
    roff.control("SH", ["ENVIRONMENT VARIABLES:"]);
    for param in &cmd.env_params {
        let mut header = vec![];
        header.push(italic(param.var_name()));
        if param.required() {
            header.push(roman("*"));
        }
        if let Some(value) = param.default_value() {
            header.push(roman(format!(" [default: {value}]")));
        }
        let mut body = vec![];
        let mut has_help_written = false;
        if !param.describe().is_empty() {
            has_help_written = true;
            render_describe(&mut body, param.describe());
        }
        roff.control("TP", []);
        roff.text(header);
        roff.text(body);
        render_choices(roff, param, has_help_written);
    }
}

fn render_version_section(roff: &mut Roff, cmd: &Command) {
    if let Some(version) = &cmd.version {
        roff.control("SH", ["VERSION"]);
        roff.text([roman(version)]);
    }
}

fn render_author_section(roff: &mut Roff, cmd: &Command) {
    if let Some(author) = &cmd.author {
        roff.control("SH", ["AUTHORS"]);
        roff.text([roman(author)]);
    }
}

fn render_describe(body: &mut Vec<Inline>, describe: &str) {
    for line in describe.split('\n') {
        body.push(Inline::LineBreak);
        body.push(roman(line));
    }
}

fn render_choices<T: Param>(roff: &mut Roff, param: &T, has_help_written: bool) {
    if let Some(values) = param.choice_values() {
        if has_help_written {
            roff.text([Inline::LineBreak]);
        }
        let text: Vec<Inline> = vec![
            Inline::LineBreak,
            roman("["),
            italic("possible values: "),
            roman(values.join(", ")),
            roman("]"),
        ];
        roff.text(text);
    }
}
