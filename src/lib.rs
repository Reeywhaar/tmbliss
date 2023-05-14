#[cfg(test)]
pub mod test_utils;

mod args;
mod conf;
mod directory_iterator;
mod git;
mod recursive_directory_iterator;
mod time_machine;

use std::cell::RefCell;
use std::collections::HashSet;
use std::path::Path;
use std::rc::Rc;

use anyhow::{anyhow, Context, Result};
use glob_match::glob_match;
use recursive_directory_iterator::RecursiveDirectoryIterator;

pub use crate::args::{Args, Command};
use crate::conf::Conf;
use crate::directory_iterator::DirectoryIterator;
use crate::git::Git;
pub use crate::time_machine::TimeMachine;

pub struct TMBliss {}

impl TMBliss {
    pub fn run(command: Command) -> Result<()> {
        match command {
            Command::Run {
                path,
                dry_run,
                allowlist_glob,
                allowlist_path,
                skip_glob,
                skip_path,
            } => Self::mark_files(Conf {
                paths: path,
                dry_run,
                allowlist_glob,
                allowlist_path,
                skip_glob,
                skip_path,
            }),
            Command::List {
                path,
                allowlist_glob,
                allowlist_path,
                skip_glob,
                skip_path,
            } => Self::mark_files(Conf {
                paths: path,
                dry_run: true,
                allowlist_glob,
                allowlist_path,
                skip_glob,
                skip_path,
            }),
            Command::Conf { path, dry_run } => {
                let conf = Conf::parse(&path);
                match conf {
                    Ok(mut conf) => {
                        if let Some(dry_run) = dry_run {
                            conf.dry_run = dry_run;
                        }
                        Self::mark_files(conf)
                    }
                    Err(e) => Err(e),
                }
            }
            Command::Reset {
                path,
                dry_run,
                allowlist_glob,
                allowlist_path,
            } => Self::reset_files(&path, dry_run, allowlist_glob, allowlist_path),
            Command::ShowExcluded {
                path,
                allowlist_glob,
                allowlist_path,
            } => Self::reset_files(&path, true, allowlist_glob, allowlist_path),
            Command::MarkdownHelp {} => {
                clap_markdown::print_help_markdown::<Args>();
                Ok(())
            }
        }
    }

    fn mark_files(conf: Conf) -> Result<()> {
        let processed: Rc<RefCell<HashSet<String>>> = Rc::new(RefCell::new(HashSet::new()));

        let excluder = |path: &str| -> bool {
            if processed.borrow().contains(path) {
                return true;
            }

            if Git::is_git(path) {
                processed.borrow_mut().insert(path.to_string());
                return true;
            }

            for exclusion in &conf.skip_glob {
                if glob_match(exclusion, path) {
                    processed.borrow_mut().insert(path.to_string());
                    return true;
                }
            }

            for exclusion in &conf.skip_path {
                if Self::is_inside(exclusion, path) {
                    processed.borrow_mut().insert(path.to_string());
                    return true;
                }
            }

            false
        };

        for path in &conf.paths {
            Self::process_directory(path, &conf, Some(&excluder), processed.clone())
                .with_context(|| format!("Can't process directory {}", path))?;
        }

        Ok(())
    }

    fn reset_files(
        path: &str,
        dry_run: bool,
        allowlist_glob: Vec<String>,
        allowlist_path: Vec<String>,
    ) -> Result<()> {
        let iterator = RecursiveDirectoryIterator {
            path: path.to_string(),
            op: &|path| {
                for exclusion in &allowlist_path {
                    if Self::is_inside(exclusion, path) {
                        return Ok(());
                    }
                }
                for exclusion in &allowlist_glob {
                    if glob_match(exclusion, path) {
                        return Ok(());
                    }
                }
                if TimeMachine::is_excluded(path) {
                    println!("{}", path);
                    if !dry_run {
                        TimeMachine::remove_exclusion(path)?
                    }
                }
                Ok(())
            },
        };

        iterator.iterate()
    }

    fn get_git_excludes(path: &str, conf: &Conf) -> Vec<String> {
        let git = Git {
            path: path.to_string(),
        };
        git.get_ignores_list()
            .unwrap_or(Vec::new())
            .into_iter()
            .filter(|item| {
                for exclusion in &conf.skip_path {
                    if Self::is_inside(exclusion, item) {
                        return false;
                    }
                }
                for exclusion in &conf.skip_glob {
                    if glob_match(exclusion, item) {
                        return false;
                    }
                }
                for exclusion in &conf.allowlist_path {
                    if Self::is_inside(exclusion, item) {
                        return false;
                    }
                }
                for exclusion in &conf.allowlist_glob {
                    if glob_match(exclusion, item) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    fn process(
        items: Vec<String>,
        conf: &Conf,
        processed: Rc<RefCell<HashSet<String>>>,
    ) -> Result<()> {
        for item in items {
            if processed.borrow().contains(&item) {
                continue;
            }

            processed.borrow_mut().insert(item.clone());

            if TimeMachine::is_excluded(&item) {
                println!("excluded: {}", item);
            } else {
                println!("new: {}", item);
            }

            if !conf.dry_run {
                TimeMachine::add_exclusion(&item)?;
            }
        }

        Ok(())
    }

    fn is_inside(root: &str, child: &str) -> bool {
        let root = Path::new(root)
            .canonicalize()
            .unwrap_or_else(|_| Path::new(root).to_path_buf());
        let child = Path::new(child)
            .canonicalize()
            .unwrap_or_else(|_| Path::new(child).to_path_buf());

        root.eq(&child) || (child.starts_with(&root) && !root.starts_with(&child))
    }

    fn process_directory(
        path: &str,
        conf: &Conf,
        excluder: Option<&dyn Fn(&str) -> bool>,
        processed: Rc<RefCell<HashSet<String>>>,
    ) -> Result<()> {
        if let Some(excluder) = excluder {
            if excluder(path) {
                return Ok(());
            }
        }

        if TimeMachine::is_excluded(path) {
            Self::process(
                vec![Path::new(path)
                    .canonicalize()
                    .with_context(|| format!("Can't canonicalize path {}", path))?
                    .to_str()
                    .ok_or(anyhow!("Can't convert path {} to string", path))?
                    .to_string()],
                conf,
                processed,
            )
            .with_context(|| format!("Can't process path {}", path))?;
            return Ok(());
        }

        let excludes = Self::get_git_excludes(path, conf);

        Self::process(excludes.clone(), conf, processed.clone())
            .with_context(|| format!("Can't process paths {}", excludes.join(", ")))?;

        let directory_iterator = DirectoryIterator {
            path,
            exclude: excluder,
        };
        let directories = directory_iterator
            .list()
            .with_context(|| format!("Can't list directory {}", path))?;

        for path in directories {
            Self::process_directory(&path, conf, excluder, processed.clone())
                .with_context(|| format!("Can't process directory {}", path))?;
        }

        Ok(())
    }
}
