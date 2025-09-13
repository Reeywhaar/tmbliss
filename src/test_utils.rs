use std::{
    env::temp_dir,
    fs,
    path::{Path, PathBuf},
};

use uuid::Uuid;

pub struct TestDir {
    path: PathBuf,
}

impl TestDir {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let s = Self {
            path: temp_dir().join(format!("test_assets_{}", Uuid::new_v4())),
        };
        fs::create_dir_all(&s.path).unwrap();
        s
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.path.join(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).unwrap();
    }
}

pub fn unzip(file: &Path, dir: &Path) {
    let result = std::process::Command::new("unzip")
        .arg(file)
        .arg("-d")
        .arg(dir)
        .output()
        .unwrap();
    assert!(result.status.success());
}
