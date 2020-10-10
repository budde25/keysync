use super::http;
use job_scheduler::Job;
use job_scheduler::JobScheduler;
use log::{debug, error, info, warn};
use std::{thread, time};
use url::Url;

use super::file;
use super::util;

pub fn start() -> anyhow::Result<()> {
    file::create_schedule_if_not_exist()?;

    let mut sched = JobScheduler::new();
    sched = schedule_tasks(sched)?;
    let sleep_time = time::Duration::from_millis(60 * 1000); // 1 minute
    let mut last_modified = file::schedule_last_modified()?;

    let mut n: u32 = 0;
    loop {
        println!("running for {} minute(s)", n);

        let new_last_modified = file::schedule_last_modified()?;
        if last_modified != new_last_modified {
            last_modified = new_last_modified;
            sched = JobScheduler::new();
            sched = schedule_tasks(sched)?;
        }

        sched.tick();

        thread::sleep(sleep_time);
        n = n + 1;
    }
}

fn schedule_tasks(mut sched: JobScheduler) -> anyhow::Result<JobScheduler> {
    println!("Schduling jobs");
    let schedule: Vec<String> = file::get_schedule()?;
    for item in schedule {
        // Skips any empty lines
        if item.trim().is_empty() {
            continue;
        }

        let data: Vec<String> = item.split("|").map(|x| x.to_owned()).collect();
        let user: String = data[0].clone();
        let cron: String = data[1].clone();
        let url: Url = Url::parse(&data[2])?;
        let username: String = data[3].clone();

        match cron.parse() {
            Ok(valid) => {
                match file::create_file_for_user(Some(&user)) {
                    Ok(_) => debug!("authorized keys file for {} exists or was created", user),
                    Err(e) => {
                        error!(
                            "Unable to create authorized keys file for user {}. {}",
                            user, e
                        );
                        continue;
                    }
                };

                sched.add(Job::new(valid, move || {
                    run_job(user.to_owned(), url.to_owned(), username.to_owned())
                }));
                println!("Scheduled item {}", item);
            }
            Err(_) => {
                println!("Cron {} failed to be parsed, skipping...", cron);
                continue;
            }
        }
    }

    return Ok(sched);
}

fn run_job(user: String, url: Url, username: String) -> () {
    let content = http::get_standard(&username, url);

    let clean_content = match content {
        Ok(clean) => clean,
        Err(_) => return (),
    };

    let keys = util::split_keys(&clean_content);
    let exist = match file::get_current_keys(Some(&user)) {
        Ok(key) => key,
        Err(_) => return (),
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
        Err(e) => eprint!("failed to write keys to file. {}", e)
    };
}
