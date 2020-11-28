use anyhow::{Context, Result};
use log::info;
use nix::unistd::{chown, Gid, Uid, User};
use std::io::Write;
use std::path::{PathBuf,Path};
use std::{fs, fs::File};

use super::util;

// Authorized keys file implementation
#[derive(Debug)]
pub struct AuthorizedKeys {
    path: PathBuf,
}

impl AuthorizedKeys {
    /// Sets up Authorized keys for a given directory
    pub fn open_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let pathbuf = path.as_ref().to_path_buf();
        if !pathbuf.is_file() {
            File::create(&path).context("Failed to create the AuthorizedKeys file")?;
        }
        Ok(AuthorizedKeys { path: pathbuf })
    }

    /// Sets up the AuthorizedKeys object
    pub fn open<S: AsRef<str>>(user: Option<S>) -> Result<Self> {
        let ids: (Uid, Gid) = get_uid_gid(user.as_ref())?;
        // TODO support non default home dir
        let home_dir = if let Some(u) = user {
            PathBuf::from("/home").join(u.as_ref())
        } else {
            dirs::home_dir().context("Failed to get home directory")?
        };

        let path = home_dir.join(".ssh").join("authorized_keys");

        // Create the authorized keys file or path
        if !path.is_file() {
            if let Some(file_path) = path.parent() {
                if !file_path.is_dir() {
                    fs::create_dir_all(file_path).with_context(|| format!("Failed to create the directory [{}] for the authorized_keys file", file_path.display()))?;
                    chown(file_path, Some(ids.0), Some(ids.1)).with_context(|| format!("Failed to set the folder [{}] ownership to user", file_path.display()))?;
                }
            }
            File::create(&path).with_context(|| format!("Failed to create the authorized_keys [{}] file", path.display()))?;
            chown(&path, Some(ids.0), Some(ids.1)).with_context(|| format!("Failed to set authorized_keys [{}] ownership to user", path.display()))?;
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

        // If we have no keys to write we can just exit
        if keys.is_empty() {
            return Ok(());
        }

        let content: String = keys.join("\n") + "\n"; // We want each to be on its own line while also appending a newline
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

/// Gets the User Id and Group Id of user provided, if no user was provided just returns the current user
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
