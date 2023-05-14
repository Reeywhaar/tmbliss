use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Command {
    /// Runs command in given directory and marks files as excluded from backup
    Run {
        /// Directory paths to run the command in. [--path ... --path ...]
        #[arg(long)]
        path: Vec<String>,

        /// Dry run. Only show list of files that would be excluded
        #[arg(long, default_value = "false")]
        dry_run: bool,

        /// Force include file globs into backup. [--allowlist-glob ... --allowlist-glob ...]
        #[arg(long)]
        allowlist_glob: Vec<String>,

        /// Force include file paths into backup. [--allowlist-path ./1 --allowlist-path ./2]
        #[arg(long)]
        allowlist_path: Vec<String>,

        /// Skip file globs from checking.
        /// Difference with allowlist is that if condition
        /// met than program wont do processing for child directories
        /// [--skip-glob ... --skip-glob ...]
        #[arg(long)]
        skip_glob: Vec<String>,

        /// Skip file paths from checking.
        /// Difference with allowlist is that if condition
        /// met than program wont do processing for child directories
        /// [--skip-path ./1 --skip-path ./2]
        #[arg(long)]
        skip_path: Vec<String>,
    },

    /// Runs command in given directory and shows files which would be excluded from backup. Alias for 'run --dry-run'
    List {
        /// Directory paths to run the command in. [--path ... --path ...]
        #[arg(long)]
        path: Vec<String>,

        /// Force include file globs into backup. [--allowlist-glob ... --allowlist-glob ...]
        #[arg(long)]
        allowlist_glob: Vec<String>,

        /// Force include file paths into backup. [--allowlist-path ./1 --allowlist-path ./2]
        #[arg(long)]
        allowlist_path: Vec<String>,

        /// Skip file globs from checking.
        /// Difference with allowlist is that if condition
        /// met than program won't do processing for child directories
        /// [--skip-glob ... --skip-glob ...]
        #[arg(long)]
        skip_glob: Vec<String>,

        /// Skip file paths from checking.
        /// Difference with allowlist is that if condition
        /// met than program won't do processing for child directories
        /// [--skip-path ./1 --skip-path ./2]
        #[arg(long)]
        skip_path: Vec<String>,
    },

    /// Runs command with a configuration file
    Conf {
        /// Configuration file path
        #[arg(long)]
        path: String,

        /// Dry run. Overrides configuration file option
        #[arg(long)]
        dry_run: Option<bool>,
    },
    /// Reset all exclusions in given directory
    Reset {
        /// Directory path
        #[arg(long)]
        path: String,

        /// Dry run. Only show list of files that would be reset
        #[arg(long, default_value = "false")]
        dry_run: bool,

        /// Skip reset for glob matched files. [--allowlist-glob ... --allowlist-glob ...]
        #[arg(long)]
        allowlist_glob: Vec<String>,

        /// Skip reset for matched paths.  [--allowlist-path ./1 --allowlist-path ./2]
        #[arg(long)]
        allowlist_path: Vec<String>,
    },
    /// Show excluded files starting from given directory: Alias for 'reset --dry-run'
    ShowExcluded {
        /// Directory path
        #[arg(long)]
        path: String,

        /// Skip reset for glob matched files. [--allowlist-glob ... --allowlist-glob ...]
        #[arg(long)]
        allowlist_glob: Vec<String>,

        /// Skip reset for matched paths.  [--allowlist-path ./1 --allowlist-path ./2]
        #[arg(long)]
        allowlist_path: Vec<String>,
    },
    /// Generate markdown help
    MarkdownHelp,
}

#[cfg(test)]
mod tests {
    use crate::*;
    use clap::Parser;

    #[test]
    fn it_parses_paths() {
        let args = Args::parse_from(["tmbliss", "run", "--path", "./1", "--path", "./2"]);
        assert_eq!(
            args.command,
            Command::Run {
                path: [String::from("./1"), String::from("./2")].to_vec(),
                dry_run: false,
                allowlist_glob: vec![],
                allowlist_path: vec![],
                skip_glob: vec![],
                skip_path: vec![],
            }
        );
    }

    #[test]
    fn it_parses_excludes() {
        let args = Args::parse_from([
            "tmbliss",
            "run",
            "--path",
            "./",
            "--dry-run",
            "--allowlist-glob",
            ".env",
            "--allowlist-glob",
            ".env.*",
        ]);
        assert_eq!(
            args.command,
            Command::Run {
                path: [String::from("./")].to_vec(),
                dry_run: true,
                allowlist_glob: vec![String::from(".env"), String::from(".env.*")],
                allowlist_path: vec![],
                skip_glob: vec![],
                skip_path: vec![],
            }
        );
    }

    #[test]
    fn it_parses_allowlist_paths() {
        let args = Args::parse_from([
            "tmbliss",
            "run",
            "--path",
            "./",
            "--dry-run",
            "--allowlist-path",
            "./1",
            "--allowlist-path",
            "./2",
        ]);
        assert_eq!(
            args.command,
            Command::Run {
                path: [String::from("./")].to_vec(),
                dry_run: true,
                allowlist_glob: vec![],
                allowlist_path: vec![String::from("./1"), String::from("./2")],
                skip_glob: vec![],
                skip_path: vec![],
            }
        );
    }

    #[test]
    fn it_overrides_dry_run_in_conf_mode() {
        let args = Args::parse_from(["tmbliss", "conf", "--path", "./conf.json"]);
        assert_eq!(
            args.command,
            Command::Conf {
                path: "./conf.json".to_string(),
                dry_run: None
            }
        );
        {
            let args = Args::parse_from([
                "tmbliss",
                "conf",
                "--path",
                "./conf.json",
                "--dry-run",
                "true",
            ]);
            assert_eq!(
                args.command,
                Command::Conf {
                    path: "./conf.json".to_string(),
                    dry_run: Some(true)
                }
            );
        }
    }
}
