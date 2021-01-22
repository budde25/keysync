use anyhow::Result;
use filetime::FileTime;
use job_scheduler::{Job, JobScheduler};
use log::{debug, error, info};
use std::{str::FromStr, thread::sleep, time::Duration};
use url::Url;

use super::db::{db_last_modified, Database, Schedule};
use super::file::AuthorizedKeys;
use super::http::Network;

/// An implementation of the daemon
pub struct Daemon {
    sleep_time: Duration,
    scheduler: JobScheduler<'static>,
    last_modified: FileTime,
}

impl Daemon {
    /// Creates a new daemon
    pub fn new() -> Result<Self> {
        Database::open()?;
        let scheduler: JobScheduler = JobScheduler::new();
        let sleep_time: Duration = Duration::from_secs(60); // 1 minute
        let last_modified: FileTime = db_last_modified()?;
        Ok(Daemon { sleep_time, scheduler, last_modified })
    }

    /// Starts the daemon
    pub fn start(&mut self) {
        self.schedule();
        loop {
            let modified: FileTime =
                db_last_modified().unwrap_or(self.last_modified);
            if self.last_modified != modified {
                self.last_modified = modified;
                let scheduler: JobScheduler = JobScheduler::new();
                // Schedule tasks
                self.schedule();
                self.scheduler = scheduler;
            }
            self.scheduler.tick();
            sleep(self.sleep_time);
        }
    }

    /// Adds a new job the the job schedule
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
                Ok(_) => debug!(
                    "authorized keys file for {} exists or was created",
                    &user
                ),
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

            let job = Job::new(cron, move || {
                run_job(user.to_owned(), url.to_owned())
            });
            self.scheduler.add(job);
        }
    }
}

/// Runs a job that is on the schedule
fn run_job(user: String, url: Url) {
    let network = Network::new();
    let keys = match network.get_keys(&url) {
        Ok(c) => c,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let authorized_keys = match AuthorizedKeys::open(Some(&user)) {
        Ok(a) => a,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    match authorized_keys.write_keys(keys, false) {
        Ok(count) => {
            println!("Added {} keys to a {} authorized_keys file", user, count)
        }
        Err(e) => error!("{}", e),
    };
}
