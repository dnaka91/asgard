use std::ops::{Deref, DerefMut};

use anyhow::{Context, Result};
use r2d2::{ManageConnection, Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;

fn init_connection(conn: &mut Connection) -> Result<(), rusqlite::Error> {
    conn.pragma_update(None, "busy_timeout", 1000)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "wal_autocheckpoint", 1000)?;
    conn.pragma_update(None, "wal_checkpoint", "TRUNCATE")?;
    Ok(())
}

pub struct DbConnPool(Pool<SqliteConnectionManager>);

impl DbConnPool {
    pub fn get(&self) -> Result<DbConn> {
        self.0.get().map(DbConn).map_err(Into::into)
    }
}

pub struct DbConn(PooledConnection<SqliteConnectionManager>);

impl Deref for DbConn {
    type Target = Connection;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DbConn {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(debug_assertions)]
const DB_PATH: &str = "data.db";
#[cfg(not(debug_assertions))]
const DB_PATH: &str = concat!("/var/lib/", env!("CARGO_PKG_NAME"), "/data.db");

pub fn create_pool() -> Result<DbConnPool> {
    let manager = if cfg!(test) {
        SqliteConnectionManager::memory()
    } else {
        #[cfg(not(debug_assertions))]
        std::fs::create_dir_all(concat!("/var/lib/", env!("CARGO_PKG_NAME")))?;
        SqliteConnectionManager::file(DB_PATH)
    }
    .with_init(init_connection);

    // First create a single connection to make sure all eventually locking PRAGMAs are run,
    // so we don't get any errors when spinning up the pool.
    manager.connect().context("failed to initialize database")?;

    let pool = Pool::builder().build(manager)?;

    Ok(DbConnPool(pool))
}
