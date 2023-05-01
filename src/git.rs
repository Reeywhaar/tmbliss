use std::{fs, path::Path, process::Command};

use anyhow::{anyhow, Context, Result};

pub struct Git {
    pub path: String,
}

impl Git {
    pub fn get_ignores_list(&self) -> Result<Vec<String>> {
        let out = Command::new("/usr/bin/git")
            .current_dir(self.path.as_str())
            .arg("ls-files")
            .arg("--directory") // Do not list contained files of ignored directories
            .arg("--exclude-standard") // Also use `.git/info/exclude` and global `.gitignore` files
            .arg("--ignored") // List ignored files
            .arg("--others") // Include untracked files
            .arg("-z") // Do not encode "unusual" characters (e.g. "ä" is normally listed as "\303\244")
            .output()?;

        let output =
            String::from_utf8(out.stdout).with_context(|| "Cannot create string from output")?;

        let mut output: Vec<String> = output
            .split('\0')
            .filter(|p| !p.is_empty())
            .filter(|p| !p.starts_with("../"))
            .map(|v| self.path_to_string(v))
            .collect::<Result<Vec<String>>>()?;

        output.sort();

        let output = self.remove_roots(output);

        Ok(output)
    }

    pub fn path_to_string(&self, path: &str) -> Result<String> {
        Ok(fs::canonicalize(Path::new(self.path.as_str()).join(path))?
            .to_str()
            .ok_or(anyhow!("Could not convert path to string"))?
            .to_string())
    }

    pub fn is_git(path: &str) -> bool {
        path.ends_with("/.git")
    }

    fn remove_roots(&self, list: Vec<String>) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();

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
    use uuid::Uuid;

    use crate::test_utils::{join_path, path_to_string, unzip, TestDir};

    use super::*;

    #[test]
    fn it_lists_ignored_files() {
        let cwd = &path_to_string(&fs::canonicalize("./").unwrap());

        let zip = join_path(cwd, "test_assets/test_dir.zip");
        let unzipdir = TestDir {
            path: join_path(cwd, &format!("test_assets_{}", Uuid::new_v4())),
        };
        let dir = join_path(&unzipdir.path, "test_dir/test_repo");

        unzip(&zip, &unzipdir.path);

        let git = Git { path: dir.clone() };

        let mut list = git.get_ignores_list().unwrap();
        list.sort();

        let mut result = vec![
            join_path(&dir, ".excluded_glob"),
            join_path(&dir, "excluded_path"),
            join_path(&dir, "not_excluded_path"),
            join_path(&dir, "nested_dir/excluded_file.txt"),
            join_path(&dir, "nested_dir_with_single_file/excluded_file.txt"),
        ];
        result.sort();

        assert_eq!(list.join("\r\n"), result.join("\r\n"));
    }

    #[test]
    fn it_check_if_directory_is_git() {
        assert!(Git::is_git("./.git"));
    }

    #[test]
    fn it_check_if_directory_is_not_git() {
        assert!(!Git::is_git("./tests"));
    }
}
