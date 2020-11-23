use anyhow::anyhow;
use clap::arg_enum;
use cron::Schedule;
use log::{error, info, warn};
use nix::unistd::Uid;
use std::process;
use std::str::FromStr;
use structopt::StructOpt;
use url::{ParseError, Url};

mod daemon;
mod db;
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

    /// Runs the commands without committing the changes
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
    /// Retrieve a key online
    #[structopt(name = "get")]
    Get {
        /// The username of the account
        #[structopt(name = "username")]
        username: String,

        /// Retrieve from GitHub (default)
        #[structopt(short, long)]
        github: bool,

        /// Retrieve from GitLab [requires URL]
        #[structopt(name = "url", short = "h", long = "gitlab", parse(try_from_str = parse_url))]
        url: Option<Url>,

        /// Retrieve from Launchpad
        #[structopt(short, long)]
        launchpad: bool,
    },

    /// Add an automatic job
    #[structopt(name = "set")]
    Set {
        /// The local user account
        #[structopt(name = "user")]
        user: String,

        /// The username of the account to get keys from
        #[structopt(name = "username")]
        username: String,

        /// Default available schedules
        #[structopt(possible_values = &DefaultCron::variants(), case_insensitive = true)]
        schedule: DefaultCron,

        /// Retrieve from GitGub (default)
        #[structopt(short, long)]
        github: bool,

        /// Retrieve from Launchpad
        #[structopt(short, long)]
        launchpad: bool,

        /// Retrieve from GitLab [requires URL]
        #[structopt(name = "url", short = "h", long = "gitlab", parse(try_from_str = parse_url))]
        url: Option<Url>,

        /// Also runs in addition to adding to schedule
        #[structopt(short, long)]
        now: bool,

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

    /// Remove job(s) by ID
    #[structopt(name = "remove")]
    Remove {
        /// Job IDs to remove
        #[structopt()]
        id: Vec<i32>,
    },

    /// list enabled jobs
    #[structopt(name = "jobs")]
    Job {},
}

fn main() -> anyhow::Result<()> {
    // If being run by the service, probably better way to handle this
    // TODO make it better
    if std::env::args().len() == 2 && std::env::args_os().last().unwrap() == "--daemon" {
        daemon::start()?;
        anyhow::anyhow!("Process should not stop");
    }

    //Opt::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");
    let cli: Opt = Opt::from_args();

    // Sets the log level
    match cli.verbose {
        0 => env_logger::builder()
            .filter_level(log::LevelFilter::Warn)
            .format_timestamp(None)
            .init(),
        1 => env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .format_timestamp(None)
            .init(),
        2 => env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .format_timestamp(None)
            .init(),
        _ => env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .format_timestamp(None)
            .init(),
    };

    info!("Logger has been initialized");

    match cli.cmd {
        Command::Get {
            username,
            github,
            url,
            launchpad,
        } => get(username, github, url, launchpad, None, cli.dry_run)?,
        Command::Set {
            user,
            username,
            schedule,
            github,
            launchpad,
            url,
            expression,
            now,
        } => set(
            user,
            username,
            github,
            url,
            launchpad,
            schedule,
            expression,
            cli.dry_run,
            now,
        )?,
        Command::Job {} => jobs()?,
        Command::Remove { id } => remove(id)?,
    };

    Ok(())
}

/// Gets the keys from a provider
fn get(
    username: String,
    github: bool,
    gitlab: Option<Url>,
    launchpad: bool,
    user: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    info!("Getting data for {}", username);
    if Uid::current().is_root() {
        warn!("Running as root will add this to the root users authorized keys file")
    }

    if !dry_run {
        file::create_file_for_user(user)?;
    }

    let mut keys: Vec<String> = vec![];
    let urls: Vec<String> = util::create_urls(&username, github, launchpad, gitlab);
    for url in urls {
        let response = http::get_keys(&url)?;
        keys.append(&mut util::split_keys(&response));
    }

    let keys_to_add: Vec<String> = util::filter_keys(keys, file::get_current_keys(user)?);
    let num_keys_to_add: usize = keys_to_add.len();

    if !dry_run {
        file::write_keys(keys_to_add, user)?;
        println!("Added {} new keys", num_keys_to_add);
    } else {
        println!("Found {} new keys", num_keys_to_add);
    }

    Ok(())
}

fn set(
    user: String,
    username: String,
    github: bool,
    gitlab: Option<Url>,
    launchpad: bool,
    schedule: DefaultCron,
    expression: Option<String>,
    dry_run: bool,
    now: bool,
) -> anyhow::Result<()> {
    util::run_as_root()?;

    if !dry_run {
        file::create_file_for_user(Some(&user))?;
    }

    let urls: Vec<String> = util::create_urls(&username, github, launchpad, gitlab.clone());

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

    if !dry_run {
        for url in urls {
            match db::add_schedule(user.clone(), cron.clone(), url) {
                Ok(_) => println!("Successfully added import schedule"),
                Err(e) => error!("{}", e),
            };
        }
    } else {
        println!("Syntax Ok");
    }

    if now {
        get(username, github, gitlab, launchpad, Some(&user), dry_run)?;
    }

    Ok(())
}

fn jobs() -> anyhow::Result<()> {
    let jobs: Vec<db::Schedule> = db::get_schedule()?;
    let total_jobs = jobs.len();
    println!(
        "Found {} job{}",
        total_jobs,
        if total_jobs == 1 { "" } else { "s" }
    );
    if total_jobs > 0 {
        println!("{:<5}{:<15}{:<25}{:<45}", "ID", "User", "Cron", "Url");
        println!("{:-<90}", "");
        for job in jobs {
            println!(
                "{:<5}{:<15}{:<25}{:<40}",
                job.id, job.user, job.cron, job.url
            );
        }
    }
    Ok(())
}

fn remove(ids: Vec<i32>) -> anyhow::Result<()> {
    util::run_as_root()?;

    for id in ids {
        db::delete_schedule(id)?;
        println!("Removed job with id: {}", id);
    }

    return Ok(());
}

fn parse_cron(src: &str) -> Result<String, cron::error::Error> {
    Schedule::from_str(src)?;
    Ok(src.to_string())
}

fn parse_url(src: &str) -> Result<Url, ParseError> {
    if src.contains("http") {
        Url::parse(src)
    } else {
        Url::parse(&("https://".to_owned() + src))
    }
}
