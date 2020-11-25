use super::http;
use job_scheduler::Job;
use job_scheduler::JobScheduler;
use log::{debug, error};
use std::str::FromStr;
use std::{thread, time};

use super::db;
use super::file;
use super::util;

pub fn start() -> anyhow::Result<()> {
    {
        // Putting it its own scope allow it to release the connection right after creating the database
        db::Database::open()?;
    }

    let mut sched: JobScheduler = JobScheduler::new();
    sched = schedule_tasks(sched)?;
    let sleep_time = time::Duration::from_millis(60 * 1000); // 1 minute
    let mut last_modified = db::last_modified()?;

    loop {
        let new_last_modified = db::last_modified()?;
        if last_modified != new_last_modified {
            last_modified = new_last_modified;
            sched = JobScheduler::new();
            sched = schedule_tasks(sched)?;
        }

        sched.tick();

        thread::sleep(sleep_time);
    }
}

fn schedule_tasks(mut sched: JobScheduler) -> anyhow::Result<JobScheduler> {
    println!("Scheduling jobs");
    let database = db::Database::open()?;
    let schedule: Vec<db::Schedule> = database.get_schedules()?;
    for item in schedule {
        match file::AuthorizedKeys::open(Some(&item.user)) {
            Ok(_) => debug!(
                "authorized keys file for {} exists or was created",
                &item.user
            ),
            Err(e) => {
                error!(
                    "Unable to create authorized keys file for user {}. {}",
                    &item.user, e
                );
                continue;
            }
        };

        sched.add(Job::new(
            job_scheduler::Schedule::from_str(&item.cron).unwrap(),
            move || run_job(item.to_string(), item.url.to_owned()),
        ));
        println!("Scheduled item");
    }

    Ok(sched)
}

fn run_job(user: String, url: String) {
    let network = http::Network::new();
    let keys = match network.get_keys(&url) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let auth = file::AuthorizedKeys::open(Some(&user)).unwrap();
    let exist = match auth.get_keys() {
        Ok(key) => key,
        Err(_) => return,
    };
    let keys_to_add: Vec<String> = util::filter_keys(keys, exist);
    let num_keys_to_add: usize = keys_to_add.len();

    match auth.write_keys(keys_to_add) {
        Ok(_) => println!("Added {} to a authorized_keys file", num_keys_to_add),
        Err(e) => eprint!("failed to write keys to file. {}", e),
    };
}
