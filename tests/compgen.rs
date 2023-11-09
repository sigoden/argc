use crate::*;

#[test]
fn multiple() {
    let script = r###"
# @flag   -f --fc*
# @option -a --oa* <DIR>
# @option -b --ob <CMD> <DIR+>
# @option -c --oc <DIR+>
# @option -d --od <DIR> <FILE>
# @option -e --oe* <DIR+>
# @arg var* <FILE>
"###;

    snapshot_compgen!(
        script,
        [
            vec!["prog", ""],
            vec!["prog", "-"],
            vec!["prog", "--"],
            vec!["prog", "--", ""],
            vec!["prog", "-f", ""],
            vec!["prog", "--fc", ""],
            vec!["prog", "-a", ""],
            vec!["prog", "-a", "d1"],
            vec!["prog", "-a", "d1", ""],
            vec!["prog", "-b", ""],
            vec!["prog", "-b", "d1"],
            vec!["prog", "-b", "d1", ""],
            vec!["prog", "-b", "d1", "d2"],
            vec!["prog", "-b", "d1", "d2", ""],
            vec!["prog", "-c", ""],
            vec!["prog", "-c", "d1"],
            vec!["prog", "-c", "d1", ""],
            vec!["prog", "-d", "d1"],
            vec!["prog", "-d", "d1", ""],
            vec!["prog", "-d", "d1", "d2"],
            vec!["prog", "-d", "d1", "d2", ""],
            vec!["prog", "-e", "d1", ""],
            vec!["prog", "-a", "d1", "-c", "d2", "-"],
            vec!["prog", "v1"],
            vec!["prog", "v1", ""],
            vec!["prog", "v1", "v2"],
            vec!["prog", "v1", "v2", ""],
        ]
    );
}

#[test]
fn shorts() {
    const SCRIPT: &str = r###"
# @meta combine-shorts
# @flag   -a
# @flag   -b --fb
# @flag   -f --fc*
# @flag      -sa
# @option -e <FILE>
# @option -p --oa*
"###;

    snapshot_compgen!(
        SCRIPT,
        [
            vec!["prog", ""],
            vec!["prog", "-"],
            vec!["prog", "--"],
            vec!["prog", "-a"],
            vec!["prog", "-a", ""],
            vec!["prog", "-af"],
            vec!["prog", "-af", ""],
            vec!["prog", "-ae"],
            vec!["prog", "-ae", ""],
            vec!["prog", "-abe"],
            vec!["prog", "-abe", ""],
            vec!["prog", "-s"],
            vec!["prog", "-sa"],
            vec!["prog", "-sa", ""],
        ]
    );
}

#[test]
fn symbol() {
    let script = r###"
# @meta symbol +toolchain[`_choice_fn`]
# @meta symbol @file
# @option --oa

_choice_fn() {
    echo stable
    echo beta
    echo nightly
}
"###;
    snapshot_compgen!(
        script,
        [vec!["prog", "+"], vec!["prog", "@"], vec!["prog", "+s"]]
    );
}

#[test]
fn subcmds() {
    const SCRIPT: &str = r###"
# @arg file
# @cmd
cmda() { :; }
# @cmd
cmdb() { :; }
# @cmd
cmdbc() { :; }
"###;

    snapshot_compgen!(
        SCRIPT,
        [
            vec!["prog", ""],
            vec!["prog", "c"],
            vec!["prog", "cmda"],
            vec!["prog", "cmda", ""],
            vec!["prog", "help", ""],
            vec!["prog", "help", "c"],
            vec!["prog", "help", "cmda", ""],
            vec!["prog", "help", "cmdb"],
        ]
    );
}

#[test]
fn nested_subcmds() {
    const SCRIPT: &str = r###"
# @arg file
# @cmd
cmd() { :; }
# @cmd
cmd::suba() { :; }
# @cmd
cmd::subb() { :; }
"###;

    snapshot_compgen!(
        SCRIPT,
        [
            vec!["prog", ""],
            vec!["prog", "cmd"],
            vec!["prog", "cmd", ""],
            vec!["prog", "cmd", "s"],
            vec!["prog", "cmd", "suba"],
            vec!["prog", "cmd", "suba", ""],
            vec!["prog", "cmd", "help", ""],
            vec!["prog", "cmd", "help", "s"],
        ]
    );
}

