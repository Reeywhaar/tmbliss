use std::{fs, path::Path};

pub struct TestDir {
    pub path: String,
}

impl Drop for TestDir {
    fn drop(&mut self) {
        fs::remove_dir_all(Path::new(&self.path)).unwrap();
    }
}

pub fn join_path(base: &str, path: &str) -> String {
    Path::new(base).join(path).to_str().unwrap().to_string()
}

pub fn path_to_string(path: &Path) -> String {
    path.to_str().unwrap().to_string()
}

pub fn unzip(file: &str, dir: &str) {
    let result = std::process::Command::new("unzip")
        .arg(file)
        .arg("-d")
        .arg(dir)
        .output()
        .unwrap();
    assert!(result.status.success());
}
