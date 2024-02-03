# Variables

## Shell variables

Argc is equivalent to a layer of argument parsing on top of bash, so you can use shell variables normally.

```sh
# @cmd
cmd() {
  echo $1 $2
  echo "$*"
  echo "$@"
}
```

## Argc variables

### options/flags/positional variables

Argc automatically generates variables for each option/flag/arg.

Here is a simple script:
```sh
# @option --oa
# @option --ob*
# @flag   --fa
# @arg va
# @arg vb*

eval "$(argc --argc-eval "$0" "$@")"

echo '--oa:' $argc_oa
echo '--ob:' ${argc_ob[@]}
echo '--fa:' $argc_fa
echo '  va:' $argc_va
echo '  va:' ${argc_vb[@]}
```

If we run the script:
```
./script.sh --oa a --ob=b1 --ob=b2 --fa foo bar baz
```
It will print:
```
--oa: a
--ob: b1 b2
--fa: 1
  va: foo
  va: bar baz
```

### built-in variables

- `argc__args`:  The command line args
- `argc__cmd_arg_index`: The index of the command arg in `argc__args`
- `argc__fn`: The name of argc command func
- `argc__option`: The variable name of the option that is currently being completed.
- `argc__positionals`: The positional args

Run command

```
git reset --hard <tab>
```

Argc will generate variable:
```sh
argc__args=([0]="git" [1]="reset" [2]="--hard" [3]="")
argc__cmd_arg_index=1
argc__cmd_fn=reset
argc__positionals=([0]="")
```

Run command

```
find . -name '*lib*' -type <tab>
```

Argc will generate variable:
```sh
argc__args=([0]="find" [1]="." [2]="-name" [3]="'*lib*'" [4]="-type" [5]="")
argc__cmd_arg_index=0
argc__option=argc_type
argc__positionals=([0]=".")
```

## Environment variables

### User provide

- `ARGC_SHELL_PATH`: Specify the shell/bash path to use for `argc`.
- `ARGC_SCRIPT_NAME`: Specify the script filename to override the default `Argcfile.sh`. e.g. `Taskfile.sh`
- `ARGC_COMPGEN_DESCRIPTION`: If value is 0 or false, the generated completion candidates do not contain descriptions.
- `ARGC_COMPLETIONS_PATH`: Argc-based completion script searching path.
                           Colon-seperated in non-windows OS. Semicolon-separated in Windows.
                           Only if the arc-based completion script for the `<command>` is under the `ARGC_COMPLETIONS_PATH` or `PATH`, can it enable completion by sourcing `argc --argc-completions bash <command>`.

### Argc injected into Argcfile.sh

- `ARGC_PWD`: Current workdir. Only available in Argcfile.sh.

### Argc injected into choice-fn

- `ARGC_OS`: The OS type
- `ARGC_COMPGEN`: If value is 1, the script is called to generate completion candidates.
- `ARGC_CWORD`: The last word in the command line (processed). Used to filter completion candidates.
- `ARGC_LAST_ARG`: The last word in the command line (raw).

The difference between `ARGC_CWORD` and `ARGC_LAST_ARG`:
- If the command line is `git --git-dir=git`, then ARGC_LAST_ARG=`--git-dir=git` ARGC_CWORD=`git`
- If the command line is `bat --theme 'Solarized`, then ARGC_LAST_ARG=`'Solarized` ARGC_CWORD=`Solarized`