#[test]
fn flag_cmds() {
    const SCRIPT: &str = r###"
# @meta combine-shorts
# @option -G

# @cmd Run --foo
# @alias -F
# @flag --fa
--foo() {
    :;
}

# @cmd Run bar
# @alias -B
# @flag -C
# @flag -D
# @flag --fa
bar() {
    :;
}
"###;

    snapshot_compgen!(
        SCRIPT,
        [
            vec!["prog", ""],
            vec!["prog", "-"],
            vec!["prog", "-B"],
            vec!["prog", "-B", "-"],
            vec!["prog", "-G"],
        ]
    );
}

#[test]
fn positionals() {
    let script = r###"
# @cmd
# @arg dir
# @arg file*
cmda() { :; }

# @cmd
# @arg dir1
# @arg dir2
# @arg dir3
cmdb() { :; }

# @cmd
# @arg dir*
# @arg file*
cmdc() { :; }
"###;

    snapshot_compgen!(
        script,
        [
            vec!["prog", "cmda", ""],
            vec!["prog", "cmda", "v1"],
            vec!["prog", "cmda", "v1", ""],
            vec!["prog", "cmda", "v1", "v2"],
            vec!["prog", "cmda", "v1", "v2", ""],
            vec!["prog", "cmdb", ""],
            vec!["prog", "cmdb", "v1"],
            vec!["prog", "cmdb", "v1", ""],
            vec!["prog", "cmdb", "v1", "v2"],
            vec!["prog", "cmdb", "v1", "v2", ""],
            vec!["prog", "cmdb", "v1", "v2", "v3"],
            vec!["prog", "cmdb", "v1", "v2", "v3", ""],
            vec!["prog", "cmdc", ""],
            vec!["prog", "cmdc", "v1"],
            vec!["prog", "cmdc", "v1", ""],
            vec!["prog", "cmdc", "v1", "v2"],
            vec!["prog", "cmdc", "v1", "v2", ""],
        ]
    );
}

#[test]
fn choice() {
    let script = r#"
# @option --oa[`_choice_fn`]
# @option --ob[x|y|z]
# @option --oc*,[`_choice_fn`]
# @arg v1[x|y|z]
# @arg v2[`_choice_fn`]
_choice_fn() {
	echo -e "abc\ndef\nghi"
}
"#;

    snapshot_compgen!(
        script,
        [
            vec!["prog", "--oa", ""],
            vec!["prog", "--oa="],
            vec!["prog", "--oa=a"],
            vec!["prog", "--oa", "=a"],
            vec!["prog", "--ob", ""],
            vec!["prog", ""],
            vec!["prog", "v1", ""],
            vec!["prog", "'--oa="],
            vec!["prog", "'--oa=a"],
            vec!["prog", "\"--oa="],
            vec!["prog", "\"--oa=a"],
            vec!["prog", "--oc", ""],
            vec!["prog", "--oc", "abc,"],
        ]
    );
}

#[test]
fn choice_multi() {
    let script = r#"
# @option --oa*[`_choice_fn`]
_choice_fn() {
	echo -e "abc\ndef\nghi"
}
"#;

    snapshot_compgen!(script, [vec!["prog", "--oa", ""], vec!["prog", "--oa="],]);
}

#[test]
fn choice_check_vars() {
    let script = r###"
# @arg foo[`_choice_fn`]
# @arg bar[`_choice_fn`]
_choice_fn() {
    ( set -o posix ; set ) | grep argc_
}
"###;

    snapshot_compgen!(
        script,
        [
            vec!["prog", "argc"],
            vec!["prog", "argc", ""],
            vec!["prog", "argc", "argc"],
        ]
    );
}

#[test]
fn choice_slash() {
    let script = r###"
# @cmd
# @arg foo
# @arg bar[`_choice_fn`]
cmd() {
    echo $1
}
_choice_fn() {
    echo $1
}
"###;
    snapshot_compgen!(script, [vec!["prog", "cmd", "a\\b", ""],]);
}

#[test]
fn multiline_doc() {
    let script = r###"
# @cmd cmd-line1
# cmd-line2
# @option --foo option-line1
# option-line2
# @arg bar bar-line1
# bar-line2
cmda() { :; }

# @cmd line
cmdb() { :; }
"###;
    snapshot_compgen!(script, [vec!["prog", ""], vec!["prog", "cmda", ""],]);
}

#[test]
fn no_param() {
    let script = r###"
# @cmd
cmd() { :; }
"###;
    snapshot_compgen!(script, [vec!["prog", "cmd", ""],]);
}

