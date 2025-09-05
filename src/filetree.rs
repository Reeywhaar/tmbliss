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

pub enum FileTreeItem {
    File(String, String, bool),
    Gitignore(String, String, Vec<String>),
    TmBliss(String, String, Vec<String>),
}

pub struct FileTree<'a> {
    pub path: &'a str,
    pub items: Vec<FileTreeItem>,
}

impl<'a> FileTree<'a> {
    pub fn create(&self) -> HashMap<String, PathBuf> {
        let path = env::temp_dir().join(self.path);

        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }

        fs::create_dir_all(&path).unwrap();

        Command::new("/usr/bin/git")
            .arg("init")
            .current_dir(&path)
            .output()
            .expect("Failed to initialize git repository");

        let mut hashmap: HashMap<String, PathBuf> = HashMap::new();

        hashmap.insert("__workspace".to_string(), path.clone());

        for item in &self.items {
            let item_path = match item {
                FileTreeItem::File(_key, name, _) => path.join(name),
                FileTreeItem::Gitignore(_key, cpath, _) => path.join(cpath).join(".gitignore"),
                FileTreeItem::TmBliss(_key, cpath, _) => path.join(cpath).join(".tmbliss"),
            };
            let item_path = path::absolute(path.join(item_path)).unwrap();
            let item_key = match item {
                FileTreeItem::File(key, _, _) => key,
                FileTreeItem::Gitignore(key, _, _) => key,
                FileTreeItem::TmBliss(key, _, _) => key,
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
                FileTreeItem::File(_key, _name, is_excluded) => {
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
                FileTreeItem::Gitignore(_key, _path, patterns) => {
                    let mut gitignore = fs::File::create(&item_path).unwrap();
                    let mut perms = gitignore.metadata().unwrap().permissions();
                    perms.set_mode(0o777);
                    fs::set_permissions(&item_path, perms).unwrap();

                    let content = patterns.join("\n");
                    gitignore.write_all(content.as_bytes()).unwrap();
                    gitignore.write_all(b"\n").unwrap();
                }
                FileTreeItem::TmBliss(_key, _path, patterns) => {
                    let tmbliss_path = path.join(".tmbliss");
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
