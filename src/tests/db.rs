use super::*;
use assert_fs::prelude::*;
use std::time::Duration;

/// Tests that we can correctly compare last modified time, and they are same / diff in correct situations
#[test]
fn test_get_last_modified() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("file.txt");
    last_modified(file.path()).expect_err("File should not exist");
    file.touch().unwrap();
    let time = last_modified(file.path()).unwrap();
    let new_time = last_modified(file.path()).unwrap();
    assert_eq!(time, new_time);
    std::thread::sleep(Duration::from_millis(100)); // If we don't sleep its too fast lol
    file.write_str("foo").unwrap(); // New modification
    let new_time = last_modified(file.path()).unwrap();
    assert_ne!(time, new_time);
}

/// Tests that schedule only excepts good data
#[test]
fn test_new_schedule() {
    Schedule::new(None, "budd", "foo", "bar")
        .expect_err("Bad data should error");
    Schedule::new(None, "budd", "foo", "https://github.com")
        .expect_err("Bad data should error");
    Schedule::new(None, "budd", "@daily", "bar")
        .expect_err("Bad data should error");
    Schedule::new(Some(1), "budd", "@daily", "https://github.com")
        .expect("Data should pass");
}

/// Tests that the db file is properly created
#[test]
fn test_db_creation() {
    let temp = assert_fs::TempDir::new().unwrap();
    Database::open_path(temp.path().join("file.db"))
        .expect("Should create the database file"); // No file the first time
    let db = temp.child("file2.db");
    db.touch().unwrap();
    Database::open_path(db.path())
        .expect("Should setup database on empty file");
    let db_text = temp.child("file2.txt"); // Also try bad file ext
    db_text.touch().unwrap();
    Database::open_path(db.path())
        .expect("Should setup database on empty file, even for text");
}

/// Tests that the db file is properly created
#[test]
fn test_add_schedule() {
    let temp = assert_fs::TempDir::new().unwrap();
    let db = Database::open_path(temp.path().join("file.db"))
        .expect("Should create the database file");
    assert!(db
        .add_schedule("budd", "@daily", "https://github.com")
        .expect("No problems here"));
    assert!(!db
        .add_schedule("budd", "@daily", "https://github.com")
        .expect("Duplicates! return false"));
    assert!(db
        .add_schedule("budd", "@monthly", "https://github.com")
        .expect("new data no problem"));
}

/// Tests that we can read the schedule that we have created
#[test]
fn test_get_schedule() {
    let temp = assert_fs::TempDir::new().unwrap();
    let db = Database::open_path(temp.path().join("file.db"))
        .expect("Should create the database file");
    assert!(db
        .add_schedule("budd", "@daily", "https://github.com")
        .expect("No problems here"));
    assert!(db
        .add_schedule("budd", "@monthly", "https://github.com")
        .expect("No problem"));

    assert_eq!(db.get_schedules().unwrap().len(), 2);

    assert!(db
        .add_schedule("budd", "@weekly", "https://github.com")
        .expect("No problem"));

    assert_eq!(db.get_schedules().unwrap().len(), 3);
}

/// Tests that we can remove the schedule and it will error if we remove too much
#[test]
fn test_remove_schedule() {
    let temp = assert_fs::TempDir::new().unwrap();
    let db = Database::open_path(temp.path().join("file.db"))
        .expect("Should create the database file");
    assert!(db
        .add_schedule("budd", "@daily", "https://github.com")
        .expect("No problems here"));
    assert!(db
        .add_schedule("budd", "@monthly", "https://github.com")
        .expect("No problem"));

    assert!(db
        .add_schedule("budd", "@weekly", "https://github.com")
        .expect("No problem"));

    assert_eq!(db.get_schedules().unwrap().len(), 3);

    for i in db.get_schedules().expect("Should get") {
        let int = i.id.unwrap();
        db.delete_schedule(int).expect("They exist so they should delete");
    }

    assert_eq!(db.get_schedules().unwrap().len(), 0);

    db.delete_schedule(0).expect("Can't remove when there are none");
}
