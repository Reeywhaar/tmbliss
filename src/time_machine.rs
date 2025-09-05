use std::{error::Error, fmt::Display, path::Path};

use anyhow::Result;

use crate::constants::TMUTIL_ATTR;

pub struct TimeMachine {}

impl TimeMachine {
    pub fn add_exclusion(path: &Path) -> Result<(), TimeMachineError> {
        xattr::set(path, TMUTIL_ATTR, b"1").map_err(Self::parse_error)
    }

    pub fn remove_exclusion(path: &Path) -> Result<(), TimeMachineError> {
        xattr::remove(path, TMUTIL_ATTR).map_err(Self::parse_error)
    }

    pub fn is_excluded(path: &Path) -> Result<bool, TimeMachineError> {
        Ok(xattr::get(path, TMUTIL_ATTR)
            .map_err(Self::parse_error)?
            .is_some())
    }

    pub fn is_excluded_deep(path: &Path) -> Result<bool, TimeMachineError> {
        let mut p = path.to_path_buf();
        loop {
            if Self::is_excluded(&p)? {
                return Ok(true);
            }
            let parent = p.parent();
            if parent.is_none() {
                break;
            }
            p = parent.unwrap().to_path_buf();
        }
        Ok(false)
    }

    fn parse_error(err: std::io::Error) -> TimeMachineError {
        match err.raw_os_error() {
            Some(2) => TimeMachineError::FileNotFound(Some(Box::new(err))),
            Some(13) => TimeMachineError::FileInaccessible(Some(Box::new(err))),
            status => TimeMachineError::Unknown(
                match status {
                    Some(code) => format!("Error with status code {}", code),
                    None => "Error with unknown status code".to_string(),
                },
                Some(Box::new(err)),
            ),
        }
    }
}

#[derive(Debug)]
pub enum TimeMachineError {
    FileNotFound(Option<Box<std::io::Error>>),
    FileInaccessible(Option<Box<std::io::Error>>),
    Unknown(String, Option<Box<std::io::Error>>),
}

impl Error for TimeMachineError {
    fn description(&self) -> &str {
        match &self {
            TimeMachineError::FileNotFound(_) => "File not found",
            TimeMachineError::FileInaccessible(_) => "File inaccessible",
            TimeMachineError::Unknown(_description, _) => "Unknown error",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match &self {
            TimeMachineError::FileNotFound(Some(e)) => Some(e.as_ref()),
            TimeMachineError::FileInaccessible(Some(e)) => Some(e.as_ref()),
            TimeMachineError::Unknown(_, Some(e)) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl Display for TimeMachineError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            TimeMachineError::FileNotFound(_) => write!(f, "File not found"),
            TimeMachineError::FileInaccessible(_) => write!(f, "File inaccessible"),
            TimeMachineError::Unknown(description, _) => {
                write!(f, "Unknown error: {}", description)
            }
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
        assert_matches!(result, Err(TimeMachineError::FileInaccessible(_)));
    }

    #[test]
    fn it_throws_not_found_if_cant_set_xattr() {
        let path = Path::new("./test_assets/not_a_file.txt");
        let result = TimeMachine::add_exclusion(path);

        assert_matches!(result, Err(TimeMachineError::FileNotFound(_)));
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
        assert_matches!(result, Err(TimeMachineError::FileInaccessible(_)));
    }

    #[test]
    fn it_throws_not_found_if_cant_remove_xattr() {
        let path = Path::new("./test_assets/not_a_file.txt");
        let result = TimeMachine::remove_exclusion(path);

        assert_matches!(result, Err(TimeMachineError::FileNotFound(_)));
    }
}
