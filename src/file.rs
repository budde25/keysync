use anyhow;
use dirs;
use filetime::FileTime;
use log::{debug, error, info, warn};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use nix::unistd;
use nix::unistd::Uid;
use nix::unistd::Gid;

use super::util;

pub fn get_current_keys(user: Option<&str>) -> anyhow::Result<Vec<String>> {
    let content = fs::read_to_string(get_auth_keys_path(user));
    let keys_string = match content {
        Ok(val) => val,
        Err(_) => String::new(),
    };

    return Ok(util::clean_keys(util::split_keys(&keys_string)));
}

pub fn write_keys(keys: Vec<String>, username: Option<&str>) -> anyhow::Result<()> {
    let path = get_auth_keys_path(username);

    info!("Writing keys to {:?}", path);

    let content: String = keys.join("\n");
    let mut file: File = match fs::OpenOptions::new().write(true).append(true).open(&path) {
        Ok(f) => f,
        Err(e) => {
            error!("Opening file {:?} failed. {}", path, e);
            return Ok(());
        }
    };

    match file.write(content.as_bytes()) {
        Ok(_) => return Ok(()),
        Err(e) => {
            error!("Writing to file {:?} failed. {}", path, e);
            return Ok(());
        }
    }
}

pub fn get_auth_keys_path(user: Option<&str>) -> PathBuf {
    let home = match user {
        Some(username) => Option::Some(PathBuf::from("/home").join(username)),
        None => dirs::home_dir(),
    };

    let ssh_auth_path = match home {
        Some(path) => path.join(".ssh").join("authorized_keys"),
        None => PathBuf::new(), //TODO find abs path of ssh dir
    };

    return ssh_auth_path;
}

fn get_schedule_path() -> PathBuf {
    return PathBuf::from("/etc/ssh-key-sync-schedule");
}

pub fn get_schedule() -> anyhow::Result<Vec<String>> {
    let schedule_path = get_schedule_path();
    if !schedule_path.is_file() {
        return Ok(vec![]);
    }

    let content: String = fs::read_to_string(schedule_path)?;
    return Ok(content.split("\n").filter(|x| !x.trim().is_empty()).map(|x| x.to_owned()).collect());
}

pub fn create_schedule_if_not_exist() -> anyhow::Result<bool> {
    let schedule_path: PathBuf = get_schedule_path();
    if !schedule_path.is_file() {
        File::create(schedule_path)?;
        return Ok(true);
    }
    return Ok(false);
}

pub fn schedule_last_modified() -> anyhow::Result<FileTime> {
    let metadata = fs::metadata(get_schedule_path()).unwrap();
    return Ok(FileTime::from_last_modification_time(&metadata));
}

pub fn create_file_for_user(user: Option<&str>) -> anyhow::Result<()> {
    let ids = match user {
        Some(u) => util::get_uid_gid(&u)?,
        None => (Uid::current(), Gid::current()),
    };
    let path = get_auth_keys_path(user);
    create_file(path, ids.0, ids.1)?;

    return Ok(());
}

fn create_file(path: PathBuf, uid: Uid, gid: Gid) -> anyhow::Result<()> {
    let file_path = path.parent().unwrap();
    if !file_path.is_dir() {
        fs::create_dir(file_path)?;
        unistd::chown(&path, Some(uid), Some(gid))?;
    }
    if !path.is_file() {
        File::create(&path)?;
        unistd::chown(&path, Some(uid), Some(gid))?;
    }
    return Ok(())
}