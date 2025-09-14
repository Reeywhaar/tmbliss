extern crate tmbliss;

#[path = "../src/test_utils.rs"]
mod test_utils;

use crate::test_utils::{unzip, TestDir};

use std::env::current_dir;

use tmbliss::{Command, TMBliss, TimeMachine};

#[test]
fn test_run() {
    let workspace = TestDir::new();

    let zip = current_dir().unwrap().join("test_assets/test_dir.zip");

    let excluded_path = workspace.join("test_dir/test_repo/excluded_path");
    let not_excluded_glob = workspace.join("test_dir/test_repo/.excluded_glob");
    let not_excluded_dir = workspace.join("test_dir/test_repo/not_excluded_path");

    unzip(&zip, workspace.path());

    let command = Command::Run {
        path: vec![workspace.path().to_string_lossy().into_owned()],
        dry_run: false,
        allowlist_glob: vec![
            "**/.excluded_glob".to_string(),
            ".excluded_glob.*".to_string(),
        ],
        allowlist_path: vec![not_excluded_dir.to_string_lossy().into_owned()],
        skip_glob: vec![],
        skip_path: vec![],
        skip_errors: true,
        exclude_path: vec![],
    };
    let result = TMBliss::run(command);

    result.unwrap();
    assert!(TimeMachine::is_excluded(&excluded_path).unwrap());
    assert!(!TimeMachine::is_excluded(&not_excluded_glob).unwrap());
    assert!(!TimeMachine::is_excluded(&not_excluded_dir).unwrap());
}

#[test]
fn test_exclude_paths() {
    let workspace = TestDir::new();

    let zip = current_dir().unwrap().join("test_assets/test_dir.zip");
    let file = workspace
        .path()
        .join("test_dir/test_repo/path_that_should_be_excluded.txt");

    unzip(&zip, workspace.path());

    let command = Command::Run {
        path: vec![],
        dry_run: false,
        allowlist_glob: vec![],
        allowlist_path: vec![],
        skip_glob: vec![],
        skip_path: vec![],
        skip_errors: true,
        exclude_path: vec![file.to_string_lossy().into_owned()],
    };
    let result = TMBliss::run(command);

    result.unwrap();
    assert!(TimeMachine::is_excluded(&file).unwrap());
}

#[test]
fn test_skip_errors() {
    let cwd = current_dir().unwrap();

    let dir = cwd.join("test_assets");

    let root_file = dir.join("root_file.txt");

    {
        let command = Command::Run {
            path: vec![dir.to_string_lossy().into_owned()],
            dry_run: false,
            allowlist_glob: vec!["**/.DS_Store".to_string()],
            allowlist_path: vec![],
            skip_glob: vec![],
            skip_path: vec![],
            skip_errors: true,
            exclude_path: vec![root_file.to_string_lossy().into_owned()],
        };
        let result = TMBliss::run(command);

        result.unwrap();
    }

    {
        let command = Command::Run {
            path: vec![dir.to_string_lossy().into_owned()],
            dry_run: false,
            allowlist_glob: vec!["**/.DS_Store".to_string()],
            allowlist_path: vec![],
            skip_glob: vec![],
            skip_path: vec![],
            skip_errors: false,
            exclude_path: vec![root_file.to_string_lossy().into_owned()],
        };
        let result = TMBliss::run(command);

        assert_eq!(result.unwrap_err().to_string(), "File inaccessible");
    }
}

#[test]
fn test_reset() {
    let workspace = TestDir::new();

    let zip = current_dir().unwrap().join("test_assets/test_dir.zip");
    let dir = workspace.join("test_dir");

    let excluded_path = dir.join("test_repo/excluded_path");
    let not_excluded_glob = dir.join("test_repo/.excluded_glob");
    let not_excluded_path = dir.join("test_repo/not_excluded_path");

    unzip(&zip, workspace.path());

    TMBliss::run(Command::Run {
        path: vec![dir.to_string_lossy().into_owned()],
        dry_run: false,
        allowlist_glob: vec![],
        allowlist_path: vec![],
        skip_glob: vec![],
        skip_path: vec![],
        skip_errors: true,
        exclude_path: vec![],
    })
    .unwrap();

    assert!(TimeMachine::is_excluded(&excluded_path).unwrap());
    assert!(TimeMachine::is_excluded(&not_excluded_glob).unwrap());
    assert!(TimeMachine::is_excluded(&not_excluded_path).unwrap());

    TMBliss::run(Command::Reset {
        path: dir.to_string_lossy().into_owned(),
        dry_run: false,
        allowlist_glob: vec!["**/.excluded_glob".to_string()],
        allowlist_path: vec![not_excluded_path.to_string_lossy().into_owned()],
    })
    .unwrap();

    assert!(!TimeMachine::is_excluded(&excluded_path).unwrap());
    assert!(TimeMachine::is_excluded(&not_excluded_glob).unwrap());
    assert!(TimeMachine::is_excluded(&not_excluded_path).unwrap());
}
