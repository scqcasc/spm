use std::path::Path;
use rusqlite::{named_params, params, Connection, Result};

#[derive(Debug, Clone)]
pub struct PassEntry {
    pub id: i32,
    pub username: String,
    pub url: String,
    pub passphrase: String,
    pub notes: String,
}

pub fn create_database(db: &Path) -> Result<()> {
    let conn = Connection::open(db)?;

    // Create the data table
    conn.execute(
    "CREATE TABLE IF NOT EXISTS data (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username TEXT NOT NULL,
        url TEXT NOT NULL,
        passphrase TEXT NOT NULL,
        NOTES TEXT
    )",[],
    )?;
    Ok(())
}


pub fn insert_data(data: &PassEntry, db: &Path) -> Result<()> {
    let mut conn = Connection::open(db)?;

    let tx = conn.transaction()?; // Start a transaction

    tx.execute(
        "INSERT INTO data (username, url, passphrase, notes) 
        VALUES (?1, ?2, ?3, ?4)", 
        params![data.username, data.url, data.passphrase, data.notes]
    )?;

    tx.commit()?; // Commit transaction if successful

    println!(
        "Inserted into DB: username='{}', url='{}', passphrase='{}', notes='{}'",
        data.username, data.url, data.passphrase, data.notes
    );

    Ok(())
}

pub fn query_data(data: &PassEntry, db: &Path) -> Result<PassEntry> {
    let conn = Connection::open(db)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, username, url, passphrase, notes 
         FROM data WHERE username = :username AND url = :url;"
    )?;

    let entry = stmt.query_row(
        named_params! {
            ":username": data.username,
            ":url": data.url
        },
        |row| {
            Ok(PassEntry {
                id: row.get(0)?,
                username: row.get(1)?,
                url: row.get(2)?,
                passphrase: row.get(3)?,
                notes: row.get(4)?,
            })
        },
    )?;

    Ok(entry)
}

pub fn query_all(db: &Path) -> Result<Vec<PassEntry>> {
    let conn = Connection::open(db)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, username, url, passphrase, notes 
         FROM data;"
    )?;

    let rows = stmt.query_map([],
        |row| {
            Ok(PassEntry {
                id: row.get(0)?,
                username: row.get(1)?,
                url: row.get(2)?,
                passphrase: row.get(3)?,
                notes: row.get(4)?,
            })
    })?;
    let mut entries = Vec::new();
    for entry in rows {
        entries.push(entry?); // Unwrap each row and push into Vec
    }
    Ok(entries)
}

pub fn query_all_sl(db: &Path) -> Result<Vec<PassEntry>, Box<dyn std::error::Error>> {
    let conn = Connection::open(db)?;
    let mut stmt = conn.prepare(
        "SELECT id, username, url, passphrase, notes FROM data;"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(PassEntry {
            id: row.get(0)?,
            username: row.get::<_, String>(1)?.into(),  // Convert String to SharedString
            url: row.get::<_, String>(2)?.into(),
            passphrase: row.get::<_, String>(3)?.into(),
            notes: row.get::<_, String>(4)?.into(),
        })
    })?;

    let mut entries = Vec::new();
    for entry in rows {
        entries.push(entry?);
    }
    Ok(entries)
}