#[test]
fn special_arg_name() {
    let script = r###"
# @cmd
# @arg arg
cmda() { :; }

# @cmd
# @arg any
cmdb() { :; }
"###;
    snapshot_compgen!(
        script,
        [vec!["prog", "cmda", ""], vec!["prog", "cmdb", ""],]
    );
}

#[test]
fn one_combine_shorts() {
    let script = r###"
# @meta combine-shorts
# @flag -a
# @flag -b
"###;
    snapshot_compgen!(script, [vec!["prog", "-a"],]);
}

#[test]
fn no_comp_subcmds() {
    let script = r###"
# @cmd
cmda() { :; }

# @cmd
cmdb() { :; }
"###;
    snapshot_compgen!(
        script,
        [
            vec!["prog", ""],
            vec!["prog", "cmdx", ""],
            vec!["prog", "cmdx", "cmd"]
        ]
    );
}

#[test]
fn just_match() {
    let script = r###"
# @option --oa
# @option --oa-file
"###;
    snapshot_compgen!(script, [vec!["prog", "--oa"]]);
}

#[test]
fn no_flags_options() {
    let script = r###"
# @cmd
# @flag --fa
# @option --oa  <file>
no_arg() { :; }

# @cmd
# @flag --fa
# @option --oa  <file>
# @arg file
arg() { :; }
"###;

    snapshot_compgen!(
        script,
        [vec!["prog", "no_arg", ""], vec!["prog", "arg", ""]]
    );
}

#[test]
fn no_flags_options2() {
    const SCRIPT: &str = r###"
# @option --oa
# @option --ob

# @cmd
cmda() { :; }
# @cmd
cmdb() { :; }
"###;

    snapshot_compgen!(SCRIPT, [vec!["prog", ""],]);
}

#[test]
fn one_subcmd_with_options() {
    const SCRIPT: &str = r###"
# @option --oa
# @option --ob

# @cmd
cmda() { :; }
"###;

    snapshot_compgen!(SCRIPT, [vec!["prog", ""],]);
}

#[test]
fn dashes_at() {
    let script = r#"
# @arg val*[`_choice_fn`]
_choice_fn() {
    if [[ -z "$argc__dashes" ]]; then
        echo -e "abc\ndef\nghi"
    else
        echo -e "v1\nv2"
    fi
}
"#;

    snapshot_compgen!(
        script,
        [vec!["prog", "abc", ""], vec!["prog", "abc", "--", ""],]
    );
}

#[test]
fn inherit_flag_options() {
    let script = r###"
# @meta inherit-flag-options
# @flag --oa
# @option --ob[a|b]  desc 1

# @cmd
cmda() {
    :;
}

# @cmd
# @option --ob[x|y]  desc 2
cmdb() {
    :;
}
"###;
    snapshot_compgen!(
        script,
        [vec!["prog", "cmda", "--"], vec!["prog", "cmdb", "--"]]
    );
}

// ------------ compgen shell -----------

#[test]
fn no_space() {
    let script = r#"
# @option --oa*[`_choice_fn`]
_choice_fn() {
	echo -e "abc"
	echo -e "def\0"
	echo -e "ghk\thello world"
}
"#;

    snapshot_compgen_shells!(script, ["prog", "--oa", ""]);
}

#[test]
fn suffix() {
    let script = r#"
# @option --oa*[`_choice_fn`]
_choice_fn() {
    echo -e "__argc_suffix==\0"
	echo -e "abc"
	echo -e "def"
	echo -e "ghk"
}
"#;

    snapshot_compgen_shells!(script, ["prog", "--oa", ""]);
}

#[test]
fn value() {
    let script = r#"
# @option --oa[`_choice_fn`]
_choice_fn() {
    echo "abc def"
    echo "abc xyz"
    echo "abc:def"
    echo "abc:xyz"
    echo "abc>def"
    echo "abc>xyz"
}
"#;

    snapshot_compgen_shells!(script, ["prog", "--oa", "abc"]);
}

#[test]
fn value_display() {
    let script = r#"
# @option --oa*[`_choice_fn`]
_choice_fn() {
	echo "abc:def:xyz"
	echo "abc:def:tsr"
	echo "abc:ijk:abc"
	echo "abc:ijk:xyz"
}
"#;

    snapshot_compgen_shells!(script, ["prog", "--oa="]);
}

#[test]
fn multi_parts() {
    let script = r#"
# @option --oa*[`_choice_fn`]
_choice_fn() {
    echo __argc_prefix=A/
    echo __argc_filter=''
	echo B
	echo -e "B/\0"
}
"#;

    snapshot_compgen_shells!(script, ["prog", "--oa", "A/"]);
}

