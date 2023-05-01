use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context, Result};

pub struct RecursiveDirectoryIterator<'a> {
    pub path: String,
    pub op: &'a dyn Fn(&str) -> Result<()>,
}

impl<'a> RecursiveDirectoryIterator<'a> {
    pub fn iterate(&self) -> Result<()> {
        for entry in
            fs::read_dir(&self.path).with_context(|| format!("Can't read dir {}", self.path))?
        {
            let entry = entry?;
            let path = self.try_canonicalize(entry.path());
            let pathstr = path
                .to_str()
                .ok_or(anyhow!("Could not convert path to string"))?;

            (self.op)(pathstr).with_context(|| format!("Can't process path {}", pathstr))?;

            if path.is_dir() {
                let iterator = RecursiveDirectoryIterator {
                    path: pathstr.to_string(),
                    op: self.op,
                };
                iterator.iterate()?;
            }
        }

        Ok(())
    }

    fn try_canonicalize(&self, path: PathBuf) -> PathBuf {
        path.canonicalize().unwrap_or(path)
    }
}
