use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;

pub struct Git {
    pub path: PathBuf,
}

impl Git {
    /// Lists all files that are ignored by git
    pub fn get_ignores_list(&self) -> Result<Vec<PathBuf>> {
        if !self.path.is_dir() {
            return Err(anyhow::anyhow!("Path is not a directory"));
        }

        let mut gitignore_builder = ignore::gitignore::GitignoreBuilder::new(&self.path);
        gitignore_builder.add(self.path.join(".gitignore"));
        let gitignore = gitignore_builder.build_global();
        if gitignore.1.is_some() {
            return Err(anyhow::anyhow!(gitignore.1.unwrap()));
        }
        let gitignore = gitignore.0;

        let mut ignored: Vec<PathBuf> = vec![];

        fn visitor(
            path: &Path,
            gitignore: &ignore::gitignore::Gitignore,
            ignored: &mut Vec<PathBuf>,
        ) -> Result<()> {
            let is_dir = path.is_dir();
            if path.ends_with(".git") {
                return Ok(());
            }
            if gitignore.matched(path, is_dir).is_ignore() {
                ignored.push(path.canonicalize()?);
                return Ok(());
            }
            if is_dir {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    visitor(&entry.path(), gitignore, ignored)?;
                }
            }
            Ok(())
        }

        visitor(&self.path, &gitignore, &mut ignored)?;

        ignored.sort();
        Ok(ignored)
    }

    /// Checks if a directory is a git service directory (".git")
    pub fn is_git(path: &Path) -> bool {
        path.ends_with(".git")
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
