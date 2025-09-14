#[cfg(test)]
pub mod test_utils;

mod args;
mod conf;
mod constants;
mod directory_iterator;
mod git;
mod logger;
mod recursive_directory_iterator;
mod time_machine;

use std::collections::HashSet;
use std::path::Path;
use std::rc::Rc;
use std::{cell::RefCell, path::PathBuf};

use anyhow::{Context, Result};
use glob_match::glob_match;
use recursive_directory_iterator::RecursiveDirectoryIterator;

pub use crate::args::{Args, Command};
use crate::conf::Conf;
use crate::directory_iterator::DirectoryIterator;
use crate::git::Git;
use crate::logger::Logger;
pub use crate::time_machine::{TimeMachine, TimeMachineError};

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
                skip_errors,
                exclude_path,
            } => {
                let logger = Logger { filter: None };

                Self::mark_files(
                    Conf {
                        paths: path,
                        dry_run,
                        allowlist_glob,
                        allowlist_path,
                        skip_glob,
                        skip_path,
                        skip_errors,
                        exclude_paths: exclude_path,
                    },
                    &logger,
                )
            }
            Command::List {
                path,
                allowlist_glob,
                allowlist_path,
                skip_glob,
                skip_path,
                skip_errors,
                exclude_path,
            } => {
                let logger = Logger { filter: None };

                Self::mark_files(
                    Conf {
                        paths: path,
                        dry_run: true,
                        allowlist_glob,
                        allowlist_path,
                        skip_glob,
                        skip_path,
                        skip_errors,
                        exclude_paths: exclude_path,
                    },
                    &logger,
                )
            }
            Command::Conf { path, dry_run } => {
                let conf = Conf::parse(&path);
                match conf {
                    Ok(mut conf) => {
                        let logger = Logger { filter: None };

                        if let Some(dry_run) = dry_run {
                            conf.dry_run = dry_run;
                        }
                        Self::mark_files(conf, &logger)
                    }
                    Err(e) => Err(e),
                }
            }
            Command::Service { path, dry_run } => {
                let conf = Conf::parse(&path);
                match conf {
                    Ok(mut conf) => {
                        let filter = |label: &str, _message: &str| {
                            if label == "excluded" {
                                return true;
                            }
                            false
                        };
                        let logger = Logger {
                            filter: Some(&filter),
                        };
                        if let Some(dry_run) = dry_run {
                            conf.dry_run = dry_run;
                        }
                        logger.log("started", &chrono::Local::now().to_string());
                        logger.log("dry run", &conf.dry_run.to_string());
                        Self::mark_files(conf, &logger)?;
                        logger.log("ended", &chrono::Local::now().to_string());
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            Command::Reset {
                path,
                dry_run,
                allowlist_glob,
                allowlist_path,
            } => Self::reset_files(
                Path::new(&path),
                dry_run,
                allowlist_glob,
                allowlist_path,
                &Logger { filter: None },
            ),
            Command::ShowExcluded {
                path,
                allowlist_glob,
                allowlist_path,
            } => Self::reset_files(
                Path::new(&path),
                true,
                allowlist_glob,
                allowlist_path,
                &Logger { filter: None },
            ),
            Command::MarkdownHelp => {
                clap_markdown::print_help_markdown::<Args>();
                Ok(())
            }
        }
    }

    fn mark_files(conf: Conf, logger: &Logger) -> Result<()> {
        let processed: Rc<RefCell<HashSet<PathBuf>>> = Rc::new(RefCell::new(HashSet::new()));

        let excluder = |path: &Path| -> bool {
            if processed.borrow().contains(path) {
                return true;
            }

            if Git::is_git(path) {
                processed.borrow_mut().insert(path.to_owned());
                return true;
            }

            for exclusion in &conf.skip_glob {
                let strpath = path.to_str();
                if strpath.is_none() {
                    continue;
                }
                let strpath = strpath.unwrap();
                if glob_match(exclusion, strpath) {
                    processed.borrow_mut().insert(path.to_owned());
                    return true;
                }
            }

            for exclusion in &conf.skip_path {
                if Self::is_inside(Path::new(exclusion), path) {
                    processed.borrow_mut().insert(path.to_owned());
                    return true;
                }
            }

            false
        };

        for item in &conf.exclude_paths {
            Self::process(Path::new(item), &conf, processed.clone(), logger)?;
        }

        for path in &conf.paths {
            Self::process_directory(
                Path::new(path),
                &conf,
                Some(&excluder),
                processed.clone(),
                logger,
            )
            .with_context(|| format!("Can't process directory {}", path))?;
        }

        Ok(())
    }

    fn reset_files(
        path: &Path,
        dry_run: bool,
        allowlist_glob: Vec<String>,
        allowlist_path: Vec<String>,
        logger: &Logger,
    ) -> Result<()> {
        let iterator = RecursiveDirectoryIterator {
            path,
            op: &|path| {
                for exclusion in &allowlist_path {
                    if Self::is_inside(Path::new(exclusion), path) {
                        return Ok(true);
                    }
                }
                for exclusion in &allowlist_glob {
                    if glob_match(exclusion, &path.to_string_lossy()) {
                        return Ok(true);
                    }
                }
                if TimeMachine::is_excluded(path)? {
                    logger.log("excluded", &path.to_string_lossy());
                    if !dry_run {
                        TimeMachine::remove_exclusion(path)?
                    }
                }
                Ok(true)
            },
        };

        iterator.iterate()
    }

    fn get_git_excludes(path: &Path, conf: &Conf) -> Vec<PathBuf> {
        let git = Git {
            path: path.to_path_buf(),
        };
        git.get_ignores_list()
            .unwrap_or_default()
            .into_iter()
            .filter(|item| {
                for exclusion in &conf.skip_path {
                    if Self::is_inside(Path::new(exclusion), item) {
                        return false;
                    }
                }
                for exclusion in &conf.skip_glob {
                    if glob_match(exclusion, &item.to_string_lossy()) {
                        return false;
                    }
                }
                for exclusion in &conf.allowlist_path {
                    if Self::is_inside(Path::new(exclusion), item) {
                        return false;
                    }
                }
                for exclusion in &conf.allowlist_glob {
                    if glob_match(exclusion, &item.to_string_lossy()) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    fn process(
        item: &Path,
        conf: &Conf,
        processed: Rc<RefCell<HashSet<PathBuf>>>,
        logger: &Logger,
    ) -> Result<()> {
        if processed.borrow().contains(item) {
            return Ok(());
        }

        processed.borrow_mut().insert(item.to_owned());

        let check_result = TimeMachine::is_excluded(item);
        match check_result {
            Ok(is_excluded) => {
                if is_excluded {
                    logger.log("excluded", &item.to_string_lossy());
                    return Ok(());
                } else {
                    logger.log("new", &item.to_string_lossy());
                }
            }
            Err(e) => {
                if conf.skip_errors {
                    logger.log(
                        "error_checking",
                        &[item.to_string_lossy().as_ref(), &e.to_string()].join(", "),
                    );
                    return Ok(());
                } else {
                    return Err(e.into());
                }
            }
        }

        if !conf.dry_run {
            let result = TimeMachine::add_exclusion(item);
            match result {
                Ok(_) => {}
                Err(e) => {
                    if conf.skip_errors {
                        logger.log(
                            "error_excluding",
                            &[item.to_string_lossy().as_ref(), &e.to_string()].join(", "),
                        );
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }

        Ok(())
    }

    fn is_inside(root: &Path, child: &Path) -> bool {
        let root = root
            .canonicalize()
            .unwrap_or_else(|_| Path::new(root).to_path_buf());
        let child = child
            .canonicalize()
            .unwrap_or_else(|_| Path::new(child).to_path_buf());

        root.eq(&child) || (child.starts_with(&root) && !root.starts_with(&child))
    }

    fn process_directory(
        path: &Path,
        conf: &Conf,
        excluder: Option<&dyn Fn(&Path) -> bool>,
        processed: Rc<RefCell<HashSet<PathBuf>>>,
        logger: &Logger,
    ) -> Result<()> {
        if let Some(excluder) = excluder {
            if excluder(path) {
                return Ok(());
            }
        }

        if TimeMachine::is_excluded(path)? {
            Self::process(path, conf, processed, logger)
                .with_context(|| format!("Can't process path {}", path.display()))?;
            return Ok(());
        }

        let excludes = Self::get_git_excludes(path, conf);

        for item in excludes.clone() {
            Self::process(&item, conf, processed.clone(), logger).with_context(|| {
                format!(
                    "Can't process paths {}",
                    excludes
                        .iter()
                        .map(|p| p.to_string_lossy())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        }

        let directory_iterator = DirectoryIterator {
            path,
            exclude: excluder,
        };
        let directories = directory_iterator
            .list()
            .with_context(|| format!("Can't list directory {}", path.display()))?;

        for path in directories {
            Self::process_directory(&path, conf, excluder, processed.clone(), logger)
                .with_context(|| format!("Can't process directory {}", path.display()))?;
        }

        Ok(())
    }
}
