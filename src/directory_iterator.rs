use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};

pub struct DirectoryIterator<'a> {
    pub path: &'a Path,
    pub exclude: Option<&'a dyn for<'b> Fn(&'b Path) -> bool>,
}

impl DirectoryIterator<'_> {
    pub fn list(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        if !self.path.is_dir() {
            return Err(anyhow!("Path is not a directory"));
        }

        let entries = fs::read_dir(self.path)
            .with_context(|| format!("Can't read dir {}", self.path.display()))?;

        for entry in entries {
            let entry = entry?.path();

            if let Some(excluder) = self.exclude {
                if excluder(&entry) {
                    continue;
                }
            }

            if !entry.is_symlink() && entry.is_dir() {
                paths.push(entry.to_path_buf());
            }
        }

        Ok(paths)
    }
}
