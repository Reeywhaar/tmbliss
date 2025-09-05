use std::{
    collections::HashMap,
    env,
    fs::{self},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{self, PathBuf},
    process::Command,
};

use anyhow::Context;
use tmbliss::TimeMachine;
use uuid::Uuid;

static TMBLISS_FILE: &str = ".tmbliss";

pub enum FileTreeItem {
    Directory {
        key: String,
        name: String,
        is_excluded: bool,
    },
    File {
        key: String,
        name: String,
        is_excluded: bool,
    },
    Gitignore {
        key: String,
        path: String,
        patterns: Vec<String>,
    },
    TmBliss {
        key: String,
        path: String,
        patterns: Vec<String>,
    },
}

pub struct FileTree {
    path: PathBuf,
    items: Vec<FileTreeItem>,
}

impl Drop for FileTree {
    fn drop(&mut self) {
        if self.path.exists() {
            fs::remove_dir_all(&self.path).unwrap();
        }
    }
}

impl FileTree {
    pub fn new(items: Vec<FileTreeItem>) -> Self {
        Self {
            path: env::temp_dir()
                .join(format!("tmbliss_test_workspace_{}", Uuid::new_v4()))
                .to_path_buf(),
            items,
        }
    }

    pub fn create(&self) -> HashMap<String, PathBuf> {
        if self.path.exists() {
            fs::remove_dir_all(&self.path).unwrap();
        }

        fs::create_dir_all(&self.path).unwrap();

        Command::new("/usr/bin/git")
            .arg("init")
            .current_dir(&self.path)
            .output()
            .expect("Failed to initialize git repository");

        let mut hashmap: HashMap<String, PathBuf> = HashMap::new();

        hashmap.insert("__workspace".to_string(), self.path.clone());

        for item in &self.items {
            let item_path = match item {
                FileTreeItem::Directory { name, .. } => self.path.join(name),
                FileTreeItem::File { name, .. } => self.path.join(name),
                FileTreeItem::Gitignore { path, .. } => self.path.join(path).join(".gitignore"),
                FileTreeItem::TmBliss { path, .. } => self.path.join(path).join(TMBLISS_FILE),
            };
            let item_path = path::absolute(self.path.join(item_path)).unwrap();
            let item_key = match item {
                FileTreeItem::Directory { key, .. } => key,
                FileTreeItem::File { key, .. } => key,
                FileTreeItem::Gitignore { key, .. } => key,
                FileTreeItem::TmBliss { key, .. } => key,
            };

            hashmap.insert(item_key.clone(), item_path.clone());

            let item_dir = item_path
                .parent()
                .with_context(|| format!("Failed to get parent directory for {:?}", item_path))
                .unwrap();

            fs::create_dir_all(item_dir)
                .with_context(|| format!("Failed to create directory {:?}", item_dir))
                .unwrap();

            match item {
                FileTreeItem::Directory { is_excluded, .. } => {
                    fs::create_dir_all(&item_path).unwrap();

                    if *is_excluded {
                        TimeMachine::add_exclusion(&item_path)
                            .with_context(|| format!("Failed to add exclusion for {:?}", item_path))
                            .unwrap()
                    }
                }
                FileTreeItem::File { is_excluded, .. } => {
                    let file = fs::File::create(&item_path).unwrap();
                    let mut perms = file.metadata().unwrap().permissions();
                    perms.set_mode(0o777);
                    fs::set_permissions(&item_path, perms).unwrap();

                    if *is_excluded {
                        TimeMachine::add_exclusion(&item_path)
                            .with_context(|| format!("Failed to add exclusion for {:?}", item_path))
                            .unwrap()
                    }
                }
                FileTreeItem::Gitignore { patterns, .. } => {
                    let mut gitignore = fs::File::create(&item_path).unwrap();
                    let mut perms = gitignore.metadata().unwrap().permissions();
                    perms.set_mode(0o777);
                    fs::set_permissions(&item_path, perms).unwrap();

                    let content = patterns.join("\n");
                    gitignore.write_all(content.as_bytes()).unwrap();
                    gitignore.write_all(b"\n").unwrap();
                }
                FileTreeItem::TmBliss { patterns, .. } => {
                    let tmbliss_path = self.path.join(TMBLISS_FILE);
                    let mut tmbliss = fs::File::create(&tmbliss_path).unwrap();
                    let mut perms = tmbliss.metadata().unwrap().permissions();
                    perms.set_mode(0o777);
                    fs::set_permissions(&tmbliss_path, perms).unwrap();

                    let content = patterns.join("\n");
                    tmbliss.write_all(content.as_bytes()).unwrap();
                    tmbliss.write_all(b"\n").unwrap();
                }
            }
        }

        hashmap
    }
}