#[test]
fn multi_parts2() {
    let script = r###"
# @option --oa*[`_choice_fn`]
_choice_fn() {
    echo __argc_prefix=A/B/
    echo __argc_filter=''
	echo
	echo C
	echo D
}
"###;

    snapshot_compgen_shells!(script, ["prog", "--oa", "A/B/"]);
}

#[test]
fn assing_option_value() {
    let script = r"
# @option --oa[`_choice_fn`]
# @arg val[`_choice_fn`]
_choice_fn() {
    echo __argc_filter=
    ( set -o posix ; set ) | grep 'argc_\|ARGC_COMPGEN\|ARGC_CWORD\|ARGC_LAST_ARG\|ARGC_VARS' 
}
";

    snapshot_compgen!(
        script,
        [
            vec!["prog", "--oa=abc"],
            vec!["prog", "oa=abc"],
            vec!["prog", "--", "--oa=abc"]
        ]
    );
}

#[test]
fn arg_terminated() {
    let script = r###"
# @arg cmd
# @arg args~[`_choice_fn`]
_choice_fn() {
    echo __argc_filter=
    echo ${argc__positionals[@]}
    echo ok
}
"###;

    snapshot_compgen!(
        script,
        [
            vec!["sudo", "cmd", ""],
            vec!["sudo", "cmd", "-"],
            vec!["sudo", "cmd", "--"],
            vec!["sudo", "cmd", "--foo"],
            vec!["sudo", "cmd", "foo"],
        ]
    );
}

#[test]
fn option_terminated() {
    let script = r###"
# @option --oa~[`_choice_fn`]
# @option --ob
_choice_fn() {
    echo __argc_filter=
    echo ok
}
"###;

    snapshot_compgen!(
        script,
        [
            vec!["prog", "--oa"],
            vec!["prog", "--oa", ""],
            vec!["prog", "--oa", "--"],
            vec!["prog", "--oa", "v1", "v2"],
            vec!["prog", "--oa", "--", ""],
        ]
    );
}

#[test]
fn option_prefixed() {
    let script = r###"
# @option -D-*[`_choice_fn`]
# @option -X --ox-*[`_choice_fn`]
# @option -s-[`_choice_fn`]
_choice_fn() {
    echo VAR1=value1 
    echo VAR2=value2
    echo VAR3
}
"###;

    snapshot_compgen!(
        script,
        [
            vec!["prog", "-D"],
            vec!["prog", "-DVAR1"],
            vec!["prog", "-X"],
            vec!["prog", "-XVAR1"],
            vec!["prog", "-XVAR3", "-"],
            vec!["prog", "--ox"],
            vec!["prog", "--ox", "VAR1"],
            vec!["prog", "-s"],
            vec!["prog", "-sVAR3", "-"],
        ]
    );
}

#[test]
fn last_arg_option_assign() {
    let script = r###"
# @option --oa~[`_choice_fn`]
# @option --ob <file>
# @arg args~[`_choice_fn`]
_choice_fn() {
    echo __argc_filter=
    echo ok
}
"###;

    snapshot_compgen!(
        script,
        [
            vec!["prog", "--ob="],
            vec!["prog", "--oa", "--ob="],
            vec!["prog", "abc", "--ob="],
        ]
    );
}

#[test]
fn fallback_comp_file() {
    let script = r###"
# @cmd
args() {
    :;
}

# @cmd
# @option --file
# @option --value
cmd() {
    :;
}
"###;

    snapshot_compgen!(
        script,
        [
            vec!["prog", "args", ""],
            vec!["prog", "args", "v"],
            vec!["prog", "cmd", "--file", ""],
            vec!["prog", "cmd", "--file", "v"],
            vec!["prog", "cmd", "--value", ""],
            vec!["prog", "cmd", "--value", "v"],
        ]
    );
}

#[test]
fn redirect_symbols() {
    let script = r###"
# @option --oa
# @arg text*
"###;

    snapshot_compgen!(script, [vec!["prog", ">", "Argc"]], argc::Shell::Bash);
}

#[test]
fn delegated() {
    let script = r###"
# @arg args~[`_choice_delegate`]

_choice_delegate() {
    echo $1
}
"###;

    snapshot_compgen!(
        script,
        [vec!["prog", "abc"], vec!["prog", "-a"], vec!["prog", "-"]],
        argc::Shell::Bash
    );
}

