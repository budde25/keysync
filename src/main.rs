use anyhow;
use log::{debug, error, info, warn};
use structopt::clap::Shell;
use structopt::StructOpt;
use url::Url;

mod daemon;
mod file;
mod http;

#[derive(StructOpt)]
#[structopt(
    name = "SSH Key Sync",
    about = "A command line client and service for keeping SHH keys up to date with a list Ex: Github."
)]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    pub cmd: Command,
}
/// Main Cli struct with StructOpt
#[derive(Debug, StructOpt)]
#[structopt(
    name = "SSH Key Sync",
    about = "A command line client and service for keeping SHH keys up to date with a list Ex: Github."
)]
enum Command {
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
        #[structopt(name = "url", short = "l", long = "gitlab")]
        url: Option<String>,
    },

    /// Add an automatic job
    #[structopt(name = "set")]
    Set {},

    /// Current enabled jobs
    #[structopt(name = "jobs")]
    Job {},
}

fn main() -> anyhow::Result<()> {
    // If being run by the service, probably better way to handle this
    // TODO make it better
    if std::env::args().len() == 2 && std::env::args_os().into_iter().last().unwrap() == "--daemon"
    {
        daemon::start()?;
        anyhow::anyhow!("Process should not stop");
    }

    Opt::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");

    let cli: Opt = Opt::from_args();

    // Sets the log level
    match cli.verbose {
        0 => env_logger::builder()
            .filter_level(log::LevelFilter::Warn)
            .init(),
        1 => env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .init(),
        2 => env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init(),
        _ => env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init(),
    };

    info!("Logger has been intaialized");

    match cli.cmd {
        Command::Get {
            username,
            github,
            url,
        } => get(username, github, url)?,
        Command::Set {} => (),
        Command::Job {} => (),
    };

    return Ok(());
}

/// Gets the keys from a provider
fn get(username: String, mut github: bool, gitlab: Option<String>) -> anyhow::Result<()> {
    info!("Getting data for {}", username);

    // if none are selected default to github
    if !github && gitlab.is_none() {
        github = true;
    }

    debug!(
        "github: {}, gitlab: {}, username {}",
        github,
        gitlab.is_some(),
        username,
    );

    let mut keys: Vec<String> = vec![];

    if github {
        let url = Url::parse(http::GITHUB_URL)?;
        let response = http::get_standard(&username, url)?;
        keys.append(&mut file::split_keys(&response));
    }

    if let Some(mut url) = gitlab {
        // Default url for empty string
        if url.trim().is_empty() {
            url = http::GITLAB_URL.to_string();
        }

        // Adds https but allows http if specified
        if !url.contains("http") {
            url = format!("{}{}", "https://", url);
        }

        let gitlab_url: Url = Url::parse(&url)?;
        let response = http::get_standard(&username, gitlab_url)?;
        keys.append(&mut file::split_keys(&response));
    }

    let keys_to_add: Vec<String> = filter_keys(keys, file::get_current_keys(None)?);
    let num_keys_to_add: usize = keys_to_add.len();

    file::write_keys(keys_to_add, None)?;

    println!("Added {} keys", num_keys_to_add);
    return Ok(());
}

/// Filters the keys to prevent adding duplicates
pub fn filter_keys(to_add: Vec<String>, exist: Vec<String>) -> Vec<String> {
    return to_add
        .iter()
        .filter(|x| !exist.contains(x))
        .map(|x| x.to_owned() + " # ssh-import ssh-key-sync")
        .collect();
}
