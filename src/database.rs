use crate::DB_NAME;
use anyhow::Result;
use rusqlite::{types::Null, Connection};
use std::{path::Path, sync::Arc, time::UNIX_EPOCH};
use teloxide::types::User;
use tokio::sync::Mutex;

pub async fn setup_db() -> Result<Connection> {
    let exists = Path::new(DB_NAME).exists();
    let conn = Connection::open(DB_NAME)?;
    if !exists {
        conn.execute(
            "CREATE TABLE user (
                id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                username TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE shit_session (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                duration INTEGER,
                location TEXT,
                haemorrhoids BOOLEAN NOT NULL,
                constipated BOOLEAN NOT NULL,
                FOREIGN KEY(user_id) REFERENCES user(id)
            )",
            [],
        )?;
    }
    return Ok(conn);
}

pub async fn create_or_update_user(conn: Arc<Mutex<Connection>>, user: &User) -> Result<()> {
    let conn = conn.lock().await;
    let username = if user.username.is_some() {
        user.username.as_ref().unwrap().clone()
    } else {
        user.full_name()
    };
    conn.execute(
        "INSERT INTO user(id, username) VALUES (?, ?) ON CONFLICT(id) DO UPDATE SET username = ?",
        rusqlite::params![user.id.0, username, username],
    )?;
    return Ok(());
}

/// With both `duration` and `location` as NULL
pub async fn insert_shitting_session(
    conn: Arc<Mutex<Connection>>,
    user: &User,
    haemorrhoids: bool,
    constipated: bool,
) -> Result<()> {
    let conn = conn.lock().await;
    conn.execute(
        "INSERT INTO shit_session(
            user_id,
            timestamp,
            duration,
            location,
            haemorrhoids,
            constipated
        ) VALUES (
            ?,
            ?,
            ?,
            ?,
            ?,
            ?
        )",
        rusqlite::params![
            user.id.0,
            UNIX_EPOCH.elapsed()?.as_secs(),
            Null,
            Null,
            haemorrhoids,
            constipated
        ],
    )?;
    return Ok(());
}

// With `location` set as NULL
pub async fn insert_shitting_session_with_duration(
    conn: Arc<Mutex<Connection>>,
    user: &User,
    duration: u64,
    haemorrhoids: bool,
    constipated: bool,
) -> Result<()> {
    let conn = conn.lock().await;
    conn.execute(
        "INSERT INTO shit_session(
            user_id,
            timestamp,
            duration,
            location,
            haemorrhoids,
            constipated
        ) VALUES (
            ?,
            ?,
            ?,
            ?,
            ?,
            ?
        )",
        rusqlite::params![
            user.id.0,
            UNIX_EPOCH.elapsed()?.as_secs(),
            duration,
            Null,
            haemorrhoids,
            constipated
        ],
    )?;
    return Ok(());
}

pub async fn insert_shitting_session_with_location(
    conn: Arc<Mutex<Connection>>,
    user: &User,
    duration: u64,
    location: &str,
    haemorrhoids: bool,
    constipated: bool,
) -> Result<()> {
    let conn = conn.lock().await;
    conn.execute(
        "INSERT INTO shit_session(
            user_id,
            timestamp,
            duration,
            location,
            haemorrhoids,
            constipated
        ) VALUES (
            ?,
            ?,
            ?,
            ?,
            ?,
            ?
        )",
        rusqlite::params![
            user.id.0,
            UNIX_EPOCH.elapsed()?.as_secs(),
            duration,
            location,
            haemorrhoids,
            constipated
        ],
    )?;
    return Ok(());
}
