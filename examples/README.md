# Argc Examples

Each of these examples demonstrates one aspect or feature of argc.

For guides and reference documentation, see [docs/README.md](../docs/README.md).

- [demo](./demo) - A demo CLI with subcommands, flags, options, and positional args.
- [multiline](./multiline) - how to write multiline descriptions and help text.
- [nested-commands](./nested-commands) - how to declare nested subcommands with `::`.
- [hooks](./hooks) - how to use the `_argc_before` and `_argc_after` hooks.
- [parallel](./parallel) - how to run subcommands in parallel with `--argc-parallel`.

- [args](./args) - all kinds of `@arg` positional parameter definitions.
- [options](./options) - all kinds of `@option` and `@flag` declarations.
- [bind-envs](./bind-envs) - how to bind env vars to flags, options, and args.
- [envs](./envs) - all kinds of `@env` declarations.

- [default-subcommand](./default-subcommand) - how to pick a default subcommand with `@meta default-subcommand`.
- [require-tools](./require-tools) - how to verify required tools with `@meta require-tools`.
- [inherit-flag-options](./inherit-flag-options) - how to share flags with subcommands via `@meta inherit-flag-options`.
- [combine-shorts](./combine-shorts) - how to combine short flags (e.g. `-rf`) with `@meta combine-shorts`.
- [symbol](./symbol) - how to use a custom option prefix with `@meta symbol`.

- [external](./external) - how to add external subcommand scripts with `@meta external-subcommands`.
