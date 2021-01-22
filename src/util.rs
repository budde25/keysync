use anyhow::{anyhow, Result};
use nix::unistd::Uid;
use regex::Regex;
use rustyline::{error::ReadlineError, Editor};
use std::process::{exit, Command};

// From regex example
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> =
            once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

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
        .map(|x| x.trim())
        .filter(|x| {
            let re: &Regex = regex!(
                r"^(ssh-rsa AAAAB3NzaC1yc2|ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNT|ecdsa-sha2-nistp384 AAAAE2VjZHNhLXNoYTItbmlzdHAzODQAAAAIbmlzdHAzOD|ecdsa-sha2-nistp521 AAAAE2VjZHNhLXNoYTItbmlzdHA1MjEAAAAIbmlzdHA1Mj|ssh-ed25519 AAAAC3NzaC1lZDI1NTE5|ssh-dss AAAAB3NzaC1kc3)",
            );
            re.is_match(x)
        })
        .map(|x| x.to_owned())
        .collect()
}

/// Removes any garbage from the keys Ex: comments
pub fn clean_keys(original_keys: Vec<String>) -> Vec<String> {
    original_keys
        .iter()
        .map(|x| {
            x.split(' ').map(|x| x.to_owned()).collect::<Vec<String>>()[0..2]
                .join(" ")
        })
        .collect()
}

/// Runs the current command line options as root, (assuming sudo is installed)
pub fn run_as_root(user: Option<&str>) -> Result<()> {
    if !Uid::current().is_root() {
        let result = if let Some(u) = user {
            Command::new("sudo")
                .args(std::env::args())
                .arg("--user")
                .arg(u)
                .spawn()
        } else {
            Command::new("sudo").args(std::env::args()).spawn()
        };

        match result {
            Ok(mut sudo) => {
                let output =
                    sudo.wait().expect("Command failed to request root");
                if output.success() {
                    exit(0);
                } else {
                    Err(anyhow!("Command failed"))
                }
            }
            Err(_) => Err(anyhow!("Requires root")),
        }
    } else {
        Ok(())
    }
}

/// Prompts the user a question, with a (Y,n) attached.
/// Returns true if the user responds with y or yes, false otherwise
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
        Err(err) => match err {
            ReadlineError::Eof | ReadlineError::Interrupted => (),
            _ => println!("Error: {:?}", err),
        },
    }
    Ok(false)
}

/// Gets the current user, from the $USER env variable
pub fn get_current_user() -> Result<String> {
    Ok(std::env::var("USER")?)
}

/// Unit Tests
#[cfg(test)]
#[path = "./tests/util.rs"]
mod test;
