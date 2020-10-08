use dirs;
use std::path::PathBuf;
use std::fs::File;
use std::fs;
use anyhow;
use std::io::Write;

pub fn get_current_keys() -> anyhow::Result<Vec<String>>{
    let content = fs::read_to_string(get_auth_keys_path()?);
    let keys_string = match content {
        Ok(val) => val,
        Err(_) => String::new()
    };

    return Ok(clean_keys(split_keys(&keys_string)));
}

pub fn write_keys(keys: Vec<String>) -> anyhow::Result<()> {
    let content: String = keys.join("\n");
    let mut file: File = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(&get_auth_keys_path()?)?;
    
    file.write(content.as_bytes())?;
    return Ok(())
}

pub fn split_keys(all_keys: &str) -> Vec<String> {
    return all_keys.split("\n")
        .filter(|x| x.contains("ssh") || x.contains("ecdsa"))
        .map(|x| x.trim().to_owned())
        .collect();
}

fn clean_keys(original_keys: Vec<String>) -> Vec<String> {
    return original_keys.iter().map(|x| x
        .split(" ")
        .map(|x| x.to_owned())
        .collect::<Vec<String>>()[0..2]
        .join(" ")
    ).collect();
}

fn get_auth_keys_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir();

    let ssh_auth_path = match home {
        Some(path) => path.join(".ssh").join("authorized_keys"), 
        None => PathBuf::new(), //TODO find abs path of ssh dir
    };

    if !ssh_auth_path.is_file(){
        File::create(&ssh_auth_path)?;
    }

    return Ok(ssh_auth_path);
}