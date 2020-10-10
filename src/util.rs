use nix::unistd;
use nix::unistd::Gid;
use nix::unistd::Uid;

pub fn get_uid_gid(user: &str) -> anyhow::Result<(Uid, Gid)> {
    let mut uid: Uid = Uid::current();
    let mut gid: Gid = Gid::current();
    let user_option = unistd::User::from_name(user)?;

    match user_option {
        Some(user) => {
            uid = user.uid;
            gid = user.gid;
        }
        None => {}
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

// TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_clean() {
        let keys = vec!["ssh 12e32 removes this".to_owned(), "test 1".to_owned(), "ecdsa 78".to_owned()];
        let clean = clean_keys(keys);
        assert_eq!(clean[0], "ssh 12e32");
    }

    #[test]
    fn split() {
        let keys = "ssh-rsa key\nssh-rsa key2";
        let arr = split_keys(keys);
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn filter() {
        let org_keys = "ssh-rsa key\nssh-rsa key2";
        let new_keys = "ssh-rsa key2\nssh-rsa key3";
        let org_arr = split_keys(org_keys);
        let new_arr = split_keys(new_keys);
        let diff = filter_keys(org_arr, new_arr);
        assert_eq!(diff.len(), 1);
    }
}
