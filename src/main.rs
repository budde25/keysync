use anyhow;
use clap::arg_enum;
use cron::Schedule;
use log::{debug, error, info, warn};
use nix::unistd::Uid;
use std::str::FromStr;
use structopt::clap::Shell;
use structopt::StructOpt;
use url::Url;

mod daemon;
mod file;
mod http;
mod util;

// Default Options options for a cron job
arg_enum! {
    #[derive(Debug)]
    enum DefaultCron {
        Hourly,
        Daily,
        Weekly,
        Monthly,
        Custom,
    }
}

#[derive(StructOpt)]
#[structopt(
    name = "SSH Key Sync",
    about = "A command line client and service for keeping SHH keys up to date with a list Ex: Github."
)]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Runs the commands without commiting the changes
    #[structopt(short, long)]
    dry_run: bool,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
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
    Set {
        /// Thelocal user who get the keys
        #[structopt(name = "user")]
        user: String,

        /// The username of the account to get keys from
        #[structopt(name = "username")]
        username: String,

        /// Premade schedules
        #[structopt(possible_values = &DefaultCron::variants(), case_insensitive = true)]
        schedule: DefaultCron,

        /// Retrive from github (default)
        #[structopt(short, long)]
        github: bool,

        /// Retrive from gitlab, requires url
        #[structopt(name = "url", short = "l", long = "gitlab")]
        url: Option<String>,

        /// A schedule in cron format Ex: '* * * * * *'
        #[structopt(
            name = "cron",
            short,
            long,
            required_if("schedule", "Custom"),
            required_if("schedule", "custom"),
            required_if("schedule", "CUSTOM")
        )]
        expression: Option<String>,
    },

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

    //Opt::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");

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
        } => get(username, github, url, cli.dry_run)?,
        Command::Set {
            user,
            username,
            schedule,
            github,
            url,
            expression,
        } => set(
            user,
            username,
            github,
            url,
            schedule,
            expression,
            cli.dry_run,
        )?,
        Command::Job {} => jobs()?,
    };

    return Ok(());
}

/// Gets the keys from a provider
fn get(
    username: String,
    mut github: bool,
    gitlab: Option<String>,
    dry_run: bool,
) -> anyhow::Result<()> {
    info!("Getting data for {}", username);

    // if none are selected default to github
    if !github && gitlab.is_none() {
        github = true;
    }

    if Uid::current().is_root() {
        warn!("Running get as root downloads the keys to the root users authorized keys file, which might not be intended");
    }

    if !dry_run {
        file::create_file_for_user(None)?;
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
        keys.append(&mut util::split_keys(&response));
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
        keys.append(&mut util::split_keys(&response));
    }

    let keys_to_add: Vec<String> = util::filter_keys(keys, file::get_current_keys(None)?);
    let num_keys_to_add: usize = keys_to_add.len();

    if !dry_run {
        file::write_keys(keys_to_add, None)?;
        println!("Added {} new keys", num_keys_to_add);
    } else {
        println!("Found {} new keys", num_keys_to_add);
    }

    return Ok(());
}

fn set(
    user: String,
    username: String,
    mut github: bool,
    gitlab: Option<String>,
    schedule: DefaultCron,
    expression: Option<String>,
    dry_run: bool,
) -> anyhow::Result<()> {
    if !Uid::current().is_root() {
        warn!("Adding new jobs requires write access to /etc/, you will probably need to run this as root");
    }

    if !dry_run {
        file::create_file_for_user(Some(&user))?;
        file::create_schedule_if_not_exist()?;
    }

    // if none are selected default to github
    if !github && gitlab.is_none() {
        github = true;
    }

    if github && gitlab.is_some() {
        error!("Must pick gitlab or github");
        return Ok(());
    }

    let cron_result: Result<String, cron::error::Error> = match schedule {
        DefaultCron::Hourly => parse_cron("@hourly"),
        DefaultCron::Daily => parse_cron("@daily"),
        DefaultCron::Weekly => parse_cron("@weekly"),
        DefaultCron::Monthly => parse_cron("@monthly"),
        DefaultCron::Custom => match expression {
            Some(exp) => parse_cron(&exp),
            None => {
                error!("Cron expression must be defined with 'Custom' Schedule");
                return Ok(());
            }
        },
    };

    let cron = match cron_result {
        Ok(c) => c,
        Err(e) => {
            error!("Unable to format this cron expression. {}", e);
            return Ok(());
        }
    };

    let url_to_add: String;
    if github {
        url_to_add = http::GITHUB_URL.to_owned();
    } else if let Some(u) = gitlab {
        // Default url for empty string
        if u.trim().is_empty() {
            url_to_add = http::GITLAB_URL.to_owned();
        } else {
            url_to_add = u;
        }
    } else {
        url_to_add = http::GITLAB_URL.to_owned();
    }

    let url_as_url: Url = Url::parse(&url_to_add)?;

    if !dry_run {
        file::write_to_schedule(&user, &cron, &url_as_url.to_string(), &username)?;
        println!("Successfully added import schedule")
    } else {
        println!("Syntax Ok")
    }

    return Ok(());
}

fn jobs() -> anyhow::Result<()> {
    let jobs = file::get_schedule()?;
    let total_jobs = jobs.len();
    println!(
        "Found {} job{}",
        total_jobs,
        if total_jobs == 1 { "" } else { "s" }
    );
    if total_jobs > 0 {
        println!(
            "{:<5}{:<15}{:<25}{:<30}{:<15}",
            "ID", "User", "Cron", "Url", "Username"
        );
        println!("{:-<90}", "");
        let mut count: u8 = 0;
        for job in jobs {
            count += 1;
            let data: Vec<&str> = job.split("|").collect();
            let user: &str = data[0];
            let cron: &str = data[1];
            let url: &str = data[2];
            let username: &str = data[3];
            println!(
                "{:<5}{:<15}{:<25}{:<30}{:<15}",
                count, user, cron, url, username
            );
        }
    }

    return Ok(());
}

fn parse_cron(src: &str) -> Result<String, cron::error::Error> {
    Schedule::from_str(src)?;
    return Ok(src.to_string());
}
