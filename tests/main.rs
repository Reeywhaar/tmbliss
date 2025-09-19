extern crate tmbliss;

#[path = "../src/filetree.rs"]
mod filetree;

use crate::filetree::{FileTree, FileTreeItem};

use std::env::current_dir;
use test_case::test_case;

use tmbliss::{Command, TMBliss, TimeMachine};

#[test_case("sec*.txt" ; "sec*.txt")]
#[test_case("/sec*.txt" ; "/sec*.txt")]
#[test_case("/secret.txt" ; "/secret.txt")]
#[test_case("secret.txt" ; "secret.txt")]
fn test_tmbliss_glob_exclusion(case: &str) {
    let tree = FileTree::new(vec![
        FileTreeItem::Gitignore {
            key: "gitignore".to_string(),
            path: "".to_string(),
            patterns: vec!["**/devfile.txt".to_string(), "secret.txt".to_string()],
        },
        FileTreeItem::TmBliss {
            key: "tmbliss".to_string(),
            path: "".to_string(),
            patterns: vec![case.to_string(), "sub".to_string()],
        },
        FileTreeItem::File {
            key: "devfile".to_string(),
            name: "devfile.txt".to_string(),
            is_excluded: false,
        },
        FileTreeItem::File {
            key: "secret".to_string(),
            name: "secret.txt".to_string(),
            is_excluded: false,
        },
        FileTreeItem::File {
            key: "sub/devfile".to_string(),
            name: "sub/devfile.txt".to_string(),
            is_excluded: false,
        },
    ]);

    let hmap = tree.create();

    let command = Command::Run {
        path: vec![hmap
            .get("__workspace")
            .unwrap()
            .to_string_lossy()
            .to_string()],
        dry_run: false,
        allowlist_glob: vec![],
        allowlist_path: vec![],
        skip_glob: vec![],
        skip_path: vec![],
        skip_errors: false,
        exclude_path: vec![],
    };
    let result = TMBliss::run(command);
    result.unwrap();

    assert!(
        TimeMachine::is_excluded_deep(hmap.get("devfile").unwrap()).unwrap(),
        "Devfile is not excluded"
    );
    assert!(
        !TimeMachine::is_excluded_deep(hmap.get("secret").unwrap()).unwrap(),
        "Secret file is excluded, but it should not be"
    );
    assert!(
        !TimeMachine::is_excluded_deep(hmap.get("sub/devfile").unwrap()).unwrap(),
        "Sub/devfile is excluded, but it should not be"
    );
}

#[test]
fn test_tmbliss_glob_exclusion_2() {
    let tree = FileTree::new(vec![
        FileTreeItem::Gitignore {
            key: "gitignore".to_string(),
            path: "".to_string(),
            patterns: vec!["**/devfile.txt".to_string(), "sub".to_string()],
        },
        FileTreeItem::TmBliss {
            key: "tmbliss".to_string(),
            path: "".to_string(),
            patterns: vec!["sub".to_string()],
        },
        FileTreeItem::File {
            key: "devfile".to_string(),
            name: "devfile.txt".to_string(),
            is_excluded: false,
        },
        FileTreeItem::Directory {
            key: "sub".to_string(),
            name: "sub".to_string(),
            is_excluded: false,
        },
        FileTreeItem::File {
            key: "sub/devfile".to_string(),
            name: "sub/devfile.txt".to_string(),
            is_excluded: false,
        },
    ]);

    let hmap = tree.create();

    let command = Command::Run {
        path: vec![hmap
            .get("__workspace")
            .unwrap()
            .to_string_lossy()
            .to_string()],
        dry_run: false,
        allowlist_glob: vec![],
        allowlist_path: vec![],
        skip_glob: vec![],
        skip_path: vec![],
        skip_errors: false,
        exclude_path: vec![],
    };
    let result = TMBliss::run(command);
    result.unwrap();

    assert!(
        TimeMachine::is_excluded_deep(hmap.get("devfile").unwrap()).unwrap(),
        "Devfile is not excluded"
    );
    assert!(
        !TimeMachine::is_excluded_deep(hmap.get("sub").unwrap()).unwrap(),
        "Sub directory is excluded, but it should not be"
    );
    assert!(
        !TimeMachine::is_excluded_deep(hmap.get("sub/devfile").unwrap()).unwrap(),
        "Sub/devfile is excluded, but it should not be"
    );
}

#[test]
fn test_run() {
    let filetree = FileTree::new_test_repo();

    let fmap = filetree.create();
    let workspace = fmap.get("__workspace").unwrap();

    let excluded_path = fmap.get("excluded_path").unwrap();
    let not_excluded_glob = fmap.get(".excluded_glob").unwrap();
    let not_excluded_dir = fmap.get("not_excluded_path").unwrap();

    let command = Command::Run {
        path: vec![workspace.to_string_lossy().into_owned()],
        dry_run: false,
        allowlist_glob: vec![
            "**/.excluded_glob".to_string(),
            ".excluded_glob.*".to_string(),
        ],
        allowlist_path: vec![not_excluded_dir.to_string_lossy().into_owned()],
        skip_glob: vec![],
        skip_path: vec![],
        skip_errors: false,
        exclude_path: vec![],
    };
    let result = TMBliss::run(command);

    result.unwrap();
    assert!(TimeMachine::is_excluded(excluded_path).unwrap());
    assert!(!TimeMachine::is_excluded(not_excluded_glob).unwrap());
    assert!(!TimeMachine::is_excluded(not_excluded_dir).unwrap());
}

#[test]
fn test_exclude_paths() {
    let filetree = FileTree::new_test_repo();

    let fmap = filetree.create();

    let file = fmap.get("path_that_should_be_excluded.txt").unwrap();

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
    assert!(TimeMachine::is_excluded(file).unwrap());
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
    let filetree = FileTree::new_test_repo();

    let fmap = filetree.create();

    let dir = fmap.get("__workspace").unwrap();

    let excluded_path = dir.join("excluded_path");
    let not_excluded_glob = dir.join(".excluded_glob");
    let not_excluded_path = dir.join("not_excluded_path");

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
