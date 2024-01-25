# @meta

Add metadata

| usage                        | description                                                   |
| ---------------------------- | ------------------------------------------------------------- |
| `@meta combine-shorts`       | Short flags can be combined, e.g. `prog -xf => prog -x -f `   |
| `@meta inherit-flag-options` | Subcommands will inherit the flags/options from their parent. |
| `@meta no-inherit-env`       | Subcommands will not inherit the envs from their parent.      |
| `@meta dotenv [<path>]`      | Load dotenv, accept an optional path.                         |