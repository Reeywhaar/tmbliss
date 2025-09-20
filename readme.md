# TMBliss

[**Cli Documentation**](./cli.md)

TMBliss is a rust written cli utility for MacOS heavily inspired by [tmignore](https://github.com/samuelmeuli/tmignore). It adds exclusions for derived development and other undesired files such as `node_modules` directory or build output to **Time Machine** backup.

## Installation

`tmbliss` can be installed via homebrew

```
brew install reeywhaar/tap/tmbliss
```

Or by manually downloading archive from [releases](./releases/latest) page. `silicon.zip` is for silicon (M1) cpus and `intel.zip` is for intel.

## Basic Usage

To show which files would be excluded from given directory you can run:

```
tmbliss run --path ~/Dev --allowlist-glob "**/.env" --dry-run
```

Every option can be seen in [Cli Documentation](./cli.md)

## .tmbliss file
You can create `.tmbliss` file, that acts as `.gitignore` in reverse. You can declare globs to be force included into TimeMachine backup even if it is defined in `.gitignore`. Kinda same as `--allowlist-glob` but per directory
