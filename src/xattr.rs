use std::{io, process::Command};

pub struct Xattr {}

impl Xattr {
    pub fn set_xattr(path: &str, attribute: &str) -> Result<(), io::Error> {
        let result = Command::new("/usr/bin/xattr")
            .arg("-w")
            .arg(attribute)
            .arg("1")
            .arg(path)
            .output()?;

        if !result.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Cannot set xattr"));
        }

        Ok(())
    }

    pub fn unset_xattr(path: &str, attribute: &str) -> Result<(), io::Error> {
        let result = Command::new("/usr/bin/xattr")
            .arg("-d")
            .arg(attribute)
            .arg(path)
            .output()?;

        if !result.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Cannot unset xattr"));
        }

        Ok(())
    }
}
