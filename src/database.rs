use crate::{
    bot::{ShitSession, ShitUser},
    DB_NAME,
};
use anyhow::Result;
use rusqlite::{params, types::Null, Connection};
use std::{sync::Arc, time::UNIX_EPOCH};
use teloxide::types::User;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ToiletDB {
    conn: Arc<Mutex<Connection>>,
}

impl ToiletDB {
    pub async fn new() -> Result<ToiletDB> {
        let conn = Connection::open(DB_NAME)?;
        let mut exists = conn
            .prepare("SELECT * FROM sqlite_master WHERE type = 'table' AND name = 'user'")?
            .exists([])?;
        if !exists {
            conn.execute(
                "CREATE TABLE user (
                    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                    username TEXT NOT NULL
                )",
                [],
            )?;
        }
        exists = conn
            .prepare("SELECT * FROM sqlite_master WHERE type = 'table' AND name = 'shit_session'")?
            .exists([])?;
        if !exists {
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
        return Ok(ToiletDB {
            conn: Arc::new(Mutex::new(conn)),
        });
    }

    pub async fn create_or_update_user(&self, user: &User) -> Result<()> {
        let conn = self.conn.lock().await;
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
        &self,
        user: &User,
        haemorrhoids: bool,
        constipated: bool,
    ) -> Result<ShitSession> {
        let conn = self.conn.lock().await;
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
        &self,
        user: &User,
        duration: u64,
        haemorrhoids: bool,
        constipated: bool,
    ) -> Result<ShitSession> {
        let conn = self.conn.lock().await;
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
                timestamp - duration,
                duration,
                Null,
                haemorrhoids,
                constipated
            ],
        )?;

        let ris = conn.query_row(
            "SELECT * FROM shit_session WHERE user_id = ? AND timestamp = ?",
            params![user.id.0, timestamp - duration],
            |row| {
                Ok(ShitSession {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    timestamp: row.get(2)?,
                    duration: row.get(3)?,
                    location: row.get(4)?,
                    haemorrhoids,
                    constipated,
                })
            },
        )?;
        return Ok(ris);
    }

    pub async fn insert_shitting_session_with_location(
        &self,
        user: &User,
        duration: u64,
        location: &str,
        haemorrhoids: bool,
        constipated: bool,
    ) -> Result<ShitSession> {
        let conn = self.conn.lock().await;
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
                timestamp - duration,
                duration,
                location,
                haemorrhoids,
                constipated
            ],
        )?;

        Ok(conn.query_row(
            "SELECT * FROM shit_session WHERE user_id = ? AND timestamp = ?",
            params![user.id.0, timestamp - duration],
            |row| {
                Ok(ShitSession {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    timestamp: row.get(2)?,
                    duration: row.get(3)?,
                    location: row.get(4)?,
                    haemorrhoids,
                    constipated,
                })
            },
        )?)
    }

    pub async fn query_shit_session_from(
        &self,
        user: &User,
        timestamp: u64,
    ) -> Result<Vec<ShitSession>> {
        let conn = self.conn.lock().await;
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
    pub async fn delete_shit_session(&self, id: u64) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM shit_session WHERE id = ?", params![id])?;
        return Ok(());
    }

    pub async fn query_sessions_of_user(&self, user: u64) -> Result<Vec<ShitSession>> {
        let mut ris = Vec::new();
        let conn = self.conn.lock().await;
        let mut statement = conn.prepare("SELECT * FROM shit_session WHERE user_id = ?")?;
        let mut state_iter = statement.query(params![user])?;
        while let Ok(Some(row)) = state_iter.next() {
            ris.push(ShitSession {
                id: row.get(0)?,
                user_id: row.get(1)?,
                timestamp: row.get(2)?,
                duration: row.get(3)?,
                location: row.get(4)?,
                haemorrhoids: row.get(5)?,
                constipated: row.get(6)?,
            });
        }
        return Ok(ris);
    }

    pub async fn query_user(&self, user_id: u64) -> Result<ShitUser> {
        let conn = self.conn.lock().await;
        let ris = conn.query_row("SELECT * FROM user WHERE id = ?", params![user_id], |row| {
            Ok(ShitUser {
                id: row.get(0)?,
                username: row.get(1)?,
            })
        })?;
        return Ok(ris);
    }

    pub async fn query_username(&self, username: &str) -> Result<ShitUser> {
        let conn = self.conn.lock().await;
        let ris = conn.query_row(
            "SELECT * FROM user WHERE username = ?",
            params![username],
            |row| {
                Ok(ShitUser {
                    id: row.get(0)?,
                    username: row.get(1)?,
                })
            },
        )?;
        return Ok(ris);
    }
}
