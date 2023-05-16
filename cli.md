# Command-Line Help for `tmbliss`

This document contains the help content for the `tmbliss` command-line program.

**Command Overview:**

* [`tmbliss`↴](#tmbliss)
* [`tmbliss run`↴](#tmbliss-run)
* [`tmbliss list`↴](#tmbliss-list)
* [`tmbliss conf`↴](#tmbliss-conf)
* [`tmbliss service`↴](#tmbliss-service)
* [`tmbliss reset`↴](#tmbliss-reset)
* [`tmbliss show-excluded`↴](#tmbliss-show-excluded)
* [`tmbliss markdown-help`↴](#tmbliss-markdown-help)

## `tmbliss`

**Usage:** `tmbliss <COMMAND>`

###### **Subcommands:**

* `run` — Runs command in given directory and marks files as excluded from backup
* `list` — Runs command in given directory and shows files which would be excluded from backup. Alias for 'run --dry-run'
* `conf` — Runs command with a configuration file
* `service` — Same as 'conf' but with logging suitable for a service
* `reset` — Reset all exclusions in given directory
* `show-excluded` — Show excluded files starting from given directory: Alias for 'reset --dry-run'
* `markdown-help` — Generate markdown help



## `tmbliss run`

Runs command in given directory and marks files as excluded from backup

**Usage:** `tmbliss run [OPTIONS]`

###### **Options:**

* `--path <PATH>` — Directory paths to run the command in. [--path ... --path ...]
* `--dry-run` — Dry run. Only show list of files that would be excluded

  Default value: `false`
* `--allowlist-glob <ALLOWLIST_GLOB>` — Force include file globs into backup. Allows multiple globs. [--allowlist-glob ... --allowlist-glob ...]
* `--allowlist-path <ALLOWLIST_PATH>` — Force include file paths into backup. Allows multiple paths. [--allowlist-path ./1 --allowlist-path ./2]
* `--skip-glob <SKIP_GLOB>` — Skip file globs from checking. Difference with allowlist is that if condition met than program wont do processing for child directories. Allows multiple globs. [--skip-glob ... --skip-glob ...]
* `--skip-path <SKIP_PATH>` — Skip file paths from checking. Difference with allowlist is that if condition met than program wont do processing for child directories. Allows multiple paths. [--skip-path ./1 --skip-path ./2]
* `--skip-errors` — Skip errors when adding or checking exclusion. In case of for example insufficient permissions

  Default value: `true`
* `--exclude-path <EXCLUDE_PATH>` — Path that should be removed from time machine backup. Allows multiple paths. [--exclude-path ./1 --exclude-path ./2]



## `tmbliss list`

Runs command in given directory and shows files which would be excluded from backup. Alias for 'run --dry-run'

**Usage:** `tmbliss list [OPTIONS]`

###### **Options:**

* `--path <PATH>` — Directory paths to run the command in. [--path ... --path ...]
* `--allowlist-glob <ALLOWLIST_GLOB>` — Force include file globs into backup. [--allowlist-glob ... --allowlist-glob ...]
* `--allowlist-path <ALLOWLIST_PATH>` — Force include file paths into backup. [--allowlist-path ./1 --allowlist-path ./2]
* `--skip-glob <SKIP_GLOB>` — Skip file globs from checking. Difference with allowlist is that if condition met than program won't do processing for child directories [--skip-glob ... --skip-glob ...]
* `--skip-path <SKIP_PATH>` — Skip file paths from checking. Difference with allowlist is that if condition met than program won't do processing for child directories [--skip-path ./1 --skip-path ./2]
* `--skip-errors` — Skip errors when adding or checking exclusion. In case of for example insufficient permissions

  Default value: `true`
* `--exclude-path <EXCLUDE_PATH>` — Path that should be removed from time machine backup



## `tmbliss conf`

Runs command with a configuration file

**Usage:** `tmbliss conf [OPTIONS] --path <PATH>`

###### **Options:**

* `--path <PATH>` — Configuration file path
* `--dry-run <DRY_RUN>` — Dry run. Overrides configuration file option

  Possible values: `true`, `false`




## `tmbliss service`

Same as 'conf' but with logging suitable for a service

**Usage:** `tmbliss service [OPTIONS] --path <PATH>`

###### **Options:**

* `--path <PATH>` — Configuration file path
* `--dry-run <DRY_RUN>` — Dry run. Overrides configuration file option

  Possible values: `true`, `false`




## `tmbliss reset`

Reset all exclusions in given directory

**Usage:** `tmbliss reset [OPTIONS] --path <PATH>`

###### **Options:**

* `--path <PATH>` — Directory path
* `--dry-run` — Dry run. Only show list of files that would be reset

  Default value: `false`
* `--allowlist-glob <ALLOWLIST_GLOB>` — Skip reset for glob matched files. [--allowlist-glob ... --allowlist-glob ...]
* `--allowlist-path <ALLOWLIST_PATH>` — Skip reset for matched paths.  [--allowlist-path ./1 --allowlist-path ./2]



## `tmbliss show-excluded`

Show excluded files starting from given directory: Alias for 'reset --dry-run'

**Usage:** `tmbliss show-excluded [OPTIONS] --path <PATH>`

###### **Options:**

* `--path <PATH>` — Directory path
* `--allowlist-glob <ALLOWLIST_GLOB>` — Skip reset for glob matched files. [--allowlist-glob ... --allowlist-glob ...]
* `--allowlist-path <ALLOWLIST_PATH>` — Skip reset for matched paths.  [--allowlist-path ./1 --allowlist-path ./2]



## `tmbliss markdown-help`

Generate markdown help

**Usage:** `tmbliss markdown-help`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