#[test]
fn cmd_aliases() {
    let script = r###"
# @cmd
# @alias a, ab
abc() { :; }
"###;

    snapshot_compgen!(script, [vec!["prog", ""]], argc::Shell::Bash);
}

#[test]
fn multi_char() {
    let script = r#"
# @option --oa*,[`_choice_fn`]
_choice_fn() {
    echo -e "abc\ndef\nijk"
}
"#;

    snapshot_compgen_shells!(script, ["prog", "--oa", "abc,"]);
}

#[test]
fn multi_char2() {
    let script = r#"
# @option --oa*,[`_choice_fn`]
_choice_fn() {
    echo -e "abc\ndef\nijk"
}
"#;

    snapshot_compgen_shells!(script, ["prog", "--oa=abc,"]);
}

#[test]
fn starts_quote() {
    let script = r#"
# @option --oa[`_choice_fn`]
_choice_fn() {
    echo -e "abc\ndef\nijk"
}
"#;

    snapshot_compgen_shells!(script, ["prog", "--oa='"]);
}

#[test]
fn starts_quote2() {
    let script = r#"
# @option --oa[`_choice_fn`]
_choice_fn() {
    echo -e "abc\ndef\nijk"
}
"#;

    snapshot_compgen_shells!(script, ["prog", "'--oa="]);
}

#[test]
fn desc() {
    let script = r#"
# @option --oa[`_choice_fn`]
_choice_fn() {
    echo -e "abc\tdesc"
    echo -e "def\t(desc) "
    echo -e " ijk\t  value (desc)"
    echo -e " xyz \t[desc]"
    echo -e " cjk\t福聲幸雪弓們家扒乍植哪黑信，坡也士背文反四未間美穿八和經。何朵申別兆洋行苗青誰圓弓葉福音語：向哭扒長次友誰員完"
}
"#;

    snapshot_compgen_shells!(script, ["prog", "--oa", ""]);
}

#[test]
fn desc2() {
    let script = r#"
# @option --oa[`_choice_fn`]
_choice_fn() {
    echo -e "abc\t(desc1)"
    echo -e "def\t(desc2)"
}
"#;

    snapshot_compgen_shells!(script, ["prog", "--oa", "a"]);
}

#[test]
fn escape() {
    let script = r#"
# @option --oa[`_choice_fn`]
_choice_fn() {
    echo -e "a:b>c"
    echo -e "d:e>f"
}
"#;

    snapshot_compgen_shells!(script, ["prog", "--oa", ""]);
}

#[test]
fn bash_shell() {
    let script = r#"
# @option --oa[`_choice_fn`]
# @option --ob[`_choice_fn2`]
# @option --oc <file>
# @flag   --fa
_choice_fn() {
	echo "abc:def:xyz"
	echo "abc:def:tsr"
	echo "abc:ijk:abc"
	echo "abc:ijk:xyz"
}
_choice_fn2() {
    echo __argc_prefix=/A/
    echo __argc_filter=B
    echo -e "B"
    echo -e "B/\0"
}
"#;

    snapshot_compgen!(
        script,
        [
            vec!["prog", "--oa", ""],
            vec!["prog", "--oa", "abc:"],
            vec!["prog", "--ob", "/A/B"],
            vec!["prog", "--oc", "foo="],
            vec!["prog", "-"]
        ],
        argc::Shell::Bash
    );
}

#[test]
fn kinds_of() {
    let script = r#"
# @arg val[`_choice_fn`]
_choice_fn() {
    echo -e "a1\tdesc a1"
    echo -e "b1\0\tdesc b1"
    echo -e "c1\t"
    echo -e "d1\0"
    echo -e "e1"
    echo -e "f1\t/color:yellow\tdesc f1"
}
"#;

    snapshot_compgen_shells!(script, ["prog", ""]);
}

#[test]
fn filter_quote() {
    let script = r#"
# @arg args[`_choice_fn`]
_choice_fn() {
    echo "__argc_prefix=${ARGC_CWORD%%=*}="
    echo "__argc_filter=${ARGC_CWORD#*=}"
    echo foo
    echo bar
    :;
}
"#;

    snapshot_compgen_shells!(script, ["prog", "v='"]);
}

