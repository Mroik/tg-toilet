use crate::{bot::ShitSession, DB_NAME};
use anyhow::Result;
use rusqlite::{params, types::Null, Connection};
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
        params![user.id.0, username, username],
    )?;
    return Ok(());
}

/// With both `duration` and `location` as NULL
pub async fn insert_shitting_session(
    conn: Arc<Mutex<Connection>>,
    user: &User,
    haemorrhoids: bool,
    constipated: bool,
) -> Result<ShitSession> {
    let conn = conn.lock().await;
    let timestamp = UNIX_EPOCH.elapsed()?.as_secs();
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
        params![user.id.0, timestamp, Null, Null, haemorrhoids, constipated],
    )?;

    Ok(conn.query_row(
        "SELECT * FROM shit_session WHERE user_id = ? AND timestamp = ?",
        params![user.id.0, timestamp],
        |row| {
            Ok(ShitSession {
                id: row.get(0)?,
                user_id: row.get(1)?,
                timestamp,
                duration: row.get(3)?,
                location: row.get(4)?,
                haemorrhoids,
                constipated,
            })
        },
    )?)
}

/// With `location` set as NULL
pub async fn insert_shitting_session_with_duration(
    conn: Arc<Mutex<Connection>>,
    user: &User,
    duration: u64,
    haemorrhoids: bool,
    constipated: bool,
) -> Result<ShitSession> {
    let conn = conn.lock().await;
    let timestamp = UNIX_EPOCH.elapsed()?.as_secs();
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
        params![
            user.id.0,
            timestamp,
            duration,
            Null,
            haemorrhoids,
            constipated
        ],
    )?;

    Ok(conn.query_row(
        "SELECT * FROM shit_session WHERE user_id = ? AND timestamp = ?",
        params![user.id.0, timestamp],
        |row| {
            Ok(ShitSession {
                id: row.get(0)?,
                user_id: row.get(1)?,
                timestamp,
                duration: row.get(3)?,
                location: row.get(4)?,
                haemorrhoids,
                constipated,
            })
        },
    )?)
}

pub async fn insert_shitting_session_with_location(
    conn: Arc<Mutex<Connection>>,
    user: &User,
    duration: u64,
    location: &str,
    haemorrhoids: bool,
    constipated: bool,
) -> Result<ShitSession> {
    let conn = conn.lock().await;
    let timestamp = UNIX_EPOCH.elapsed()?.as_secs();
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
        params![
            user.id.0,
            timestamp,
            duration,
            location,
            haemorrhoids,
            constipated
        ],
    )?;

    Ok(conn.query_row(
        "SELECT * FROM shit_session WHERE user_id = ? AND timestamp = ?",
        params![user.id.0, timestamp],
        |row| {
            Ok(ShitSession {
                id: row.get(0)?,
                user_id: row.get(1)?,
                timestamp,
                duration: row.get(3)?,
                location: row.get(4)?,
                haemorrhoids,
                constipated,
            })
        },
    )?)
}

pub async fn query_shit_session_from(
    conn: Arc<Mutex<Connection>>,
    user: &User,
    timestamp: u64,
) -> Result<Vec<ShitSession>> {
    let conn = conn.lock().await;
    let mut statement =
        conn.prepare("SELECT * FROM shit_session WHERE timestamp >= ? AND user_id = ?")?;
    let mut res = Vec::new();
    let mut state_iter = statement.query(params![timestamp, user.id.0])?;
    while let Ok(Some(r)) = state_iter.next() {
        res.push(ShitSession {
            id: r.get(0)?,
            user_id: r.get(1)?,
            timestamp: r.get(2)?,
            duration: r.get(3)?,
            location: r.get(4)?,
            haemorrhoids: r.get(5)?,
            constipated: r.get(6)?,
        });
    }
    return Ok(res);
}

pub async fn delete_shit_session(conn: Arc<Mutex<Connection>>, id: u64) -> Result<()> {
    let conn = conn.lock().await;
    conn.execute("DELETE FROM shit_session WHERE id = ?", params![id])?;
    return Ok(());
}
