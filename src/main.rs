use anyhow::anyhow;
use structopt::clap::Shell;
use structopt::StructOpt;

mod http;
mod file;
#[derive(Debug, StructOpt)]
#[structopt(
    name = "SSH Key Sync",
    about = "A command line client and service for keeping SHH keys up to date with a list Ex: Github."
)]
enum Cli {
    /// The username to fetch
    #[structopt(name = "get")]
    Get {
        /// The username to pull from
        #[structopt(name = "username")]
        username: String,

        /// Retrive from github (default)
        #[structopt(short, long)]
        github: bool,

        /// Retrive from launchpad
        #[structopt(short, long)]
        launchpad: bool,
    },

    /// Set a import to run on a job
    #[structopt(name = "set")]
    Set {},

    /// See active jobs
    #[structopt(name = "jobs")]
    Job {},

}

fn main() -> anyhow::Result<()> {
    Cli::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");

    let cli = Cli::from_args();
    match cli {
        Cli::Get {username, github, launchpad} => get(username, github, launchpad)?,
        Cli::Set {} => (),
        Cli::Job {} => (),
    }
    return Ok(());
}

fn get(username: String, mut github: bool, launchpad: bool) -> anyhow::Result<()> {

    // if none are selected default to github
    if !github && !launchpad {
        github = true;
    }

    let mut keys: Vec<String> = vec![];

    let github_response: String;
    if github {
        github_response = http::get_github(&username)?;
        let mut a = file::split_keys(&github_response);
        keys.append(&mut a);
    }

    let keys_to_add = filter_keys(keys, file::get_current_keys());
    let num_keys_to_add = keys_to_add.len();

    file::write_keys(keys_to_add)?;

    println!("Added {} keys", num_keys_to_add);
    return Ok(())
}

fn filter_keys(to_add: Vec<String>, exist: Vec<String>) -> Vec<String> {
    return to_add.iter().filter(|x| !exist.contains(x)).map(|x| x.to_owned() + " # ssh-import ssh-key-sync").collect();
}