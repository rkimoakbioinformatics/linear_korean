use crate::error::*;
use crate::file::get_root_dir;
use rusqlite::{params, Connection};
use std::path::PathBuf;

static SETTINGS_DB_FILENAME: &str = "settings.sqlite3";

fn get_settings_db_p() -> PathBuf {
    let mut p = get_root_dir();
    p.push(SETTINGS_DB_FILENAME);
    p
}

fn sqlite_error(context: &str, err: rusqlite::Error) -> Error {
    Error::Config(ConfigError {
        msg: format!("{}: {:?}", context, err),
    })
}

fn open_settings_db() -> Result<Connection, Error> {
    let db_path = get_settings_db_p();
    let conn = Connection::open(&db_path).map_err(|e| {
        Error::Config(ConfigError {
            msg: format!("Cannot open settings DB {:?}: {:?}", db_path, e),
        })
    })?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            name TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| sqlite_error("Cannot create settings table", e))?;
    Ok(conn)
}

pub fn init_settings_db() -> Result<(), Error> {
    let _ = open_settings_db()?;
    Ok(())
}

pub fn get_setting_value(name: &str) -> Result<Option<String>, Error> {
    if name.trim().is_empty() {
        return Ok(None);
    }
    let conn = open_settings_db()?;
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE name = ?1")
        .map_err(|e| sqlite_error("Cannot prepare get_setting query", e))?;
    let mut rows = stmt
        .query(params![name])
        .map_err(|e| sqlite_error("Cannot execute get_setting query", e))?;
    if let Some(row) = rows
        .next()
        .map_err(|e| sqlite_error("Cannot fetch get_setting row", e))?
    {
        let value: String = row
            .get(0)
            .map_err(|e| sqlite_error("Cannot decode get_setting value", e))?;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

pub fn set_setting_value(name: &str, value: &str) -> Result<(), Error> {
    if name.trim().is_empty() {
        return Err(Error::Config(ConfigError {
            msg: "Setting name cannot be empty".to_string(),
        }));
    }
    let conn = open_settings_db()?;
    conn.execute(
        "INSERT INTO settings (name, value)
         VALUES (?1, ?2)
         ON CONFLICT(name) DO UPDATE SET value = excluded.value",
        params![name, value],
    )
    .map_err(|e| sqlite_error("Cannot upsert setting", e))?;
    Ok(())
}
