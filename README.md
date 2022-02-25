# Argc

Argc is a handy way to parse shell script parameters.

## Install

Download from [Relase](https://github.com/sigoden/argc/releases)

or 

```
cargo install --locked argc
```

## Get Started

Write the following `demo.sh` script

```sh
# @flag     -q --quit                   Do not print anything to stdout
# @option   --grep* <REGEX>             Limit the commits output
# @option   --color[auto|always|never]  Print colorful output 
# @arg      pathspec+                   Files to handle
```

Argc parses command arguments and prints variables

```sh
argc demo.sh --grep FOO --grep BAR BAZ -q README.md CHANGELOG.md
```

```
argc_quit=1
argc_grep=( "FOO" "BAR" "BAZ" )
argc_color="auto"
argc_pathspec=( "README.md" "CHANGELOG.md" )
```

Argc recognizes `-h` option and prints help text


```sh
argc demo.sh -h
```

```
demo 

USAGE:
    demo [OPTIONS] <PATHSPEC>...

ARGS:
    <PATHSPEC>...    Files to handle

OPTIONS:
        --color <COLOR>      Print colorful output [default: auto] [possible values: auto, always,
                             never]
        --grep <REGEX>...    Limit the commits output
    -h, --help               Print help information
    -q, --quit               Do not print anything to stdout
```

Argc identifies parameter errors and prints error text

```
argc demo.sh --color=none
```

```
error: "none" isn't a valid value for '--color <COLOR>'
        [possible values: auto, always, never]

USAGE:
    demo --color <COLOR> <PATHSPEC>...

For more information try --help

```

How Argc works:

1. Extract parameter definitions from script comments
2. Parse command line arguments
3. If the parameter is abnormal, output error text or help information
4. If everything is normal, output the parsed parameter variable


```sh
res=$(argc $0 "$@")
if [ $? -eq  1 ]; then
    echo -n $res
    exit 1
fi
eval "$res"

echo ${argc_pathspec[@]}
```

The above code is too redundant, and Argc provides the `-e` option to rescue


```sh
eval "(argc -e $0 "$@")"

echo ${argc_pathspec[@]}
```

## Tag


Argc generates parsing rules and help documentation based on tags (fields marked with `@` in comments).

```sh
# @describe   A fictional versioning CLI
# @version    2.17.1 
# @author     nobody <nobody@example.com>
# @flag       --no-pager          Do not pipe Git output into a pager
# @option     --git-dir=.git      Set the path to the repository

# @cmd        Shows the commit log.
# @arg        refspec*            Specify what destination ref
log() {
    echo git log ${argc_refspec[@]}
}
```

```
test2 2.17.1
nobody <nobody@example.com>
A fictional versioning CLI

USAGE:
    test2 [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --git-dir <GIT-DIR>    Set the path to the repository [default: .git]
    -h, --help                 Print help information
        --no-pager             Do not pipe Git output into a pager
    -V, --version              Print version information

SUBCOMMANDS:
    help    Print this message or the help of the given subcommand(s)
    log     Shows the commit log.
```

### @describe

```sh
# @describe [string]

# @describe A fictional versioning CLI
```

Define description

### @version

```sh
# @version [string]
# @version 2.17.1 
```

Define version


### @author

```sh
# @author [string]
# @author nobody <nobody@example.com>
```

Define author

### @cmd

```sh
# @cmd [string]

# @cmd Shows the commit log.
log() {
}
```
Define subcommand

### @option

```sh
# @cmd [short] [long][modifer] [notation] [string]

# @option -j, --threads <NUM>       Number of threads to use.
# @option --grep* <PATTERN>
# @option --dump-format[=json|yaml]
# @option --shell-arg=-cu 
```

Define value option

#### modifer

The symbol after the long option name is the modifier, such as `*` in `--grep*`

- `*`: occur multiple times, optional
- `+`: occur multiple times, required
- `!`: required
- `=value`: default value
- `[a|b|c]`: choices
- `[=a|b|c]`: choices, first is default.

#### notation

Used to indicate that the option is a value option, other than a flag option.

If not provided, the option name is used as a placeholder by default.

You can use placeholder hint option value types `<NUM>`, `<PATH>`, `<PATTERN>`, `<DATE>`

### @flag

```sh
# @flag [short] [long] [help string]

## @flag  --no-pager
## @flag  -q, --quiet Do not print anything to stdout
```

Define flag option

### @arg

```sh
# @arg <name>[modifer] [help string]

## @arg pathspec* Files to add content from.
```
Define positional arguement

#### modifer

- `*`: occur multiple times, optional
- `+`: occur multiple times, required
- `!`: required