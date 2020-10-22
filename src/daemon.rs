use super::http;
use job_scheduler::Job;
use job_scheduler::JobScheduler;
use log::{debug, error};
use std::{thread, time};
use std::str::FromStr;

use super::file;
use super::util;
use super::db;

pub fn start() -> anyhow::Result<()> {
    db::create_db()?;

    let mut sched: JobScheduler = JobScheduler::new();
    sched = schedule_tasks(sched)?;
    let sleep_time = time::Duration::from_millis(60 * 1000); // 1 minute
    let mut last_modified = file::schedule_last_modified()?;

    loop {
        let new_last_modified = file::schedule_last_modified()?;
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
    let schedule: Vec<db::Schedule> = db::get_schedule()?;
    for item in schedule {
        match file::create_file_for_user(Some(&item.user)) {
            Ok(_) => debug!("authorized keys file for {} exists or was created", &item.user),
            Err(e) => {
                error!(
                    "Unable to create authorized keys file for user {}. {}",
                    &item.user, e
                );
                continue;
            }
        }

        sched.add(Job::new(cron::Schedule::from_str(&item.cron).unwrap(), move || {
            run_job(item.user.to_owned(), item.url.to_owned())
        }));
        println!("Scheduled item");
    }

    Ok(sched)
}

fn run_job(user: String, url: String) {
    let content = match http::get_keys(&url) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let keys = util::split_keys(&content);
    let exist = match file::get_current_keys(Some(&user)) {
        Ok(key) => key,
        Err(_) => return,
    };
    let keys_to_add: Vec<String> = util::filter_keys(keys, exist);
    let num_keys_to_add: usize = keys_to_add.len();

    match file::create_file_for_user(Some(&user)) {
        Ok(_) => (),
        Err(e) => eprint!("failed to create file for user. {}", e),
    };

    match file::write_keys(keys_to_add, Some(&user)) {
        Ok(_) => println!(
            "Added {} to {}'s authorized_keys file",
            num_keys_to_add, user
        ),
        Err(e) => eprint!("failed to write keys to file. {}", e),
    };
}