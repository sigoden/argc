# Argc Documentation

Argc is a Bash CLI framework and command runner. It lets you describe a CLI with comment tags, then generates argument parsing, help text, validation, and shell completion from that description.

## Documentation Map

| Page | What it covers |
| --- | --- |
| [Comment Tags](./comment-tags.md) | Every supported tag: format, behavior, generated variables, and examples |
| [Argc Tooling](./argc-tooling.md) | Every `--argc-*` tool, including eval, create, build, man pages, completions, export, parallel execution, and runtime paths |
| [Command Runner](./command-runner.md) | Recipes, dependencies, hooks, environment variables, required tools, and project-root behavior |
| [Metadata](./meta.md) | Every supported `@meta` key and its behavior |
| [Compgen](./compgen.md) | Completion engine: command lookup, path completion, candidate protocol, control lines, and patterns |
| [External Subcommands](./external-subcommands.md) | Delegating to external argc scripts, naming rules, and limitations |
| [Advanced Topics](./advanced.md) | Escape hatch and trailing underscores |
| [Variables](./variables.md) | Generated `argc_*` variables, built-in variables, and general environment variables |
| [Grammar](./grammar.ebnf) | Formal EBNF grammar |
| [Export Formats](./export-formats.md) | JSON object schema produced by `--argc-export` and the fields of each exported type |

## Conventions used in the docs

- `prog` is a placeholder for your script name.
- Code blocks use `sh` for shell code and plain text for command output.
- `$argc_*` variables are created by argc after argument parsing.
