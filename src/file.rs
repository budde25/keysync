use filetime::FileTime;
use log::{error, info};
use nix::unistd;
use nix::unistd::Gid;
use nix::unistd::Uid;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use super::util;

pub fn get_current_keys(user: Option<&str>) -> anyhow::Result<Vec<String>> {
    let content = fs::read_to_string(get_auth_keys_path(user));
    let keys_string = match content {
        Ok(val) => val,
        Err(_) => String::new(),
    };

    Ok(util::clean_keys(util::split_keys(&keys_string)))
}

pub fn write_keys(keys: Vec<String>, username: Option<&str>) -> anyhow::Result<()> {
    let path = get_auth_keys_path(username);

    info!("Writing keys to {:?}", path);

    if keys.is_empty() {
        return Ok(());
    }

    let content: String = keys.join("\n") + "\n";
    let mut file: File = match fs::OpenOptions::new().write(true).append(true).open(&path) {
        Ok(f) => f,
        Err(e) => {
            error!("Opening file {:?} failed", path);
            return Err(anyhow::anyhow!("{}", e));
        }
    };

    match file.write_all(content.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Writing to file {:?} failed", path);
            return Err(anyhow::anyhow!("{}", e));
        }
    }
}

pub fn get_auth_keys_path(user: Option<&str>) -> PathBuf {
    let home = match user {
        Some(username) => Option::Some(PathBuf::from("/home").join(username)),
        None => dirs::home_dir(),
    };

    match home {
        Some(path) => path.join(".ssh").join("authorized_keys"),
        None => PathBuf::new(), //TODO find abs path of ssh dir
    }
}

pub fn get_schedule_path() -> PathBuf {
    PathBuf::from("/usr/share/keysync/schedule.db")
}

pub fn schedule_last_modified() -> anyhow::Result<FileTime> {
    let metadata = fs::metadata(get_schedule_path())?;
    Ok(FileTime::from_last_modification_time(&metadata))
}

pub fn create_file_for_user(user: Option<&str>) -> anyhow::Result<()> {
    let ids = match user {
        Some(u) => util::get_uid_gid(&u)?,
        None => (Uid::current(), Gid::current()),
    };
    let path = get_auth_keys_path(user);
    create_file(path, ids.0, ids.1)?;

    Ok(())
}

fn create_file(path: PathBuf, uid: Uid, gid: Gid) -> anyhow::Result<()> {
    let file_path = path.parent().unwrap();
    if !file_path.is_dir() {
        fs::create_dir(file_path)?;
        unistd::chown(file_path, Some(uid), Some(gid))?;
    }
    if !path.is_file() {
        File::create(&path)?;
        unistd::chown(&path, Some(uid), Some(gid))?;
    }
    Ok(())
}
