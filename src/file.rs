use anyhow::{Context, Result};
use log::info;
use nix::unistd::{chown, Gid, Uid, User};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, fs::File};

use super::util;

// Authorized keys file implementation
#[derive(Debug)]
pub struct AuthorizedKeys {
    path: PathBuf,
}

impl AuthorizedKeys {
    /// Sets up Authorized keys for a given directory
    #[allow(dead_code)] // Used for testing
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
                    fs::create_dir_all(file_path).with_context(|| {
                        format!(
                            "Failed to create the directory [{}] for the authorized_keys file",
                            file_path.display()
                        )
                    })?;
                    chown(file_path, Some(ids.0), Some(ids.1)).with_context(|| {
                        format!(
                            "Failed to set the folder [{}] ownership to user",
                            file_path.display()
                        )
                    })?;
                }
            }
            File::create(&path).with_context(|| {
                format!(
                    "Failed to create the authorized_keys [{}] file",
                    path.display()
                )
            })?;
            chown(&path, Some(ids.0), Some(ids.1)).with_context(|| {
                format!(
                    "Failed to set authorized_keys [{}] ownership to user",
                    path.display()
                )
            })?;
        }

        Ok(AuthorizedKeys { path })
    }

    /// Gets array of current authorized keys, and true if the keys file ends with a newline, false otherwise
    fn get_keys(&self) -> Result<(Vec<String>, bool)> {
        info!("Reading keys to {}", self.path.display());
        let keys_string = fs::read_to_string(&self.path)
            .with_context(|| format!("Error reading keys from file: {}", self.path.display()))?;
        let keys = util::clean_keys(util::split_keys(&keys_string));
        let ends_with_newline = keys_string.ends_with("\n");
        Ok((keys, ends_with_newline))
    }

    /// Writes array of keys to authorized keys file, returns amount of keys to write or written
    pub fn write_keys(&self, keys: Vec<String>, dry_run: bool) -> Result<usize> {
        let (existing_keys, ends_with_newline) = self.get_keys()?;
        let keys_to_add = util::filter_keys(keys, existing_keys);

        info!("Writing keys to {}", self.path.display());

        // If we have no keys to write we can just exit
        if keys_to_add.is_empty() || dry_run {
            return Ok(keys_to_add.len());
        }

        let prefix = if !ends_with_newline {
            String::from("\n")
        } else {
            String::new()
        };
        let content: String = prefix + &keys_to_add.join("\n") + "\n"; // We want each to be on its own line while also appending a newline

        let mut file: File = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("Error opening keys file: {}", self.path.display()))?;

        file.write_all(content.as_bytes())
            .with_context(|| format!("Error writing keys from file: {}", self.path.display()))?;
        Ok(keys_to_add.len())
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

// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    /// Tests that a new file will have no keys
    #[test]
    fn test_getting_empty_keys() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("authorized_keys"); // Don't create, functions should do it for us
        let authorized_keys = AuthorizedKeys::open_path(file.path()).unwrap();
        assert_eq!(authorized_keys.get_keys().unwrap().0.len(), 0);
    }

    /// Tests that a new file will have no keys
    #[test]
    fn test_getting_two_keys() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("authorized_keys");
        file.touch().unwrap();
        file.write_str("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDArNszFqR3vxzTe+pr/U/kmCn8aQAHNKfPMK4DJEvMvEbypiJV3Pm4iQG8jK6xBOTvcrFJTDX0VvgG0ky+iGOaLXw/M30BUsRhZlonasa0tbuu1PtHXlToXaCPyIPB39XucTjOQYtyFoS7yMfBuw0JhQ4ETJflvvHet5UkrbcqoSrac2ljtokmwR7z6cFEJTDXncEAhJsSJVQgPXWlf/j76XV8tP7ZFOBR7UVLSR2TXCLtg67o4Whu3ji/BV5Qa6t6Ef6rT4mndB29rY9D35qpASVlic84WzYKwRSfsc9FtryaA6mQMbfhN3xySKkfV5CgrVCH/rHGP09VzMlrlR+tHZDqznxeL4pr7+uJOHvMbgZHBvdbanQyApSGdB6HbRB1z8lVmbtOAsuK4TNkTQUNo8204NKJgtEsZnbqOWM0OMiJpjmhftqMq0Wl7OzZYWDzAEgS3ELoAl1DCkO4RkXsXWdHNK3p2MtxXOj3yM6MWZTPGT3dJXqATdu4lzknvSc= #maybe a comment
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDPx4jsUuivW/Yz0r7eD/InptzObq+qmwEP7fJrNZIOkKYyfVaxIxHYnAix7h4Qjk6dRq15to9slBSohRlXpXAx0WFpOMRgxC56uqnbGfh3fh8XxEIr23OHxiwoh4paS6CKu9Jz53S8lM6jSHsdH+0CmLm/iEw9Y0KtzOEzee6RR6EJUvs4TGSvaapOQJse4ZQNFJU0xBMVaGs4HQ2VitwrWVn/lvJoSoWk2fAAEGLcI2FOEoMBfnaAwyRj3F/L3hJ4vu77N7qvxdVCz7FRAEGPBcnoaeB4ivA2MXz3tEkHAilMTiUIMdPjS65lPyXfzWvlVQid3iMOb7oQcD4cI3oJ").unwrap();
        let authorized_keys = AuthorizedKeys::open_path(file.path()).unwrap();
        assert_eq!(authorized_keys.get_keys().unwrap().0.len(), 2);
    }

    /// Tests that we can write to an existing file
    #[test]
    fn test_writing_empty_file() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("authorized_keys");
        let keys = vec!["ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDArNszFqR3vxzTe+pr/U/kmCn8aQAHNKfPMK4DJEvMvEbypiJV3Pm4iQG8jK6xBOTvcrFJTDX0VvgG0ky+iGOaLXw/M30BUsRhZlonasa0tbuu1PtHXlToXaCPyIPB39XucTjOQYtyFoS7yMfBuw0JhQ4ETJflvvHet5UkrbcqoSrac2ljtokmwR7z6cFEJTDXncEAhJsSJVQgPXWlf/j76XV8tP7ZFOBR7UVLSR2TXCLtg67o4Whu3ji/BV5Qa6t6Ef6rT4mndB29rY9D35qpASVlic84WzYKwRSfsc9FtryaA6mQMbfhN3xySKkfV5CgrVCH/rHGP09VzMlrlR+tHZDqznxeL4pr7+uJOHvMbgZHBvdbanQyApSGdB6HbRB1z8lVmbtOAsuK4TNkTQUNo8204NKJgtEsZnbqOWM0OMiJpjmhftqMq0Wl7OzZYWDzAEgS3ELoAl1DCkO4RkXsXWdHNK3p2MtxXOj3yM6MWZTPGT3dJXqATdu4lzknvSc=".to_owned(),"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDPx4jsUuivW/Yz0r7eD/InptzObq+qmwEP7fJrNZIOkKYyfVaxIxHYnAix7h4Qjk6dRq15to9slBSohRlXpXAx0WFpOMRgxC56uqnbGfh3fh8XxEIr23OHxiwoh4paS6CKu9Jz53S8lM6jSHsdH+0CmLm/iEw9Y0KtzOEzee6RR6EJUvs4TGSvaapOQJse4ZQNFJU0xBMVaGs4HQ2VitwrWVn/lvJoSoWk2fAAEGLcI2FOEoMBfnaAwyRj3F/L3hJ4vu77N7qvxdVCz7FRAEGPBcnoaeB4ivA2MXz3tEkHAilMTiUIMdPjS65lPyXfzWvlVQid3iMOb7oQcD4cI3oJ".to_owned()];
        let authorized_keys = AuthorizedKeys::open_path(file.path()).unwrap();
        assert_eq!(authorized_keys.write_keys(keys, false).unwrap(), 2);
        assert_eq!(authorized_keys.get_keys().unwrap().0.len(), 2);
    }

    /// Tests that we can write to an existing file with data
    #[test]
    fn test_writing_empty_file_one_dup() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("authorized_keys");
        file.write_str("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDArNszFqR3vxzTe+pr/U/kmCn8aQAHNKfPMK4DJEvMvEbypiJV3Pm4iQG8jK6xBOTvcrFJTDX0VvgG0ky+iGOaLXw/M30BUsRhZlonasa0tbuu1PtHXlToXaCPyIPB39XucTjOQYtyFoS7yMfBuw0JhQ4ETJflvvHet5UkrbcqoSrac2ljtokmwR7z6cFEJTDXncEAhJsSJVQgPXWlf/j76XV8tP7ZFOBR7UVLSR2TXCLtg67o4Whu3ji/BV5Qa6t6Ef6rT4mndB29rY9D35qpASVlic84WzYKwRSfsc9FtryaA6mQMbfhN3xySKkfV5CgrVCH/rHGP09VzMlrlR+tHZDqznxeL4pr7+uJOHvMbgZHBvdbanQyApSGdB6HbRB1z8lVmbtOAsuK4TNkTQUNo8204NKJgtEsZnbqOWM0OMiJpjmhftqMq0Wl7OzZYWDzAEgS3ELoAl1DCkO4RkXsXWdHNK3p2MtxXOj3yM6MWZTPGT3dJXqATdu4lzknvSc= #maybe a comment
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDPx4jsUuivW/Yz0r7eD/InptzObq+qmwEP7fJrNZIOkKYyfVaxIxHYnAix7h4Qjk6dRq15to9slBSohRlXpXAx0WFpOMRgxC56uqnbGfh3fh8XxEIr23OHxiwoh4paS6CKu9Jz53S8lM6jSHsdH+0CmLm/iEw9Y0KtzOEzee6RR6EJUvs4TGSvaapOQJse4ZQNFJU0xBMVaGs4HQ2VitwrWVn/lvJoSoWk2fAAEGLcI2FOEoMBfnaAwyRj3F/L3hJ4vu77N7qvxdVCz7FRAEGPBcnoaeB4ivA2MXz3tEkHAilMTiUIMdPjS65lPyXfzWvlVQid3iMOb7oQcD4cI3oJ\n").unwrap();
        let keys = vec!["ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDNYBebNl8QY24FWJZLmwDrtSvQZaiMXe8sNUHbDNkPPNKe1XPqH1tD8ZBjqxlz/mxMfLOR6izFYj6Y6OSo4kqmuhhXZvSST0sPashF7PM/sfiR4gjoisuE/CfUsjGbvSBfQ5nfUJkQoUrzMQjA5CltbGO8prd6mt0u37RJqdBzAUAKaflwlpei5AWnE3a0RAlm29s/1bec28oQWkxqjLf11SqqJNgOsL70NDByMk+8LQ2OqEJHVHAIQjml/oSgKBG+SLO3uOX6M3bhmtAaP2MYAZdp/84qDFMX5lKk37k3xMjWmlHstyqkgJ3Hfp/Ft2KC1pPTKNK+C5VYqlROO/qKNSlIf80HNSGHBJymU/ncjcxOaofSkrLHzANsloYJileuUZs9sN48Eebrx0enz5Bf6ySxp5mmjesbASd0nxHXjZ0k9fiacAVeUTs54H/1VfIhLfVz4L8laZ79Ncbi0rprK7TofAKIErOOddgp3Wlj6CKeF+v6yPRCimb+IUt0qf0=".to_owned(),"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDPx4jsUuivW/Yz0r7eD/InptzObq+qmwEP7fJrNZIOkKYyfVaxIxHYnAix7h4Qjk6dRq15to9slBSohRlXpXAx0WFpOMRgxC56uqnbGfh3fh8XxEIr23OHxiwoh4paS6CKu9Jz53S8lM6jSHsdH+0CmLm/iEw9Y0KtzOEzee6RR6EJUvs4TGSvaapOQJse4ZQNFJU0xBMVaGs4HQ2VitwrWVn/lvJoSoWk2fAAEGLcI2FOEoMBfnaAwyRj3F/L3hJ4vu77N7qvxdVCz7FRAEGPBcnoaeB4ivA2MXz3tEkHAilMTiUIMdPjS65lPyXfzWvlVQid3iMOb7oQcD4cI3oJ".to_owned()];
        let authorized_keys = AuthorizedKeys::open_path(file.path()).unwrap();
        assert_eq!(authorized_keys.write_keys(keys, false).unwrap(), 1); // wrote one key only, since one was already there
        assert_eq!(authorized_keys.get_keys().unwrap().0.len(), 3);
    }

    /// Tests that we can write to an existing file with data
    #[test]
    fn test_writing_existing_with_a_newline() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("authorized_keys");
        file.write_str("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDArNszFqR3vxzTe+pr/U/kmCn8aQAHNKfPMK4DJEvMvEbypiJV3Pm4iQG8jK6xBOTvcrFJTDX0VvgG0ky+iGOaLXw/M30BUsRhZlonasa0tbuu1PtHXlToXaCPyIPB39XucTjOQYtyFoS7yMfBuw0JhQ4ETJflvvHet5UkrbcqoSrac2ljtokmwR7z6cFEJTDXncEAhJsSJVQgPXWlf/j76XV8tP7ZFOBR7UVLSR2TXCLtg67o4Whu3ji/BV5Qa6t6Ef6rT4mndB29rY9D35qpASVlic84WzYKwRSfsc9FtryaA6mQMbfhN3xySKkfV5CgrVCH/rHGP09VzMlrlR+tHZDqznxeL4pr7+uJOHvMbgZHBvdbanQyApSGdB6HbRB1z8lVmbtOAsuK4TNkTQUNo8204NKJgtEsZnbqOWM0OMiJpjmhftqMq0Wl7OzZYWDzAEgS3ELoAl1DCkO4RkXsXWdHNK3p2MtxXOj3yM6MWZTPGT3dJXqATdu4lzknvSc= #maybe a comment\n").unwrap();
        let keys = vec!["ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDNYBebNl8QY24FWJZLmwDrtSvQZaiMXe8sNUHbDNkPPNKe1XPqH1tD8ZBjqxlz/mxMfLOR6izFYj6Y6OSo4kqmuhhXZvSST0sPashF7PM/sfiR4gjoisuE/CfUsjGbvSBfQ5nfUJkQoUrzMQjA5CltbGO8prd6mt0u37RJqdBzAUAKaflwlpei5AWnE3a0RAlm29s/1bec28oQWkxqjLf11SqqJNgOsL70NDByMk+8LQ2OqEJHVHAIQjml/oSgKBG+SLO3uOX6M3bhmtAaP2MYAZdp/84qDFMX5lKk37k3xMjWmlHstyqkgJ3Hfp/Ft2KC1pPTKNK+C5VYqlROO/qKNSlIf80HNSGHBJymU/ncjcxOaofSkrLHzANsloYJileuUZs9sN48Eebrx0enz5Bf6ySxp5mmjesbASd0nxHXjZ0k9fiacAVeUTs54H/1VfIhLfVz4L8laZ79Ncbi0rprK7TofAKIErOOddgp3Wlj6CKeF+v6yPRCimb+IUt0qf0=".to_owned()];
        let authorized_keys = AuthorizedKeys::open_path(file.path()).unwrap();
        assert_eq!(authorized_keys.write_keys(keys, false).unwrap(), 1);
        assert_eq!(authorized_keys.get_keys().unwrap().0.len(), 2);
    }

    /// Tests that we can write to an existing file with data
    #[test]
    fn test_writing_existing_without_a_newline() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("authorized_keys");
        file.write_str("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDArNszFqR3vxzTe+pr/U/kmCn8aQAHNKfPMK4DJEvMvEbypiJV3Pm4iQG8jK6xBOTvcrFJTDX0VvgG0ky+iGOaLXw/M30BUsRhZlonasa0tbuu1PtHXlToXaCPyIPB39XucTjOQYtyFoS7yMfBuw0JhQ4ETJflvvHet5UkrbcqoSrac2ljtokmwR7z6cFEJTDXncEAhJsSJVQgPXWlf/j76XV8tP7ZFOBR7UVLSR2TXCLtg67o4Whu3ji/BV5Qa6t6Ef6rT4mndB29rY9D35qpASVlic84WzYKwRSfsc9FtryaA6mQMbfhN3xySKkfV5CgrVCH/rHGP09VzMlrlR+tHZDqznxeL4pr7+uJOHvMbgZHBvdbanQyApSGdB6HbRB1z8lVmbtOAsuK4TNkTQUNo8204NKJgtEsZnbqOWM0OMiJpjmhftqMq0Wl7OzZYWDzAEgS3ELoAl1DCkO4RkXsXWdHNK3p2MtxXOj3yM6MWZTPGT3dJXqATdu4lzknvSc= #maybe a comment").unwrap();
        let keys = vec!["ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDNYBebNl8QY24FWJZLmwDrtSvQZaiMXe8sNUHbDNkPPNKe1XPqH1tD8ZBjqxlz/mxMfLOR6izFYj6Y6OSo4kqmuhhXZvSST0sPashF7PM/sfiR4gjoisuE/CfUsjGbvSBfQ5nfUJkQoUrzMQjA5CltbGO8prd6mt0u37RJqdBzAUAKaflwlpei5AWnE3a0RAlm29s/1bec28oQWkxqjLf11SqqJNgOsL70NDByMk+8LQ2OqEJHVHAIQjml/oSgKBG+SLO3uOX6M3bhmtAaP2MYAZdp/84qDFMX5lKk37k3xMjWmlHstyqkgJ3Hfp/Ft2KC1pPTKNK+C5VYqlROO/qKNSlIf80HNSGHBJymU/ncjcxOaofSkrLHzANsloYJileuUZs9sN48Eebrx0enz5Bf6ySxp5mmjesbASd0nxHXjZ0k9fiacAVeUTs54H/1VfIhLfVz4L8laZ79Ncbi0rprK7TofAKIErOOddgp3Wlj6CKeF+v6yPRCimb+IUt0qf0=".to_owned()];
        let authorized_keys = AuthorizedKeys::open_path(file.path()).unwrap();
        assert_eq!(authorized_keys.write_keys(keys, false).unwrap(), 1);
        assert_eq!(authorized_keys.get_keys().unwrap().0.len(), 2);
    }
}
