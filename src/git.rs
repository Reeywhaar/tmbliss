use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};

pub struct Git {
    pub path: PathBuf,
}

impl Git {
    /// Lists all files that are ignored by git
    pub fn get_ignores_list(&self) -> Result<Vec<PathBuf>> {
        let out = Command::new("/usr/bin/git")
            .current_dir(&self.path)
            .arg("ls-files")
            .arg("--directory") // Do not list contained files of ignored directories
            .arg("--exclude-standard") // Also use `.git/info/exclude` and global `.gitignore` files
            .arg("--ignored") // List ignored files
            .arg("--others") // Include untracked files
            .arg("-z") // Do not encode "unusual" characters (e.g. "ä" is normally listed as "\303\244")
            .output()?;

        let output =
            String::from_utf8(out.stdout).with_context(|| "Cannot create string from output")?;

        let mut output: Vec<PathBuf> = output
            .split('\0')
            .filter(|p| !p.is_empty())
            .filter(|p| !p.starts_with("../"))
            .map(|v| self.join_path(v))
            .collect::<Result<Vec<PathBuf>>>()?;

        output.sort();

        let output = self
            .remove_roots(output)
            .iter()
            .map(|p| p.canonicalize().unwrap())
            .collect::<Vec<PathBuf>>();

        Ok(output)
    }

    /// Checks if a directory is a git service directory (".git")
    pub fn is_git(path: &Path) -> bool {
        path.ends_with(".git")
    }

    fn join_path(&self, path: &str) -> Result<PathBuf> {
        Ok(self.path.join(path))
    }

    fn remove_roots(&self, list: Vec<PathBuf>) -> Vec<PathBuf> {
        let mut output: Vec<PathBuf> = Vec::new();

        'outer: for i in 0..list.len() {
            let item = &list[i];
            let remain = list[i + 1..].to_vec();
            for subitem in remain {
                if subitem.starts_with(item) {
                    continue 'outer;
                }
            }
            output.push(item.clone());
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;

    use crate::test_utils::{unzip, TestDir};

    use super::*;

    #[test]
    fn it_lists_ignored_files() {
        let workspace = TestDir::new();

        let zip = current_dir().unwrap().join("test_assets/test_dir.zip");
        let dir = workspace.join("test_dir/test_repo");

        unzip(&zip, workspace.path());

        let git = Git { path: dir.clone() };

        let mut list = git
            .get_ignores_list()
            .unwrap()
            .iter()
            .map(|p| p.to_str().unwrap().to_string())
            .collect::<Vec<String>>();
        list.sort();

        let mut result = [
            dir.join(".excluded_glob"),
            dir.join("excluded_path"),
            dir.join("not_excluded_path"),
            dir.join("nested_dir/excluded_file.txt"),
            dir.join("nested_dir_with_single_file/excluded_file.txt"),
        ]
        .map(|p| p.canonicalize().unwrap().to_str().unwrap().to_string())
        .to_vec();
        result.sort();

        assert_eq!(list, result);
    }

    #[test]
    fn it_check_if_directory_is_git() {
        assert!(Git::is_git(&current_dir().unwrap().join(".git")));
    }

    #[test]
    fn it_check_if_directory_is_not_git() {
        assert!(!Git::is_git(&current_dir().unwrap().join("tests")));
        assert!(!Git::is_git(&current_dir().unwrap().join("tests.git")));
    }
}
