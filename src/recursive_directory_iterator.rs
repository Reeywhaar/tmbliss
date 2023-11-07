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

            if !path.is_symlink() && path.is_dir() {
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

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use uuid::Uuid;

    use crate::test_utils::{join_path, path_to_string, unzip, TestDir};

    use super::*;

    #[test]
    fn it_works_recursively() {
        let cwd = &path_to_string(&fs::canonicalize("./").unwrap());

        let zip = join_path(cwd, "test_assets/test_dir.zip");
        let unzipdir = TestDir {
            path: join_path(cwd, &format!("test_assets_{}", Uuid::new_v4())),
        };
        let dir = join_path(&unzipdir.path, "test_dir/test_repo");

        unzip(&zip, &unzipdir.path);

        let paths = Rc::new(RefCell::new(Vec::<String>::new()));

        let iterator = RecursiveDirectoryIterator {
            path: dir,
            op: &|path| {
                let paths = paths.clone();
                let mut paths = paths.try_borrow_mut().unwrap();
                paths.push(path.to_string());
                Ok(())
            },
        };

        iterator.iterate().unwrap();

        assert_eq!(paths.clone().try_borrow().unwrap().len(), 38);
    }
}
