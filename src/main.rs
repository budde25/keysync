mod cli;
mod daemon;
mod db;
mod file;
mod http;
mod service;
mod util;

use anyhow::{anyhow, Result};
use clap::{value_t_or_exit, values_t_or_exit, ArgMatches};
use cron::Schedule;
use log::{info, warn};
use nix::unistd::Uid;
use url::Url;

use daemon::Daemon;
use db::Database;
use file::AuthorizedKeys;
use http::Network;

/// Main, returns () on success
fn main() -> Result<()> {
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

    // matches subcoommands, passing args to each
    match matches.subcommand() {
        ("get", Some(m)) => get(m)?,
        ("set", Some(m)) => set(m)?,
        ("jobs", Some(_)) => jobs()?,
        ("remove", Some(m)) => remove(m)?,
        ("daemon", Some(m)) => daemon(m)?,
        _ => unreachable!(),
    }

    Ok(())
}

/// Gets the keys from a provider
fn get(m: &ArgMatches) -> Result<()> {
    let username: String = value_t_or_exit!(m, "username", String);

    info!("Getting data for {}", username);
    if Uid::current().is_root() {
        warn!("Running as root will add this to the root users authorized keys file")
    }

    let mut gitlab_url: Option<Url> = None;
    let gitlab: bool = if let Some(u) = m.value_of("gitlab") {
        if !u.is_empty() {
            gitlab_url = Some(Url::parse(u)?);
        }
        true
    } else {
        false
    };

    let network: Network = Network::new();

    let keys: Vec<String> = network.get_keys_services(
        username,
        m.is_present("github"),
        m.is_present("launchpad"),
        gitlab,
        gitlab_url,
    )?;

    let user: Option<String> = if m.is_present("user") {
        Some(value_t_or_exit!(m, "user", String))
    } else {
        None
    };

    let authorized_keys: AuthorizedKeys = AuthorizedKeys::open(user)?;

    let dry_run = m.is_present("dry_run");
    let count = authorized_keys.write_keys(keys, dry_run)?;
    println!(
        "{} {} new keys",
        if dry_run { "Found" } else { "Added" },
        count
    );

    Ok(())
}

/// Adds a new schedule for the Systemd service to run
fn set(m: &ArgMatches) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    panic!("Platform not supported");

    // Get variables
    let user: String = value_t_or_exit!(m, "user", String);
    let username: String = value_t_or_exit!(m, "username", String);
    let cron: Schedule = if m.is_present("cron") {
        value_t_or_exit!(m, "cron", Schedule)
    } else {
        let default_cron = value_t_or_exit!(m, "schedule", cli::DefaultCron);
        default_cron.to_schedule()
    };

    let mut gitlab_url: Option<Url> = None;
    let gitlab: bool = if let Some(u) = m.value_of("gitlab") {
        if !u.is_empty() {
            gitlab_url = Some(Url::parse(u)?);
        }
        true
    } else {
        false
    };

    service::check()?;

    util::run_as_root()?;

    AuthorizedKeys::open(Some(&user))?;

    let urls: Vec<String> = http::create_urls(
        &username,
        m.is_present("github"),
        m.is_present("launchpad"),
        gitlab,
        gitlab_url,
    );

    if !m.is_present("dry-run") {
        let database = Database::open()?;
        for url in urls {
            database.add_schedule(&user, &cron.to_string(), &url)?;
            println!("Successfully added import schedule with url: {}", url);
        }
    } else {
        println!("Syntax Ok");
    }

    if m.is_present("now") {
        get(m)?;
    }

    Ok(())
}

/// Prints currently set jobs
fn jobs() -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    panic!("Platform not supported");

    service::check()?;

    // TODO check if service is running
    let database = Database::open()?;
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

/// Removes a schedule by id
fn remove(m: &ArgMatches) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    panic!("Platform not supported");

    service::check()?;

    util::run_as_root()?;
    let database = Database::open()?;
    let ids: Vec<u32> = values_t_or_exit!(m, "ids", u32);
    for id in ids {
        database.delete_schedule(id)?;
        println!("Removed job with id: {}", id);
    }

    Ok(())
}

/// To be run by Systemd, runs until stopped
fn daemon(m: &ArgMatches) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    panic!("Platform not supported");

    let install = m.is_present("install");
    let enable = m.is_present("enable");

    if install {
        service::install_service()?;
    }
    if enable && !service::enable_service()? {
        return Err(anyhow!("Failed to enable keysync service"));
    }

    if install || enable {
        return Ok(());
    }

    let mut daemon = Daemon::new()?;
    daemon.start();
    Ok(())
}
