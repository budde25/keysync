use anyhow::{Context, Result};
use log::info;
use nix::unistd::{chown, Gid, Uid, User};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use super::util;

// Authorized keys file implementation
#[derive(Debug)]
pub struct AuthorizedKeys {
    path: PathBuf,
}

impl AuthorizedKeys {
    /// Sets up the AuthorizedKeys object
    pub fn open(user: Option<&str>) -> Result<Self> {
        let ids: (Uid, Gid) = get_uid_gid(user)?;
        // TODO support non default home dir
        let home_dir = if let Some(u) = user {
            PathBuf::from("/home").join(u)
        } else {
            dirs::home_dir().context("Failed to get users home directory")?
        };

        let path = home_dir.join(".ssh").join("authorized_keys");

        // Create the authorized keys file or path
        if !path.is_file() {
            if let Some(file_path) = path.parent() {
                if !file_path.is_dir() {
                    fs::create_dir(file_path)?;
                    chown(file_path, Some(ids.0), Some(ids.1))?;
                }
            }
            File::create(&path)?;
            chown(&path, Some(ids.0), Some(ids.1))?;
        }

        Ok(AuthorizedKeys { path })
    }

    /// Gets array of current authorized keys
    pub fn get_keys(&self) -> Result<Vec<String>> {
        info!("Reading keys to {}", self.path.display());
        let keys_string = fs::read_to_string(&self.path)
            .with_context(|| format!("Error reading keys from file: {}", self.path.display()))?;
        Ok(util::clean_keys(util::split_keys(&keys_string)))
    }

    /// Writes array of keys to authorized keys file
    pub fn write_keys(&self, keys: Vec<String>) -> Result<()> {
        info!("Writing keys to {}", self.path.display());
        if keys.is_empty() {
            return Ok(());
        }
        let content: String = keys.join("\n") + "\n";
        let mut file: File = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("Error opening keys file: {}", self.path.display()))?;

        file.write_all(content.as_bytes())
            .with_context(|| format!("Error writing keys from file: {}", self.path.display()))?;
        Ok(())
    }
}

/// Gets the User Id and Group Id of user provided of current user
fn get_uid_gid<S: AsRef<str>>(user: Option<S>) -> Result<(Uid, Gid)> {
    if let Some(u) = user {
        match User::from_name(u.as_ref())
            .with_context(|| format!("Unable to get the Uid and Gid of user: {}", u.as_ref()))?
        {
            Some(user) => Ok((user.uid, user.gid)),
            None => Ok((Uid::current(), Gid::current())),
        }
    } else {
        Ok((Uid::current(), Gid::current()))
    }
}
