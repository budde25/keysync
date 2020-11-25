use anyhow::{anyhow, Context, Result};
use cron;
use filetime::FileTime;
use rusqlite::{params, Connection, NO_PARAMS};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::usize;
use std::{fmt, fs};
use url::Url;

/// A Schedule representation
#[derive(Debug, PartialEq)]
pub struct Schedule {
    pub id: Option<u32>,
    pub user: String,
    pub cron: String,
    pub url: String,
}

impl fmt::Display for Schedule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.id {
            Some(i) => write!(
                f,
                "[id: {}, user: {}, cron: {}, url: {}]",
                i, self.user, self.cron, self.url
            ),
            None => write!(
                f,
                "[user: {}, cron: {}, url: {}]",
                self.user, self.cron, self.url
            ),
        }
    }
}

impl Schedule {
    /// Creates a new schedule object and also verifies the types
    pub fn new<S: AsRef<str>>(id: Option<u32>, user: S, cron: S, url: S) -> Result<Self> {
        if let Err(e) = cron::Schedule::from_str(cron.as_ref()) {
            return Err(anyhow!(
                "Failed to parse cron expression: {}",
                cron.as_ref()
            ));
        };
        Url::from_str(url.as_ref())?;
        Ok(Schedule {
            id,
            user: user.as_ref().to_string(),
            cron: cron.as_ref().to_string(),
            url: url.as_ref().to_string(),
        })
    }
}

/// Object representing a database
pub struct Database {
    connection: Connection,
}

impl Database {
    /// Opens and create (if nonesistant) a database in the default path;
    pub fn open() -> Result<Self> {
        // TODO allow for windows/mac compatbility
        Database::open_path("/usr/share/keysync/schedule.db")
    }

    /// Opens and create (if nonesistant) a database with a given path;
    fn open_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let p = path.as_ref().to_owned();

        // Setup path
        if !p.is_file() {
            // Setup directory that will contain the database
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "Failed to create path that will contain database: {}",
                        parent.display()
                    )
                })?;
            }
            // Connection::open will create the database file
        }

        let conn: Connection = Connection::open(path).with_context(|| {
            format!(
                "Failed to open database connection with path: {}",
                p.display()
            )
        })?;

        // Create table ONLY if it doesn't exist
        conn.execute(
            "create table if not exists Schedule (
            id integer primary key,
            user text not null,
            cron text not null,
            url text not null
            )",
            NO_PARAMS,
        )
        .context("Error initalizing new database")?;
        Ok(Database { connection: conn })
    }

    /// Deletes a schedule with a given ID
    pub fn delete_schedule(&self, id: u32) -> Result<()> {
        self.connection
            .execute("DELETE FROM Schedule WHERE ID = ?1", params![id])
            .with_context(|| format!("Error deleting databse entry with id: {}", id))?;
        Ok(())
    }

    /// Adds a new schedule to the database
    pub fn add_schedule<S: AsRef<str>>(&self, user: S, cron: S, url: S) -> Result<()> {
        let schedule = Schedule::new(None, user, cron, url)?;
        self.connection
            .execute(
                "INSERT INTO Schedule (user, cron, url) VALUES (?1, ?2, ?3)",
                params![schedule.user, schedule.cron, schedule.url],
            )
            .with_context(|| format!("Error inserting schedule {} into the database", schedule))?;
        Ok(())
    }

    /// Gets a list of schedules from the database
    pub fn get_schedules(&self) -> Result<Vec<Schedule>> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, user, cron, url FROM Schedule")?;
        let schedule_iter = stmt.query_map(params![], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;

        Ok(schedule_iter
            .filter_map(|x| x.ok())
            .map(|x: (u32, String, String, String)| Schedule::new(Some(x.0), x.1, x.2, x.3))
            .filter_map(|x| x.ok())
            .collect())
    }
}

pub fn last_modified() -> Result<FileTime> {
    let path = PathBuf::from("/usr/share/keysync/schedule.db");
    let metadata = fs::metadata(&path).with_context(|| {
        format!(
            "Error getting filetime of file with path: {}",
            &path.display()
        )
    })?;
    Ok(FileTime::from_last_modification_time(&metadata))
}
