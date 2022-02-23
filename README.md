# Argc

Argc is a handy way to handle cli parameters for shell script.

## Example

The content of the sample file `git.sh` is as follows

```sh
#!/bin/bash
# @describe   A fictional versioning CLI
# @version    2.17.1 
# @author     nobody <nobody@example.com>
# @flag       --no-pager          Do not pipe Git output into a pager
# @flag       -p --paginate       Pipe all output into less or $PAGER
# @option     --git-dir=.git      Set the path to the repository

# @cmd        Shows the commit log.
# @arg        refspec*            Specify what destination ref
# @flag       --source            Print out the ref name
# @option     --decorate[=no|short|full|auto]
# @option     --grep* <REGEX>     Limit the commits output
log() {
    echo "git log"
}

# @cmd        Add file contents to the index
# @arg        pathspec+           Files to add content from. 
# @flag       -n --dry-run        Don’t actually add the file
add() {
    echo "git add"
}

eval $(argc -e $0 "$@")
```

Argc generate help

```sh
argc git.sh help
```

```
git 2.17.1
nobody <nobody@example.com>
A fictional versioning CLI

USAGE:
    git [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --git-dir <GIT-DIR>    Set the path to the repository [default: .git]
    -h, --help                 Print help information
        --no-pager             Do not pipe Git output into a pager
    -p, --paginate             Pipe all output into less or $PAGER
    -V, --version              Print version information

SUBCOMMANDS:
    add     Add file contents to the index
    help    Print this message or the help of the given subcommand(s)
    log     Shows the commit log.
```

Argc generate help for subcommand.

```sh
argc git.sh log -h
```

```
git-log 
Shows the commit log.

USAGE:
    git log [OPTIONS] [--] [REFSPEC]...

ARGS:
    <REFSPEC>...    Specify what destination ref

OPTIONS:
        --decorate <DECORATE>    [default: no] [possible values: no, short, full, auto]
        --grep <REGEX>...        Limit the commits output
    -h, --help                   Print help information
        --source                 Print out the ref name
```

Argc parsing parameter got error:

```
argc git.sh -p log --decorate shot
```

```sh
error: "shot" isn't a valid value for '--decorate <DECORATE>'
        [possible values: no, short, full, auto]

USAGE:
    git log --decorate <DECORATE>

For more information try --help
```

> content is output to stderr, exit code  `$?` = 1

Argc successfully parses arguments.

```sh
argc git.sh -p log --grep ARG1 --grep ARG2
```

```
argc_paginate=1
argc_git_dir=".git"
argc_decorate="no"
argc_grep=( "ARG1" "ARG2" )
```

> content is output to stderr, exit code  `$?` = 2

## How it works

How Argc works:

1. Extract parameter definitions from script comments
2. Parse and match command line arguments
3. If the parameter is abnormal, output error text or help information
4. If everything is normal, output the parsed parameter variable

Argc generates parsing rules and help documentation based on tags (fields marked with `@` in comments).

`@describe`, `@version`, `@author` define description, version, author respectively.

`@flag` defines flag options, `@option` defines value options, and `@arg` defines positional arguments.

`@cmd` marks subcommands.

A parameter or option name followed by `*` means zero (i.e. optional) or more values are allowed

A parameter or option name followed by `+` means that one (i.e. required) or more values are allowed

The parameter followed by `=` indicates the default value

The parameter followed by `[]` indicates optional values, the optional values are separated by `|`, and `=` is inserted before the first optional value to indicate that it is also the default value

Check [syntax](./docs//SYNTAX.md) if you have any trouble with tags.

## `-e` option

Argc has a `-e` option, which when turned on means to adjust the output specifically for the sh `eval` command.

```sh
argc -e git.sh -p log --grep ARG1 --grep ARG2
```

```
...
argc_grep=( "ARG1" "ARG2" )
log
```

Function calls are inserted at the end of normal output.


Let's check  the help text/error prompt output.

```sh
argc -e git.sh log -h 2>/dev/null
```

The normal mode is that stdout is blank, now insert the following script:

```sh
exit 1
```
This way the script will automatically exit if the parameter is wrong.

We can insert the following code at the end of the script:

```sh
eval $(argc -e $0 "$@")
```

Delegate Argc to handle command line arguments for our script.

If the arguments are OK, the variable is assigned and the function is called.

If the parameter is abnormal, print help or error message and exit execution.