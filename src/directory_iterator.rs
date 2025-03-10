use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};

pub struct DirectoryIterator<'a> {
    pub path: &'a str,
    pub exclude: Option<&'a dyn Fn(&str) -> bool>,
}

impl DirectoryIterator<'_> {
    pub fn list(&self) -> Result<Vec<String>> {
        let mut paths = Vec::new();

        let path = Path::new(&self.path);

        if !path.is_dir() {
            return Err(anyhow!("Path is not a directory"));
        }

        for entry in
            fs::read_dir(self.path).with_context(|| format!("Can't read dir {}", self.path))?
        {
            let entry = entry?;
            let path = self.try_canonicalize(entry.path());
            let pathstr = path
                .to_str()
                .ok_or(anyhow!("Could not convert path to string"))?;

            if let Some(excluder) = self.exclude {
                if excluder(pathstr) {
                    continue;
                }
            }

            if !path.is_symlink() && path.is_dir() {
                paths.push(pathstr.to_string());
            }
        }

        Ok(paths)
    }

    fn try_canonicalize(&self, path: PathBuf) -> PathBuf {
        path.canonicalize().unwrap_or(path)
    }
}
