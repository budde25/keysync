use anyhow;
use dirs;
use filetime::FileTime;
use log::{debug, error, info, warn};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn get_current_keys(user: Option<String>) -> anyhow::Result<Vec<String>> {
    let content = fs::read_to_string(get_auth_keys_path(user)?);
    let keys_string = match content {
        Ok(val) => val,
        Err(_) => String::new(),
    };

    return Ok(clean_keys(split_keys(&keys_string)));
}

pub fn write_keys(keys: Vec<String>, username: Option<String>) -> anyhow::Result<()> {
    let path = get_auth_keys_path(username)?;

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

pub fn split_keys(all_keys: &str) -> Vec<String> {
    return all_keys
        .split("\n")
        .filter(|x| x.contains("ssh") || x.contains("ecdsa"))
        .map(|x| x.trim().to_owned())
        .collect();
}

fn clean_keys(original_keys: Vec<String>) -> Vec<String> {
    return original_keys
        .iter()
        .map(|x| x.split(" ").map(|x| x.to_owned()).collect::<Vec<String>>()[0..2].join(" "))
        .collect();
}

fn get_auth_keys_path(user: Option<String>) -> anyhow::Result<PathBuf> {
    let home = match user {
        Some(username) => Option::Some(PathBuf::from("/home").join(username)),
        None => dirs::home_dir(),
    };

    let ssh_auth_path = match home {
        Some(path) => path.join(".ssh").join("authorized_keys"),
        None => PathBuf::new(), //TODO find abs path of ssh dir
    };

    if !ssh_auth_path.is_file() {
        File::create(&ssh_auth_path)?;
    }

    return Ok(ssh_auth_path);
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
    return Ok(content.split("\n").map(|x| x.to_owned()).collect());
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
