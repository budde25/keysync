use clap::{App, AppSettings, Arg, SubCommand};
use cron::Schedule;
use nix::unistd::User;
use std::str::FromStr;
use url::Url;


/// Struct of default key downloading schedules
#[derive(Debug)]
pub enum DefaultCron {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

/// FromStr implementation
impl FromStr for DefaultCron {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hourly" => Ok(DefaultCron::Hourly),
            "daily" => Ok(DefaultCron::Daily),
            "weekly" => Ok(DefaultCron::Weekly),
            "monthly" => Ok(DefaultCron::Monthly),
            _ => Err("no match"),
        }
    }
}

/// Turns the defaults into a Cron Schedule
impl DefaultCron {
    pub fn to_schedule(&self) -> Schedule {
        match self {
            DefaultCron::Hourly => cron::Schedule::from_str("@hourly").unwrap(),
            DefaultCron::Daily => cron::Schedule::from_str("@daily").unwrap(),
            DefaultCron::Weekly => cron::Schedule::from_str("@weekly").unwrap(),
            DefaultCron::Monthly => cron::Schedule::from_str("@monthly").unwrap(),
        }
    }
}

/// Generate the programs CLI
pub fn app() -> App<'static, 'static> {
    let settings = [
        AppSettings::ColoredHelp,               // Displays color, (not on windows)
        AppSettings::InferSubcommands,          // Hints
        AppSettings::VersionlessSubcommands,    // No -V on subcommands
    ];

    let get = SubCommand::with_name("get")
        .about("Retrieves a key from an online source")
        .arg(
            Arg::with_name("username")
                .help("The username of the account")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("github")
                .help("Retrieve from GitHub (default)")
                .short("g")
                .long("github"),
        )
        .arg(
            Arg::with_name("launchpad")
                .help("Retrieve from Launchpad")
                .short("l")
                .long("launchpad"),
        )
        .arg(
            Arg::with_name("gitlab")
                .help("Retrieve from GitLab with optional URL")
                .value_name("URL")
                .short("h")
                .long("gitlab")
                .empty_values(true)
                .validator(is_url_or_empty),
        );

    let set = SubCommand::with_name("set")
        .about("Add an automatic job")
        .arg(
            Arg::with_name("user")
                .help("The local user account")
                .required(true)
                .index(1)
                .validator(is_user),
        )
        .arg(
            Arg::with_name("username")
                .help("The username of the account to get keys from")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("schedule")
                .help("Default schedules")
                .required(true)
                .index(3)
                .possible_values(&["Hourly", "Daily", "Weekly", "Monthly"])
                .case_insensitive(true)
                .conflicts_with("cron"),
        )
        .arg(
            Arg::with_name("cron")
                .help("A custom schedule in cron format Ex: '* * * * * *', conflicts with schedule")
                .conflicts_with("schedule")
                .value_name("CRON")
                .short("c")
                .long("cron")
                .validator(is_cron),
        )
        .arg(
            Arg::with_name("now")
                .help("Also runs in addition to adding to schedule")
                .short("n")
                .long("now"),
        )
        .arg(
            Arg::with_name("github")
                .help("Retrieve from GitHub (default)")
                .short("g")
                .long("github"),
        )
        .arg(
            Arg::with_name("launchpad")
                .help("Retrieve from Launchpad")
                .short("l")
                .long("launchpad"),
        )
        .arg(
            Arg::with_name("gitlab")
                .help("Retrieve from GitLab with optional URL")
                .value_name("URL")
                .short("h")
                .long("gitlab")
                .empty_values(true)
                .validator(is_url_or_empty),
        );

    let remove = SubCommand::with_name("remove")
        .about("Remove job(s) by ID")
        .arg(
            Arg::with_name("ids")
                .help("Job IDs to remove")
                .multiple(true)
                .validator(is_number),
        );

    let jobs = SubCommand::with_name("jobs").about("list enabled jobs");

    let daemon = SubCommand::with_name("daemon")
        .about("Runs job daemon in background, No need to run, systemd will manage for you")
        .arg(
            Arg::with_name("install")
                .help("Install the Systemd service file")
                .long("install"),
        )
        .arg(
            Arg::with_name("enable")
                .help("Enable the keysync service")
                .long("enable"),
        );

    App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .global_settings(&settings)
        .subcommand(get)
        .subcommand(set)
        .subcommand(remove)
        .subcommand(jobs)
        .subcommand(daemon)
        .arg(
            Arg::with_name("dry_run")
                .help("Runs the commands without committing the changes")
                .short("d")
                .long("dry-run")
                .global(true),
        )
        .arg(
            Arg::with_name("verbosity")
                .help("Verbose mode (-v, -vv, -vvv)")
                .short("v")
                .multiple(true)
                .global(true),
        )
}

/// Custom validator, returns () if val is u32, error otherwise
fn is_number(val: String) -> Result<(), String> {
    val.parse::<u32>().map(|_| ()).map_err(|x| x.to_string())
}

/// Custom validator, returns () if val is a valid url or empty string, error otherwise
fn is_url_or_empty(val: String) -> Result<(), String> {
    if val.is_empty() {
        return Ok(());
    }
    val.parse::<Url>().map(|_| ()).map_err(|x| x.to_string())
}

/// Custom validator, returns () if val is valid cron schedule, error otherwise
fn is_cron(val: String) -> Result<(), String> {
    val.parse::<Schedule>()
        .map(|_| ())
        .map_err(|x| x.to_string())
}

/// Custom validator, returns () if val the user exists on the system, error otherwise
fn is_user(val: String) -> Result<(), String> {
    let result = User::from_name(&val).map_err(|x| x.to_string())?;
    if result.is_none() {
        Err(format!("user '{}' does not exist", val))
    } else {
        Ok(())
    }
}