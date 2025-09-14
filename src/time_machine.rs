use std::{error::Error, fmt::Display, path::Path, process::Command};

use anyhow::{anyhow, Context, Result};
use regex::Regex;

pub struct TimeMachine {}

impl TimeMachine {
    pub fn add_exclusion(path: &Path) -> Result<(), TimeMachineError> {
        let mut binding = Command::new("/usr/bin/tmutil");
        let command = binding.arg("addexclusion").arg(path);

        let output = command
            .output()
            .with_context(|| "Failed to execute tmutil command")
            .map_err(|e| TimeMachineError::Unknown(e.to_string()))?;

        if !output.status.success() {
            let output = output.stderr;

            let output_string = String::from_utf8_lossy(&output);

            return Err(Self::parse_error(&output_string));
        }

        Ok(())
    }

    pub fn remove_exclusion(path: &Path) -> Result<(), TimeMachineError> {
        let mut binding = Command::new("/usr/bin/tmutil");
        let command = binding.arg("removeexclusion").arg(path);

        let output = command
            .output()
            .map_err(|e| TimeMachineError::Unknown(e.to_string()))?;

        if !output.status.success() {
            let output = output.stderr;

            let output_string = String::from_utf8_lossy(&output);

            return Err(Self::parse_error(&output_string));
        }

        Ok(())
    }

    pub fn is_excluded(path: &Path) -> Result<bool> {
        let result = Command::new("/usr/bin/xattr")
            .arg(path)
            .output()
            .with_context(|| "Failed to execute xattr command")?;

        if !result.status.success() {
            return Err(anyhow!(String::from_utf8_lossy(&result.stderr).to_string()));
        }

        Ok(String::from_utf8(result.stdout)
            .with_context(|| "Cannot convert output to string")?
            .lines()
            .any(|line| line == "com.apple.metadata:com_apple_backup_excludeItem"))
    }

    fn parse_status_code(output: &str) -> isize {
        let re = Regex::new(r"Error \((.*)\) while attempting").unwrap();
        if let Some(captures) = re.captures(output) {
            if let Some(capture) = captures.get(1) {
                return capture.as_str().parse::<isize>().unwrap_or(0);
            }
        }
        0
    }

    fn parse_error(output: &str) -> TimeMachineError {
        let status = Self::parse_status_code(output);
        match status {
            -43 => TimeMachineError::FileNotFound,
            100002 => TimeMachineError::FileNotFound,
            -50 => TimeMachineError::FileInaccessible,
            -20 => TimeMachineError::FileInaccessible,
            status => TimeMachineError::Unknown(format!("Unknown error with status {}", status)),
        }
    }
}

#[derive(Debug)]
pub enum TimeMachineError {
    FileNotFound,
    FileInaccessible,
    Unknown(String),
}

impl Error for TimeMachineError {
    fn description(&self) -> &str {
        match &self {
            TimeMachineError::FileNotFound => "File not found",
            TimeMachineError::FileInaccessible => "File inaccessible",
            TimeMachineError::Unknown(_description) => "Unknown error",
        }
    }
}

impl Display for TimeMachineError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            TimeMachineError::FileNotFound => write!(f, "File not found"),
            TimeMachineError::FileInaccessible => write!(f, "File inaccessible"),
            TimeMachineError::Unknown(description) => write!(f, "Unknown error: {}", description),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use uuid::Uuid;

    use crate::test_utils::TestDir;

    use super::*;

    use std::fs::{self, File};

    #[test]
    fn it_sets_xattr() {
        let workspace = TestDir::new();
        let pathstr = workspace.join(format!("./text-{}.txt", Uuid::new_v4()));
        File::create(pathstr.clone()).unwrap();
        TimeMachine::add_exclusion(&pathstr).unwrap();

        assert!(TimeMachine::is_excluded(&pathstr).unwrap());

        fs::remove_file(pathstr).unwrap();
    }

    #[test]
    fn it_throws_inaccessible_if_cant_set_xattr() {
        let path = Path::new("./test_assets/root_file.txt");
        let result = TimeMachine::add_exclusion(path);

        assert!(!TimeMachine::is_excluded(path).unwrap());
        assert_matches!(result, Err(TimeMachineError::FileInaccessible));
    }

    #[test]
    fn it_throws_not_found_if_cant_set_xattr() {
        let path = Path::new("./test_assets/not_a_file.txt");
        let result = TimeMachine::add_exclusion(path);

        assert_matches!(result, Err(TimeMachineError::FileNotFound));
    }

    #[test]
    fn it_removes_xattr() {
        let cwd = TestDir::new();
        let pathstr = cwd.join(format!("./text-{}.txt", Uuid::new_v4()));
        File::create(pathstr.clone()).unwrap();
        TimeMachine::add_exclusion(&pathstr).unwrap();

        assert!(TimeMachine::is_excluded(&pathstr).unwrap());

        TimeMachine::remove_exclusion(&pathstr).unwrap();

        assert!(!TimeMachine::is_excluded(&pathstr).unwrap());

        fs::remove_file(pathstr).unwrap();
    }

    #[test]
    fn it_throws_inaccessible_if_cant_remove_xattr() {
        let path = Path::new("./test_assets/root_file_excluded.txt");
        let result = TimeMachine::remove_exclusion(path);

        assert!(TimeMachine::is_excluded(path).unwrap());
        assert_matches!(result, Err(TimeMachineError::FileInaccessible));
    }

    #[test]
    fn it_throws_not_found_if_cant_remove_xattr() {
        let path = Path::new("./test_assets/not_a_file.txt");
        let result = TimeMachine::remove_exclusion(path);

        assert_matches!(result, Err(TimeMachineError::FileNotFound));
    }
}
