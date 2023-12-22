# @meta

Add metadata

```sh
# @meta combine-shorts
# @meta inherit-flag-options
```

| syntax               | scope | description                                                 |
| -------------------- | ----- | ----------------------------------------------------------- |
| combine-shorts       | root  | Short flags can be combined, e.g. `prog -xf => prog -x -f ` |
| inherit-flag-options | root  | All subcommands inherit flag/options from parent command.   |