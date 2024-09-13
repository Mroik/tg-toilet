use crate::DB_NAME;
use anyhow::Result;
use rusqlite::Connection;
use std::{path::Path, sync::Arc};
use teloxide::types::User;
use tokio::sync::Mutex;

pub async fn setup_db() -> Result<Connection> {
    let exists = Path::new(DB_NAME).exists();
    let conn = Connection::open(DB_NAME)?;
    if !exists {
        conn.execute(
            "CREATE TABLE user (
                id BIGINT PRIMARY KEY,
                username TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE shit_session (
                id INT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                timestamp BIGINT NOT NULL,
                duration INT,
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

pub async fn create_or_update_user(
    conn: Arc<Mutex<Connection>>,
    user: &User,
) -> anyhow::Result<()> {
    let conn = conn.lock().await;
    let mut statement = conn.prepare(
        "INSERT INTO user(id, username) VALUES (?, ?) ON CONFLICT(id) DO UPDATE SET username = ?",
    )?;
    let username = if user.username.is_some() {
        user.username.as_ref().unwrap().clone()
    } else {
        user.full_name()
    };
    statement.execute(rusqlite::params![user.id.0, username, username])?;
    return Ok(());
}
