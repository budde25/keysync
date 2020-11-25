use anyhow::Result;
use filetime::FileTime;
use job_scheduler::{Job, JobScheduler};
use log::{debug, error, info};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use url::Url;

use super::db;
use super::file;
use super::http;
use super::util;

use db::{last_modified, Database, Schedule};
use file::AuthorizedKeys;
use http::Network;

pub struct Daemon {
    sleep_time: Duration,
    scheduler: JobScheduler<'static>,
    last_modifided: FileTime,
}

impl Daemon {
    pub fn new() -> Result<Self> {
        Database::open()?;
        let scheduler: JobScheduler = JobScheduler::new();
        let sleep_time: Duration = Duration::from_secs(60); // 1 minute
        let last_modifided: FileTime = last_modified()?;
        Ok(Daemon {
            sleep_time,
            scheduler,
            last_modifided,
        })
    }

    pub fn start(&mut self) {
        self.schedule();
        loop {
            let modified: FileTime = last_modified().unwrap_or(self.last_modifided);
            if self.last_modifided != modified {
                self.last_modifided = modified;
                let scheduler: JobScheduler = JobScheduler::new();
                // Schedule tasks
                self.schedule();
                self.scheduler = scheduler;
            }
            self.scheduler.tick();
            sleep(self.sleep_time);
        }
    }

    fn schedule(&mut self) {
        info!("Scheduling jobs");
        let database: Database = match Database::open() {
            Ok(d) => d,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };

        let schedules: Vec<Schedule> = match database.get_schedules() {
            Ok(d) => d,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };

        for schedule in schedules {
            let user: String = schedule.user.to_string();
            let url: Url = match Url::parse(&schedule.url) {
                Ok(u) => u,
                Err(e) => {
                    error!("{}", e);
                    continue;
                }
            };

            match AuthorizedKeys::open(Some(&user)) {
                Ok(_) => debug!("authorized keys file for {} exists or was created", &user),
                Err(e) => {
                    error!(
                        "Unable to create authorized keys file for user {}. {}",
                        &user, e
                    );
                    continue;
                }
            }

            let cron = match job_scheduler::Schedule::from_str(&schedule.cron) {
                Ok(c) => c,
                Err(e) => {
                    error!("{}", e);
                    continue;
                }
            };

            let job = Job::new(cron, move || run_job(user.to_owned(), url.to_owned()));
            self.scheduler.add(job);
        }
    }
}

fn run_job(user: String, url: Url) {
    let network = Network::new();
    let keys = match network.get_keys(&url) {
        Ok(c) => c,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let authorzed_keys = match AuthorizedKeys::open(Some(&user)) {
        Ok(a) => a,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let exist = match authorzed_keys.get_keys() {
        Ok(key) => key,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };
    let keys_to_add: Vec<String> = util::filter_keys(keys, exist);
    let num_keys_to_add: usize = keys_to_add.len();

    match authorzed_keys.write_keys(keys_to_add) {
        Ok(_) => println!(
            "Added {} keys to a {} authorized_keys file",
            user, num_keys_to_add
        ),
        Err(e) => error!("{}", e),
    };
}
