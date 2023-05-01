# TMBliss

[**Cli Documentation**](./cli.md)

TMBliss is a rust written cli utility for MacOS heavily inspired by [tmignore](https://github.com/samuelmeuli/tmignore). It adds exclusions for derived development and other undesired files such as `node_modules` directory or build output to **Time Machine** backup.

# Basic Usage

To show which files would be excluded from given directory you can run:

```
tmbliss --path ~/Dev --whitelist-glob "**/.env" --dry-run
```

Every option can be seen in [Cli Documentation](./cli.md)
