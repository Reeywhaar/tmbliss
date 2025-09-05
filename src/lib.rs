use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

use anyhow::{anyhow, Context, Result};
use glob_match::glob_match;
use recursive_directory_iterator::RecursiveDirectoryIterator;

pub use crate::args::{Args, Command};
use crate::conf::Conf;
use crate::constants::TMBLISS_FILE;
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

        for item in conf.exclude_paths.clone() {
            Self::process(Path::new(&item), &conf, processed.clone(), logger)?;
        }

        for path in &conf.paths {
            Self::process_directory(Path::new(path), &conf, processed.clone(), logger)?;
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

    fn process(
        item: &Path,
        conf: &Conf,
        processed: Rc<RefCell<HashSet<PathBuf>>>,
        logger: &Logger,
    ) -> Result<()> {
        let item = item
            .canonicalize()
            .with_context(|| format!("Can't canonicalize path {}", item.display()))?;
        let item = &item;
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

    fn process_directory(
        path: &Path,
        conf: &Conf,
        processed: Rc<RefCell<HashSet<PathBuf>>>,
        logger: &Logger,
    ) -> Result<()> {
        let path = Path::new(path)
            .canonicalize()
            .with_context(|| format!("Can't canonicalize path {}", path.display()))?;
        let path = &path;
        // Gather .tmbliss globs for this directory
        let mut newconf = conf.clone();
        let mut effective_skip_glob = conf.skip_glob.clone();
        let tmbliss_globs = Self::read_tmbliss_globs(path);
        let tmbliss_globs = tmbliss_globs
            .iter()
            .map(|s| -> Result<String> {
                let stripped = if s.starts_with("/") {
                    s.strip_prefix("/")
                        .ok_or_else(|| anyhow!("Failed to strip prefix from {}", s))?
                } else {
                    s.as_str()
                };
                Ok(path.join(stripped).to_string_lossy().to_string())
            })
            .collect::<Result<Vec<String>>>()?;
        effective_skip_glob.extend(tmbliss_globs);
        newconf.skip_glob = effective_skip_glob.clone();

        // Excluder closure that uses effective_skip_glob
        let excluder = |item: &Path| -> bool {
            if item.is_file() && item.file_name() == Some(OsStr::new(TMBLISS_FILE)) {
                return true;
            }
            if item.is_file() && item.file_name() == Some(OsStr::new(".gitignore")) {
                return true;
            }
            if processed.borrow().contains(item) {
                return true;
            }
            if Git::is_git(item) {
                processed.borrow_mut().insert(item.to_path_buf());
                return true;
            }
            for exclusion in &effective_skip_glob {
                if glob_match(exclusion, &item.to_string_lossy()) {
                    processed.borrow_mut().insert(item.to_path_buf());
                    return true;
                }
            }
            for exclusion in &conf.skip_path {
                if Self::is_inside(Path::new(exclusion), item) {
                    processed.borrow_mut().insert(item.to_path_buf());
                    return true;
                }
            }
            false
        };

        if excluder(path) {
            return Ok(());
        }

        if TimeMachine::is_excluded(path)? {
            Self::process(path, &newconf, processed, logger)
                .with_context(|| format!("Can't process path {}", path.display()))?;
            return Ok(());
        }

        let excludes = Self::get_git_excludes(path, &newconf);

        let parents = |item: &Path| -> Vec<PathBuf> {
            let mut out: Vec<PathBuf> = vec![];
            let mut p = item.parent();
            while let Some(par) = p {
                if par != path {
                    out.push(par.to_path_buf());
                    p = par.parent();
                } else {
                    break;
                }
            }
            out.reverse();
            out
        };

        for item in excludes.clone() {
            let excluded = excluder(&item) || parents(&item).iter().any(|p| excluder(p));
            if excluded {
                continue;
            };
            Self::process(Path::new(&item), &newconf, processed.clone(), logger).with_context(
                || {
                    format!(
                        "Can't process paths {}",
                        excludes
                            .iter()
                            .map(|p| p.to_string_lossy())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                },
            )?;
        }

        let directory_iterator = DirectoryIterator {
            path,
            exclude: Some(&excluder),
        };
        let directories = directory_iterator
            .list()
            .with_context(|| format!("Can't list directory {}", path.display()))?;

        for path in directories {
            // Recurse, passing down the new effective_skip_glob
            Self::process_directory(&path, &newconf, processed.clone(), logger)
                .with_context(|| format!("Can't process directory {}", path.display()))?;
        }

        Ok(())
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

    fn is_inside(root: &Path, child: &Path) -> bool {
        let root = root
            .canonicalize()
            .unwrap_or_else(|_| Path::new(root).to_path_buf());
        let child = child
            .canonicalize()
            .unwrap_or_else(|_| Path::new(child).to_path_buf());

        root.eq(&child) || (child.starts_with(&root) && !root.starts_with(&child))
    }

    // Reads .tmbliss file in the given directory and returns a Vec of globs (ignores comments and empty lines)
    fn read_tmbliss_globs(dir: &Path) -> Vec<String> {
        let tmbliss_path = dir.join(TMBLISS_FILE);
        let file = File::open(&tmbliss_path);
        if let Ok(file) = file {
            let reader = BufReader::new(file);
            reader
                .lines()
                .filter_map(|line| {
                    let line = line.ok()?.trim().to_string();
                    if line.is_empty() || line.starts_with('#') {
                        None
                    } else {
                        Some(line)
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}
