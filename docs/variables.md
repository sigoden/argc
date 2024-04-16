# Variables

Argc streamlines argument parsing in your shell scripts, allowing you to utilize variables seamlessly.

## Shell Variables

You can employ shell variables within your argc-based scripts just like you normally would in Bash. Argc doesn't interfere with their behavior.

```sh
# @cmd
cmd() {
  echo $1 $2  # Accessing positional arguments
  echo "$*"   # All arguments as a single string
  echo "$@"   # All arguments as separate strings
}
```

## Argc-Generated Variables

Argc automatically creates variables corresponding to the options, flags, and positional arguments defined in your script using the `@option`, `@flag`, and `@arg` directives.

```sh
# @option --oa
# @option --ob*  # Multiple values allowed
# @flag   --fa
# @arg va
# @arg vb*

eval "$(argc --argc-eval "$0" "$@")"  # Initializes Argc variables

echo '--oa:' $argc_oa
echo '--ob:' ${argc_ob[@]}  # Accessing multiple values as an array
echo '--fa:' $argc_fa
echo '  va:' $argc_va
echo '  vb:' ${argc_vb[@]}
```

Running `./script.sh --oa a --ob=b1 --ob=b2 --fa foo bar baz` would output:

```
--oa: a
--ob: b1 b2
--fa: 1
  va: foo
  vb: bar baz
```

## Built-in Variables

Argc also provides built-in variables that offer information about the parsing process:

*   **`argc__args`**: An array holding all command-line arguments.
*   **`argc__positionals`**: An array containing only the positional arguments.
*   **`argc__fn`**: The name of the function that will be executed.

**Additional Variables for Completion (Used internally by Argc-Completions):**

*   **`argc__cmd_arg_index`**: Index of the command argument within `argc__args`.
*   **`argc__cmd_fn`**: Name of the command function.
*   **`argc__dash`**: Index of the first em-dash (`--`) within the positional arguments.
*   **`argc__option`**: Variable name of the option currently being completed.

These variables are particularly useful when creating custom completion scripts. 

## Environment Variables

Several environment variables allow you to tailor Argc's behavior:

**User-Defined:**

*  **`ARGC_SHELL_PATH`**: Specifies the path to the shell/bash executable used by Argc.
*  **`ARGC_SCRIPT_NAME`**: Overrides the default script filename (Argcfile.sh).
*  **`ARGC_COMPGEN_DESCRIPTION`**: Disables descriptions for completion candidates if set to 0 or false. 
*  **`ARGC_COMPLETIONS_PATH`**: Defines the search path for Argc-based completion scripts.

**Argc-Injected:**

*  **`ARGC_PWD`**: Current working directory (available only in Argcfile.sh).

**Argc-Injected (for completion):**
*  **`ARGC_OS`**: Operating system type.
*  **`ARGC_COMPGEN`**: Indicates whether the script is being used for generating completion candidates (1) or not (0).
*  **`ARGC_CWORD`**: The last word in the processed command line.

It's important to distinguish between these two variables:

*  **`ARGC_CWORD`**: This variable isolates the final word, regardless of any preceding flags or options. For example, in the command `git --git-dir=git`, `ARGC_CWORD` would be `git`.
*  **`ARGC_LAST_ARG`**: This variable captures the entire last argument, including any flags or options attached to it. In the same example, `ARGC_LAST_ARG` would be `--git-dir=git`.

Understanding these variables is key to effectively leveraging Argc's capabilities and creating robust and user-friendly command-line interfaces.
