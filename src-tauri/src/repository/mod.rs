use std::path::PathBuf;
use rusqlite::Connection;
use tauri::{AppHandle, Manager};

pub struct Database { _connection: Connection }
impl Database {
    pub fn open_for_app(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let directory = app.path().app_data_dir()?;
        std::fs::create_dir_all(&directory)?;
        Self::open(directory.join("nightingale.sqlite3"))
    }
    fn open(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let connection = Connection::open(path)?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        connection.execute_batch(include_str!("../../migrations/0001_initial.sql"))?;
        Ok(Self { _connection: connection })
    }
}

#[cfg(test)]
mod tests { use super::*; #[test] fn creates_minimum_schema_in_memory() { let connection = Connection::open_in_memory().expect("in-memory database"); connection.execute_batch(include_str!("../../migrations/0001_initial.sql")).expect("migration"); let count: i64 = connection.query_row("SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'security_policies'", [], |row| row.get(0)).expect("schema query"); assert_eq!(count, 1); } }