#[test]
fn color() {
    let script = r#"
# @arg val[`_fn_color`]
_fn_color() {
    echo -e "kindFlag\0\t/color:cyan"
    echo -e "kindOption\0\t/color:cyan,bold"
    echo -e "kindCommand\0\t/color:magenta"
    echo -e "kindDir\0\t/color:blue,bold"
    echo -e "kindFile\0\t/color:default"
    echo -e "kindFileExe\0\t/color:green,bold"
    echo -e "kindSymlink\0\t/color:cyan,bold"
    echo -e "kindValue\0\t/color:green"
    echo -e "colorBlack\0\t/color:black"
    echo -e "colorBlackBold\0\t/color:black,bold"
    echo -e "colorRed\0\t/color:red"
    echo -e "colorRedBold\0\t/color:red,bold"
    echo -e "colorGreen\0\t/color:green"
    echo -e "colorGreenBold\0\t/color:green,bold"
    echo -e "colorYellow\0\t/color:yellow"
    echo -e "colorYellowBold\0\t/color:yellow,bold"
    echo -e "colorBlue\0\t/color:blue"
    echo -e "colorBlueBold\0\t/color:blue,bold"
    echo -e "colorMagenta\0\t/color:magenta"
    echo -e "colorMagentaBold\0\t/color:magenta,bold"
    echo -e "colorCyan\0\t/color:cyan"
    echo -e "colorCyanBold\0\t/color:cyan,bold"
    echo -e "colorWhite\0\t/color:white"
    echo -e "colorWhiteBold\0\t/color:white,bold"
    echo -e "colorDefault\0\t/color:default"
    echo -e "colorDefaultBold\0\t/color:default,bold"
}
"#;
    snapshot_compgen_shells!(script, ["prog", ""]);
}

mod filedir {
    #[cfg(windows)]
    const TEST_SHELL: argc::Shell = argc::Shell::Powershell;
    #[cfg(not(windows))]
    const TEST_SHELL: argc::Shell = argc::Shell::Elvish;

    const VALUE_NAME_SCRIPT: &str = r###"
# @option --oa <file>
# @option --ob <file:.zsh>
# @option --oc <dir>
"###;

    #[cfg(not(windows))]
    #[test]
    fn value_name() {
        snapshot_compgen!(
            VALUE_NAME_SCRIPT,
            [
                vec!["prog", "--oa", "src/"],
                vec!["prog", "--oa", "src/p"],
                vec!["prog", "--oa", "./src/"],
                vec!["prog", "--oa", "C"],
                vec!["prog", "--oa", "./C"],
                vec!["prog", "--ob", "src/bin/argc/completions/ar"],
                vec!["prog", "--oc", "src/"],
            ],
            TEST_SHELL
        );
    }

    #[cfg(windows)]
    #[test]
    fn value_name_win() {
        snapshot_compgen!(
            VALUE_NAME_SCRIPT,
            [
                vec!["prog", "--oa", "src\\"],
                vec!["prog", "--oa", "src\\p"],
                vec!["prog", "--oa", ".\\src\\"],
                vec!["prog", "--oa", "C"],
                vec!["prog", "--oa", ".\\C"],
                vec!["prog", "--ob", "src\\bin\\argc\\completions\\ar"],
                vec!["prog", "--oc", "src\\"],
            ],
            TEST_SHELL
        );
    }

    const CD_SCRIPT: &str = r#"
# @option --oa[`_choice_oa`]
# @option --ob[`_choice_ob`]
# @arg val[`choice_val`]
_choice_oa() {
    echo "__argc_cd=src"
    echo "__argc_value=file"
}
_choice_ob() {
    echo -e "__argc_suffix=:\0"
    echo "__argc_cd=src"
    echo "__argc_value=file"
}
choice_val() {
    if [[ "$1" == *"="* ]]; then
        echo __argc_prefix=${1%%=*}=
        echo __argc_filter=${1#*=}
        echo __argc_cd=src
        echo __argc_value=file

    fi
}
    "#;

    #[cfg(not(windows))]
    #[test]
    fn cd() {
        snapshot_compgen!(
            CD_SCRIPT,
            [
                vec!["prog", "--oa", ""],
                vec!["prog", "--oa="],
                vec!["prog", "--ob", ""],
                vec!["prog", "foo="]
            ],
            TEST_SHELL
        );
    }

    #[cfg(windows)]
    #[test]
    fn cd_win() {
        snapshot_compgen!(
            CD_SCRIPT,
            [
                vec!["prog", "--oa", ""],
                vec!["prog", "--ob", ""],
                vec!["prog", "--oa="],
                vec!["prog", "foo="]
            ],
            TEST_SHELL
        );
    }
}
