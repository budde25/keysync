use anyhow::anyhow;
use cron;
use log::{info, warn};
use rusqlite::NO_PARAMS;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

use super::file;

/// A Schedule representation
#[derive(Debug)]
pub struct Schedule {
    pub id: i32,
    pub user: String,
    pub cron: String,
    pub url: String,
}

/// Creates the db and path to it if it doesn't already exist
pub fn create_db() -> anyhow::Result<()> {
    let path = file::get_schedule_path();
    if path.is_file() {
        return Ok(());
    }
    info!("Database does not exist, creating...");

    let parent = path.parent().unwrap();
    if !parent.is_dir() {
        fs::create_dir_all(parent)?;
    }

    let conn: Connection = Connection::open(path)?;
    conn.execute(
        "create table if not exists Schedule (
            id integer primary key,
            user text not null,
            cron text not null,
            url text not null
         )",
        NO_PARAMS,
    )?;
    Ok(())
}

/// Deletes a schedule with a given ID
pub fn delete_schedule(id: i32) -> anyhow::Result<()> {
    let path: PathBuf = file::get_schedule_path();
    let conn: Connection = Connection::open(path)?;

    conn.execute("DELETE FROM Schedule WHERE ID = ?1", params![id])?;
    Ok(())
}

/// Adds a new schedule to the database
pub fn add_schedule(user: String, cron: String, url: String) -> anyhow::Result<()> {
    let schedule: Schedule = validate_schdule(Schedule {
        id: 0,
        user,
        cron,
        url,
    })?;

    create_db()?;
    let path: PathBuf = file::get_schedule_path();
    let conn: Connection = Connection::open(path)?;

    conn.execute(
        "INSERT INTO Schedule (user, cron, url) VALUES (?1, ?2, ?3)",
        params![schedule.user, schedule.cron, schedule.url],
    )?;
    Ok(())
}

// Gets a list of schedules from the database
pub fn get_schedule() -> anyhow::Result<Vec<Schedule>> {
    let path = file::get_schedule_path();
    let conn: Connection = Connection::open(path)?;

    let mut stmt = conn.prepare("SELECT id, user, cron, url FROM Schedule")?;
    let schedule_iter = stmt.query_map(params![], |row| {
        Ok(Schedule {
            id: row.get(0)?,
            user: row.get(1)?,
            cron: row.get(2)?,
            url: row.get(3)?,
        })
    })?;

    let mut schedules: Vec<Schedule> = vec![];
    for schd in schedule_iter {
        match schd {
            Ok(s) => {
                let id: i32 = s.id;
                let valid = validate_schdule(s);
                match valid {
                    Ok(v) => schedules.push(v),
                    Err(e) => warn!("Database has invalid data, {} for ID {}", e, id),
                }
            }
            Err(e) => warn!("Invaid result from database, {}", e),
        }
    }
    Ok(schedules)
}

// Returns a Schedule if its valid, None if its not
fn validate_schdule(schedule: Schedule) -> anyhow::Result<Schedule> {
    if Url::parse(&schedule.url).is_err() {
        return Err(anyhow!("Invalid schedule"));
    }
    if cron::Schedule::from_str(&schedule.cron).is_err() {
        return Err(anyhow!("Invalid cron"));
    }
    Ok(schedule)
}
