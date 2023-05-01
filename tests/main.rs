extern crate tmbliss;

#[path = "../src/test_utils.rs"]
mod test_utils;

use crate::test_utils::{join_path, path_to_string, unzip, TestDir};

use std::fs;

use tmbliss::{Command, TMBliss, TimeMachine};
use uuid::Uuid;

#[test]
fn test_run() {
    let cwd = &path_to_string(&fs::canonicalize("./").unwrap());

    let zip = join_path(cwd, "test_assets/test_dir.zip");
    let unzipdir = TestDir {
        path: join_path(cwd, &format!("test_assets_{}", Uuid::new_v4())),
    };
    let dir = join_path(&unzipdir.path, "test_dir");

    let excluded_path = join_path(&dir, "test_repo/excluded_path");
    let not_excluded_glob = join_path(&dir, "test_repo/.excluded_glob");
    let not_excluded_dir = join_path(&dir, "test_repo/not_excluded_path");

    unzip(&zip, &unzipdir.path);

    let command = Command::Run {
        path: [dir].to_vec(),
        dry_run: false,
        whitelist_glob: vec![
            "**/.excluded_glob".to_string(),
            ".excluded_glob.*".to_string(),
        ],
        whitelist_path: vec![not_excluded_dir.clone()],
        skip_glob: vec![],
        skip_path: vec![],
    };
    let result = TMBliss::run(command);

    assert!(result.is_ok());
    assert!(TimeMachine::is_excluded(&excluded_path));
    assert!(!TimeMachine::is_excluded(&not_excluded_glob));
    assert!(!TimeMachine::is_excluded(&not_excluded_dir));
}

#[test]
fn test_reset() {
    let cwd = &path_to_string(&fs::canonicalize("./").unwrap());

    let zip = join_path(cwd, "test_assets/test_dir.zip");
    let unzipdir = TestDir {
        path: join_path(cwd, &format!("test_assets_{}", Uuid::new_v4())),
    };
    let dir = join_path(&unzipdir.path, "test_dir");

    let excluded_path = join_path(&dir, "test_repo/excluded_path");
    let not_excluded_glob = join_path(&dir, "test_repo/.excluded_glob");
    let not_excluded_path = join_path(&dir, "test_repo/not_excluded_path");

    unzip(&zip, &unzipdir.path);

    TMBliss::run(Command::Run {
        path: [dir.clone()].to_vec(),
        dry_run: false,
        whitelist_glob: vec![],
        whitelist_path: vec![],
        skip_glob: vec![],
        skip_path: vec![],
    })
    .unwrap();

    assert!(TimeMachine::is_excluded(&excluded_path));
    assert!(TimeMachine::is_excluded(&not_excluded_glob));
    assert!(TimeMachine::is_excluded(&not_excluded_path));

    TMBliss::run(Command::Reset {
        path: dir,
        dry_run: false,
        whitelist_glob: vec!["**/.excluded_glob".to_string()],
        whitelist_path: vec![not_excluded_path.clone()],
    })
    .unwrap();

    assert!(!TimeMachine::is_excluded(&excluded_path));
    assert!(TimeMachine::is_excluded(&not_excluded_glob));
    assert!(TimeMachine::is_excluded(&not_excluded_path));
}
