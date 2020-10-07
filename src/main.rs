use anyhow::anyhow;
use structopt::clap::Shell;
use structopt::StructOpt;
use url::Url;

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
        /// The username of the account to get keys from
        #[structopt(name = "username")]
        username: String,

        /// Retrive from github (default)
        #[structopt(short, long)]
        github: bool,

        /// Retrive from gitlab, requires url
        #[structopt(name="url", short = "l", long = "gitlab")]
        url: Option<String>,
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
        Cli::Get {username, github, url} => get(username, github, url)?,
        Cli::Set {} => (),
        Cli::Job {} => (),
    }
    return Ok(());
}

fn get(username: String, mut github: bool, gitlab: Option<String>) -> anyhow::Result<()> {

    // if none are selected default to github
    if !github && gitlab.is_none() {
        github = true;
    }

    let mut keys: Vec<String> = vec![];

    if github {
        let response = http::get_github(&username)?;
        keys.append(&mut file::split_keys(&response));
    }

    if let Some(mut url) = gitlab {
        if !url.contains("http") {
            url = format!("{}{}", "https://", url);
        }
        let gitlab_url: Url = Url::parse(&url)?;
        let response = http::get_gitlab(&username, gitlab_url)?;
        keys.append(&mut file::split_keys(&response));
    }

    let keys_to_add: Vec<String> = filter_keys(keys, file::get_current_keys());
    let num_keys_to_add: usize = keys_to_add.len();

    //file::write_keys(keys_to_add)?;

    println!("Added {} keys", num_keys_to_add);
    return Ok(())
}

fn filter_keys(to_add: Vec<String>, exist: Vec<String>) -> Vec<String> {
    return to_add.iter().filter(|x| !exist.contains(x)).map(|x| x.to_owned() + " # ssh-import ssh-key-sync").collect();
}