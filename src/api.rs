use crate::{
    bot::ShitSession,
    database::{query_sessions_of_user, query_user},
};
use chrono::{DateTime, Local};
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

pub async fn start_api(conn: Arc<Mutex<Connection>>) {
    let routes = warp::get()
        .and(warp::path("sessions"))
        .and(warp::path::param())
        .and(warp::path::param())
        .and(warp::any().map(move || conn.clone()))
        .and_then(
            |_: u64, user: u64, conn: Arc<Mutex<Connection>>| async move {
                match query_sessions_of_user(conn.clone(), user).await {
                    Ok(mut data) => {
                        data.reverse();
                        let username = if let Ok(u) = query_user(conn, user).await {
                            u.username
                        } else {
                            return Err(warp::reject::not_found());
                        };
                        let page = generate_page(&data, &username).await;
                        Ok(warp::reply::html(page))
                    }
                    Err(_) => Err(warp::reject::not_found()),
                }
            },
        );

    warp::serve(routes).run(([0, 0, 0, 0], 6969)).await;
}

async fn generate_page(data: &[ShitSession], username: &str) -> String {
    let mut ris = String::new();
    ris.push_str(&format!(
        "
        <html>
            <head>
                <title>Shitting data</title>
            </head>
            <body>
                <article>
                    <h1>{}</h1>
    ",
        username
    ));
    ris.push_str(&generate_table(data).await);
    ris.push_str("</article></body></html>");
    return ris;
}

async fn generate_table(data: &[ShitSession]) -> String {
    let mut ris = String::new();
    let body = data
        .iter()
        .map(|session| {
            let mut r = String::new();
            r.push_str("<tr>");
            r.push_str("<td>");
            r.push_str(&session.id.to_string());
            r.push_str("</td>");
            r.push_str("<td>");
            r.push_str(&timestamp2datetime_string(session.timestamp));
            r.push_str("</td>");
            r.push_str("<td>");
            if session.duration.is_some() {
                r.push_str(&duration2string(session.duration.unwrap()));
            }
            r.push_str("</td>");
            r.push_str("<td>");
            if session.location.is_some() {
                r.push_str(session.location.as_ref().unwrap());
            }
            r.push_str("</td>");
            r.push_str("<td>");
            r.push_str(&session.haemorrhoids.to_string());
            r.push_str("</td>");
            r.push_str("<td>");
            r.push_str(&session.constipated.to_string());
            r.push_str("</td>");
            r.push_str("</tr>");
            r
        })
        .collect::<Vec<String>>()
        .concat();

    ris.push_str(
        "<table>
        <tr>
            <th>Id</th>
            <th>Timestamp</th>
            <th>Duration</th>
            <th>Location</th>
            <th>Haemorrhoids</th>
            <th>Constipated</th>
        </tr>",
    );
    ris.push_str(&body);
    ris.push_str("</table>");
    return ris;
}

fn timestamp2datetime_string(timestamp: u64) -> String {
    let date = DateTime::from_timestamp(timestamp as i64, 0).unwrap();
    let d = date.with_timezone(&Local);
    d.format("%Y-%m-%d %H:%M").to_string()
}

fn duration2string(timestamp: u64) -> String {
    let mut temp = timestamp;
    let hours = temp / 3600;
    temp %= 3600;
    let minutes = temp / 60;
    let secs = temp % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}
