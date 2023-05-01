use std::process::Command;

use anyhow::{Context, Result};

pub struct TimeMachine {}

impl TimeMachine {
    pub fn add_exclusion(path: &str) -> Result<()> {
        Command::new("/usr/bin/tmutil")
            .arg("addexclusion")
            .arg(path)
            .output()
            .with_context(|| format!("Can't add exclusion for {}", path))?;

        Ok(())
    }

    pub fn remove_exclusion(path: &str) -> Result<()> {
        Command::new("/usr/bin/tmutil")
            .arg("removeexclusion")
            .arg(path)
            .output()
            .with_context(|| format!("Can't remove exclusion for {}", path))?;

        Ok(())
    }

    pub fn is_excluded(path: &str) -> bool {
        let result = Command::new("/usr/bin/xattr").arg(path).output().unwrap();

        let attrs: Vec<String> = String::from_utf8(result.stdout)
            .unwrap()
            .lines()
            .map(String::from)
            .collect();

        attrs.contains(&"com.apple.metadata:com_apple_backup_excludeItem".to_string())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    use std::fs::{self, File};

    #[test]
    fn it_sets_xattr() {
        let cwd = fs::canonicalize("./").unwrap();
        let pathstr = cwd
            .join(format!("./text-{}.txt", Uuid::new_v4()))
            .to_str()
            .unwrap()
            .to_string();
        File::create(pathstr.clone()).unwrap();
        TimeMachine::add_exclusion(&pathstr).unwrap();

        assert!(TimeMachine::is_excluded(&pathstr));

        fs::remove_file(pathstr).unwrap();
    }

    #[test]
    fn it_removes_xattr() {
        let cwd = fs::canonicalize("./").unwrap();
        let pathstr = cwd
            .join(format!("./text-{}.txt", Uuid::new_v4()))
            .to_str()
            .unwrap()
            .to_string();
        File::create(pathstr.clone()).unwrap();
        TimeMachine::add_exclusion(&pathstr).unwrap();

        assert!(TimeMachine::is_excluded(&pathstr));

        TimeMachine::remove_exclusion(&pathstr).unwrap();

        assert!(!TimeMachine::is_excluded(&pathstr));

        fs::remove_file(pathstr).unwrap();
    }
}
