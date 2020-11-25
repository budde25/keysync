use clap::{value_t_or_exit, values_t_or_exit, ArgMatches};
use cron::Schedule;
use log::{info, warn};
use nix::unistd::Uid;
use url::Url;

mod cli;
mod daemon;
mod db;
mod file;
mod http;
mod util;

fn main() -> anyhow::Result<()> {
    let matches = cli::app().get_matches();

    // Sets the log level
    match matches.occurrences_of("verbosity") {
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
    }
    info!("Logger has been initialized");

    match matches.subcommand() {
        ("get", Some(m)) => get(m)?,
        ("set", Some(m)) => set(m)?,
        ("jobs", Some(_)) => jobs()?,
        ("remove", Some(m)) => remove(m)?,
        ("daemon", Some(_)) => daemon::start()?,
        _ => unreachable!(),
    }

    Ok(())
}

// Gets the keys from a provider
fn get(m: &ArgMatches) -> anyhow::Result<()> {
    let username: String = value_t_or_exit!(m, "username", String);
    let gitlab: Option<Url> = if m.is_present("gitlab") {
        Some(value_t_or_exit!(m, "gitlab", Url))
    } else {
        None
    };
    let github: bool = m.is_present("github");
    let launchpad: bool = m.is_present("launchpad");
    let dry_run: bool = m.is_present("dry_run");

    info!("Getting data for {}", username);
    if Uid::current().is_root() {
        warn!("Running as root will add this to the root users authorized keys file")
    }

    let auth: file::AuthorizedKeys = file::AuthorizedKeys::open(None)?;

    let mut keys: Vec<String> = vec![];
    let urls: Vec<String> = util::create_urls(&username, github, launchpad, gitlab);
    let network = http::Network::new();
    for url in urls {
        let response = network.get_keys(&url)?;
        keys.append(&mut util::split_keys(&response));
    }

    let keys_to_add: Vec<String> = util::filter_keys(keys, auth.get_keys()?);
    let num_keys_to_add: usize = keys_to_add.len();

    if !dry_run {
        auth.write_keys(keys_to_add)?;
        println!("Added {} new keys", num_keys_to_add);
    } else {
        println!("Found {} new keys", num_keys_to_add);
    }

    Ok(())
}

fn set(m: &ArgMatches) -> anyhow::Result<()> {
    // Get variables
    let user: String = value_t_or_exit!(m, "user", String);
    let username: String = value_t_or_exit!(m, "username", String);
    let cron: Schedule = if m.is_present("cron") {
        value_t_or_exit!(m, "cron", Schedule)
    } else {
        let default_cron = value_t_or_exit!(m, "schedule", cli::DefaultCron);
        default_cron.to_schedule()
    };
    let gitlab: Option<Url> = if m.is_present("gitlab") {
        Some(value_t_or_exit!(m, "gitlab", Url))
    } else {
        None
    };
    let github: bool = m.is_present("github");
    let launchpad: bool = m.is_present("launchpad");
    let dry_run: bool = m.is_present("dry-run");

    util::run_as_root()?;

    file::AuthorizedKeys::open(Some(&user))?;

    let urls: Vec<String> = util::create_urls(&username, github, launchpad, gitlab.clone());

    if !dry_run {
        let database = db::Database::open()?;
        for url in urls {
            database.add_schedule(&user, &cron.to_string(), &url)?;
            println!("Successfully added import schedule with url: {}", url);
        }
    } else {
        println!("Syntax Ok");
    }

    // if now {
    //     get(username, github, gitlab, launchpad, Some(&user), dry_run)?;
    // }

    Ok(())
}

fn jobs() -> anyhow::Result<()> {
    let database = db::Database::open()?;
    let jobs: Vec<db::Schedule> = database.get_schedules()?;
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
                job.id.unwrap_or(0),
                job.user,
                job.cron,
                job.url
            );
        }
    }
    Ok(())
}

fn remove(m: &ArgMatches) -> anyhow::Result<()> {
    util::run_as_root()?;
    let database = db::Database::open()?;
    let ids: Vec<u32> = values_t_or_exit!(m, "ids", u32);
    for id in ids {
        database.delete_schedule(id)?;
        println!("Removed job with id: {}", id);
    }

    return Ok(());
}
