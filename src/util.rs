use super::http;
use nix::unistd;
use nix::unistd::Gid;
use nix::unistd::Uid;
use url::Url;

pub fn get_uid_gid(user: &str) -> anyhow::Result<(Uid, Gid)> {
    let user_option = unistd::User::from_name(user)?;

    match user_option {
        Some(user) => Ok((user.uid, user.gid)),
        None => Ok((Uid::current(), Gid::current())),
    }
}

/// Filters the keys to prevent adding duplicates
pub fn filter_keys(to_add: Vec<String>, exist: Vec<String>) -> Vec<String> {
    to_add
        .iter()
        .filter(|x| !exist.contains(x))
        .map(|x| x.to_owned() + " #ssh-import keysync")
        .collect()
}

pub fn split_keys(all_keys: &str) -> Vec<String> {
    all_keys
        .split('\n')
        .filter(|x| x.contains("ssh") || x.contains("ecdsa"))
        .map(|x| x.trim().to_owned())
        .collect()
}

pub fn clean_keys(original_keys: Vec<String>) -> Vec<String> {
    original_keys
        .iter()
        .map(|x| x.split(' ').map(|x| x.to_owned()).collect::<Vec<String>>()[0..2].join(" "))
        .collect()
}

// Returns a list of urls based for each service
pub fn create_urls(
    username: &str,
    mut github: bool,
    launchpad: bool,
    gitlab: Option<Url>,
) -> Vec<String> {
    // if none are selected default to github
    if !github && !launchpad && gitlab.is_none() {
        github = true
    };

    let mut urls: Vec<String> = vec![];
    if github {
        urls.push(http::get_github(username))
    };
    if launchpad {
        urls.push(http::get_launchpad(username))
    };
    match gitlab {
        Some(url) => urls.push(http::get_gitlab(username, Some(url))),
        None => (),
    };
    urls
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_clean() {
        let keys = vec![
            "ssh 12e32 removes this".to_owned(),
            "test 1".to_owned(),
            "ecdsa 78".to_owned(),
        ];
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
