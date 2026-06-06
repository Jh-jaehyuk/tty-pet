pub mod migrations;
pub mod models;
pub mod repository;

use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

use crate::config;

pub fn open(path: &Path) -> Result<Connection> {
    config::ensure_parent_dir(path)?;
    let connection = Connection::open(path)?;
    migrations::run(&connection)?;

    Ok(connection)
}
