use nix::unistd;
use nix::unistd::Uid;
use nix::unistd::Gid;

pub fn get_uid_gid(user: &str) -> anyhow::Result<(Uid, Gid)> {
    let mut uid: Uid = Uid::current();
    let mut gid: Gid = Gid::current();
    let user_option = unistd::User::from_name(user)?;

    match user_option {
        Some(user) => {
            uid = user.uid;
            gid = user.gid;
        }
        None => {},
    };

    return Ok((uid, gid));
}

/// Filters the keys to prevent adding duplicates
pub fn filter_keys(to_add: Vec<String>, exist: Vec<String>) -> Vec<String> {
    return to_add
        .iter()
        .filter(|x| !exist.contains(x))
        .map(|x| x.to_owned() + " # ssh-import ssh-key-sync")
        .collect();
}

pub fn split_keys(all_keys: &str) -> Vec<String> {
    return all_keys
        .split("\n")
        .filter(|x| x.contains("ssh") || x.contains("ecdsa"))
        .map(|x| x.trim().to_owned())
        .collect();
}

pub fn clean_keys(original_keys: Vec<String>) -> Vec<String> {
    return original_keys
        .iter()
        .map(|x| x.split(" ").map(|x| x.to_owned()).collect::<Vec<String>>()[0..2].join(" "))
        .collect();
}