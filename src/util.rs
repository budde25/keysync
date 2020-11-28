use anyhow::{anyhow, Result};
use nix::unistd::Uid;
use rustyline::Editor;

/// Filters the keys to prevent adding duplicates, also adds import comment
/// Returns a list of keys to that are unique
pub fn filter_keys(to_add: Vec<String>, exist: Vec<String>) -> Vec<String> {
    to_add
        .iter()
        .filter(|x| !exist.contains(x))
        .map(|x| x.to_owned() + " #ssh-import keysync")
        .collect()
}

/// Splits a string of keys into a list of keys based off a newline, also discards invalid keys
pub fn split_keys(all_keys: &str) -> Vec<String> {
    all_keys
        .split('\n')
        .filter(|x| x.contains("ssh") || x.contains("ecdsa"))
        .map(|x| x.trim().to_owned())
        .collect()
}

/// Removes any garbage from the keys Ex: comments
pub fn clean_keys(original_keys: Vec<String>) -> Vec<String> {
    original_keys
        .iter()
        .map(|x| x.split(' ').map(|x| x.to_owned()).collect::<Vec<String>>()[0..2].join(" "))
        .collect()
}

/// Runs the current command line options as root, (assuming sudo is installed)
pub fn run_as_root() -> Result<()> {
    if !Uid::current().is_root() {
        match std::process::Command::new("sudo")
            .args(std::env::args())
            .spawn()
        {
            Ok(mut sudo) => {
                let output = sudo.wait().expect("Command failed to request root");
                if output.success() {
                    std::process::exit(0);
                } else {
                    Err(anyhow!("Command failed to request root"))
                }
            }
            Err(_) => Err(anyhow!("Requires root")),
        }
    } else {
        Ok(())
    }
}

/// Prompts the user a question, with a (Y,n) attached.
/// Returns true if the user repsonds with y or yes, false otherwise
pub fn get_confirmation(query: &str) -> Result<bool> {
    let mut rl = Editor::<()>::new();
    let prompt = format!("{} (Y/n)\n>> ", query);
    let readline = rl.readline(&prompt);

    match readline {
        Ok(line) => {
            let clean_line = line.trim().to_lowercase();
            if clean_line == "y" || clean_line == "yes" {
                return Ok(true);
            }
        }
        Err(err) => {
            println!("Error: {:?}", err);
        }
    }
    Ok(false)
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that clean keys removes the junk off the end of keys
    #[test]
    fn test_keys_clean() {
        let keys = vec![
            "ssh 12e32 removes this".to_owned(),
            "test 1".to_owned(),
            "ecdsa 78".to_owned(),
        ];
        let clean = clean_keys(keys);
        assert_eq!(clean[0], "ssh 12e32");
    }

    /// Tests that split properly splits on the newline
    #[test]
    fn test_split() {
        let keys = "ssh-rsa key\nssh-rsa key2";
        let arr = split_keys(keys);
        assert_eq!(arr.len(), 2);
    }

    /// Tests that the duplicate keys get filtered out
    #[test]
    fn test_filter() {
        let org_keys = "ssh-rsa key\nssh-rsa key2";
        let new_keys = "ssh-rsa key2\nssh-rsa key3";
        let org_arr = split_keys(org_keys);
        let new_arr = split_keys(new_keys);
        let diff = filter_keys(org_arr, new_arr);
        assert_eq!(diff.len(), 1);
    }
}
