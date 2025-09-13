use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

pub struct RecursiveDirectoryIterator<'a> {
    pub path: &'a Path,
    pub op: &'a dyn for<'b> Fn(&'b PathBuf) -> Result<bool>,
}

impl RecursiveDirectoryIterator<'_> {
    pub fn iterate(&self) -> Result<()> {
        for entry in fs::read_dir(self.path)
            .with_context(|| format!("Can't read dir {}", self.path.display()))?
        {
            let entry = entry?.path();

            let should_continue = (self.op)(&entry)
                .with_context(|| format!("Can't process path {}", entry.display()))?;

            if should_continue && !entry.is_symlink() && entry.is_dir() {
                let iterator = RecursiveDirectoryIterator {
                    path: &entry,
                    op: self.op,
                };
                iterator.iterate()?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, env::current_dir, rc::Rc};

    use crate::test_utils::{unzip, TestDir};

    use super::*;

    #[test]
    fn it_works_recursively() {
        let workspace = TestDir::new();

        let zip = current_dir().unwrap().join("test_assets/test_dir.zip");
        let dir = workspace.path().join("test_dir/test_repo");

        unzip(&zip, workspace.path());

        let paths = Rc::new(RefCell::new(Vec::<PathBuf>::new()));

        let iterator = RecursiveDirectoryIterator {
            path: &dir,
            op: &|path| {
                let paths = paths.clone();
                let mut paths = paths.try_borrow_mut().unwrap();
                paths.push(path.clone());
                Ok(true)
            },
        };

        iterator.iterate().unwrap();

        assert_eq!(paths.clone().try_borrow().unwrap().len(), 38);
    }
}
