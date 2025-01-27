use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};

pub const CREATE_DB: &str = "CREATE TABLE IF NOT EXISTS data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    url TEXT NOT NULL,
    passphrase TEXT NOT NULL,
    NOTES TEXT
)";

const GET_ENTRY_BY_ID: &str = "SELECT id, username, url, passphrase, notes FROM data where id = ?";
const DELETE_ENTRY_BY_ID: &str = "DELETE FROM data where id = ?";
const INSERT_ENTRY: &str = "INSERT INTO data (username, url, passphrase, notes)
    VALUES (?1, ?2, ?3, ?4) RETURNING id, username, url, passphrase, notes";
const GET_ENTRIES: &str = "SELECT id, username, url, passphrase, notes FROM data";

#[derive(Debug, PartialEq, Clone)]
pub struct PassEntry {
    pub id: i64,
    pub username: String,
    pub url: String,
    pub passphrase: String,
    pub notes: String,
}

pub fn get_entries_from_db(db_con: Arc<Mutex<sqlite::Connection>>) -> Result<Vec<PassEntry>> {
    let con = db_con
        .lock()
        .map_err(|_| anyhow!("error while locking db connection"))?;
    let mut entries: Vec<PassEntry> = vec![];
    let mut stmt = con.prepare(GET_ENTRIES)?;

    for row in stmt.iter() {
        let row = row?;
        let id = row.read::<i64, _>(0);
        let username = row.read::<&str, _>(1);
        let url = row.read::<&str, _>(2);
        let passphrase = row.read::<&str, _>(3);
        let notes = row.read::<&str, _>(4);

        entries.push(PassEntry {
            id,
            username: username.to_owned(),
            url: url.to_owned(),
            passphrase: passphrase.to_owned(),
            notes: notes.to_owned(),
        });
    }
    Ok(entries)
}
pub fn get_entry_from_db(
    db_con: Arc<Mutex<sqlite::Connection>>,
    entry_id: i64,
) -> Result<Option<PassEntry>> {
    let con = db_con
        .lock()
        .map_err(|_| anyhow!("error while locking db connection"))?;
    let mut stmt = con.prepare(GET_ENTRY_BY_ID)?;
    stmt.bind((1, entry_id))?;

    if stmt.next()? == sqlite::State::Row {
        let id = stmt.read::<i64, _>(0)?;
        let username = stmt.read::<String, _>(1)?;
        let url = stmt.read::<String, _>(2)?;
        let passphrase = stmt.read::<String, _>(3)?;
        let notes = stmt.read::<String, _>(4)?;

        return Ok(Some(PassEntry {
            id,
            username,
            url,
            passphrase,
            notes,
        }));
    }
    Ok(None)
}

pub fn insert_entry_to_db(
    db_con: Arc<Mutex<sqlite::Connection>>,
    entry: PassEntry,
) -> Result<PassEntry> {
    let con = db_con
        .lock()
        .map_err(|_| anyhow!("error while locking db connection"))?;
    let mut stmt = con.prepare(INSERT_ENTRY)?;
    stmt.bind((1, entry.username.as_str()))?;
    stmt.bind((2, entry.url.as_str()))?;
    stmt.bind((3, entry.passphrase.as_str()))?;
    stmt.bind((4, entry.notes.as_str()))?;

    if stmt.next()? == sqlite::State::Row {
        let id = stmt.read::<i64, _>(0)?;
        let username = stmt.read::<String, _>(1)?;
        let url = stmt.read::<String, _>(2)?;
        let passphrase = stmt.read::<String, _>(3)?;
        let notes = stmt.read::<String, _>(4)?;

        return Ok(PassEntry {
            id,
            username,
            url,
            passphrase,
            notes,
        });
    }

    Err(anyhow!("error while inserting entry"))
}

pub fn delete_entry_from_db(db_con: Arc<Mutex<sqlite::Connection>>, entry_id: i64) -> Result<()> {
    let con = db_con
        .lock()
        .map_err(|_| anyhow!("error while locking db connection"))?;
    let mut stmt = con.prepare(DELETE_ENTRY_BY_ID)?;
    stmt.bind((1, entry_id))?;

    if stmt.next()? == sqlite::State::Done {
        Ok(())
    } else {
        Err(anyhow!("error while deleting entry with id {}", entry_id))
    }
}
