use crate::{
    database::{
        create_or_update_user, delete_shit_session, insert_shitting_session,
        insert_shitting_session_with_duration, insert_shitting_session_with_location,
        query_sessions_skipping, query_shit_session_from,
    },
    BOT_NAME,
};
use anyhow::Result;
use chrono::{DateTime, Local};
use log::error;
use rusqlite::Connection;
use std::{
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};
use teloxide::{
    macros::BotCommands,
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{
        CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode,
        ReplyParameters, User,
    },
    utils::command::{parse_command, BotCommands as _},
    Bot,
};
use tokio::sync::Mutex;

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    Shitting,
    Week,
    Month,
    Year,
    Sessions,
}

pub struct ShitSession {
    pub id: u64,
    pub user_id: u64,
    pub timestamp: u64,
    pub duration: Option<u64>,
    pub location: Option<String>,
    pub haemorrhoids: bool,
    pub constipated: bool,
}

const SHITTING_USAGE: &str = "⚠️⚠️⚠️\nUsage:\n/shitting\n/shitting duration\n/shitting duration location\n/shitting duration location haemorrhoids\n/shitting duration location haemorrhoids constipated\nDuration: in seconds\nLocation: A string without inner whitespaces\nHaemorrhoids and Constipated are either `true` or `false`";
const PLEASE_REPORT: &str =
    "Coudln't insert your shitting session 😥\nPlease report incident to @Mroik";
const DAY: u64 = 60 * 60 * 24;

pub async fn answer(conn: Arc<Mutex<Connection>>, bot: Bot, msg: Message) -> Result<()> {
    if msg.text().is_none() {
        return Ok(());
    }

    let text = msg.text().unwrap();
    let command = Command::parse(text, BOT_NAME);
    if command.is_err() || msg.from.is_none() || !msg.chat.is_chat() {
        return Ok(());
    }

    if create_or_update_user(conn.clone(), msg.from.as_ref().unwrap())
        .await
        .is_err()
    {
        error!("Failed to query number of users");
        bot.send_message(
            msg.chat.id,
            "There was an error with the database 😥\n Please report this incident to @Mroik",
        )
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
        return Ok(());
    }

    match command.unwrap() {
        Command::Shitting => answer_shitting(conn, bot, msg).await,
        Command::Week => {
            answer_average_with_window(
                conn,
                bot,
                msg,
                DAY * 7,
                "Last week {} shat on average {} times a day",
            )
            .await
        }
        Command::Month => {
            answer_average_with_window(
                conn,
                bot,
                msg,
                DAY * 30,
                "Last month {} shat on average {} times a day",
            )
            .await
        }
        Command::Year => {
            answer_average_with_window(
                conn,
                bot,
                msg,
                DAY * 365,
                "Last year {} shat on average {} times a day",
            )
            .await
        }
        Command::Sessions => answer_sessions(conn, bot, msg).await,
    }
}

async fn timestamp2datetime_string(timestamp: u64) -> String {
    let date = DateTime::from_timestamp(timestamp as i64, 0).unwrap();
    let d = date.with_timezone(&Local);
    d.format("%Y-%m-%d %H:%M").to_string()
}

async fn duration2string(timestamp: u64) -> String {
    let mut temp = timestamp;
    let hours = temp / 3600;
    temp %= 3600;
    let minutes = temp / 60;
    let secs = temp % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

async fn generate_table(data: &[ShitSession]) -> String {
    let mut ris = String::new();

    // Check lengths of each column
    let mut max = [2, 9, 8, 8, 12, 11];
    for session in data {
        let mut cur = session.id.to_string().len();
        if cur > max[0] {
            max[0] = cur;
        }
        cur = timestamp2datetime_string(session.timestamp).await.len();
        if cur > max[1] {
            max[1] = cur;
        }
        cur = if session.duration.is_some() {
            duration2string(session.duration.unwrap()).await.len()
        } else {
            0
        };
        if cur > max[2] {
            max[2] = cur;
        }
        cur = if session.location.is_some() {
            session.location.as_ref().unwrap().len()
        } else {
            0
        };
        if cur > max[3] {
            max[3] = cur;
        }
        cur = if session.haemorrhoids { 3 } else { 2 };
        if cur > max[4] {
            max[4] = cur;
        }
        cur = if session.constipated { 3 } else { 2 };
        if cur > max[5] {
            max[5] = cur;
        }
    }

    // Header
    let mut table_line: String = (0..7 + 12 + max.iter().sum::<usize>())
        .map(|_| '-')
        .collect();
    table_line.push('\n');
    ris.push_str(&table_line);

    ris.push_str("| Id");
    for _ in 0..max[0] - 2 {
        ris.push(' ');
    }

    ris.push_str(" | Timestamp");
    for _ in 0..max[1] - 9 {
        ris.push(' ');
    }

    ris.push_str(" | Duration");
    for _ in 0..max[2] - 8 {
        ris.push(' ');
    }

    ris.push_str(" | Location");
    for _ in 0..max[3] - 8 {
        ris.push(' ');
    }

    ris.push_str(" | Haemorrhoids");
    for _ in 0..max[4] - 12 {
        ris.push(' ');
    }

    ris.push_str(" | Constipated");
    for _ in 0..max[5] - 11 {
        ris.push(' ');
    }
    ris.push_str(" |\n");
    ris.push_str(&table_line);

    // Table body
    for session in data {
        ris.push_str("| ");
        ris.push_str(&session.id.to_string());
        for _ in 0..max[0] - session.id.to_string().len() {
            ris.push(' ');
        }

        ris.push_str(" | ");
        let stamp = timestamp2datetime_string(session.timestamp).await;
        ris.push_str(&stamp);
        for _ in 0..max[1] - stamp.len() {
            ris.push(' ');
        }

        ris.push_str(" | ");
        let dur = if session.duration.is_some() {
            duration2string(session.duration.unwrap()).await
        } else {
            String::new()
        };
        ris.push_str(&dur);
        for _ in 0..max[2] - dur.len() {
            ris.push(' ');
        }

        ris.push_str(" | ");
        let mut temp_string = if session.location.is_some() {
            session.location.as_ref().unwrap()
        } else {
            ""
        };
        ris.push_str(temp_string);
        for _ in 0..max[3] - temp_string.len() {
            ris.push(' ');
        }

        ris.push_str(" | ");
        temp_string = if session.haemorrhoids { "Yes" } else { "No" };
        ris.push_str(temp_string);
        for _ in 0..max[4] - temp_string.len() {
            ris.push(' ');
        }

        ris.push_str(" | ");
        temp_string = if session.constipated { "Yes" } else { "No" };
        ris.push_str(temp_string);
        for _ in 0..max[5] - temp_string.len() {
            ris.push(' ');
        }

        ris.push_str(" |");
        ris.push('\n');
        ris.push_str(&table_line);
    }
    return ris;
}

async fn answer_sessions(conn: Arc<Mutex<Connection>>, bot: Bot, msg: Message) -> Result<()> {
    let (_, args) = parse_command(msg.text().unwrap(), BOT_NAME).unwrap();
    let user = msg.from.as_ref().unwrap();
    let skip_point = match args.len() {
        0 => i64::MAX as u64,
        1 => args.first().unwrap().parse()?,
        _ => {
            bot.send_message(
                msg.chat.id,
                "⚠️⚠️⚠️\nUsage:\n/sessions\n/sessions <number of entries to skip>",
            )
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;
            return Ok(());
        }
    };

    let mut data = query_sessions_skipping(conn, user, skip_point).await?;
    data.reverse();
    let table = generate_table(&data).await;
    bot.send_message(msg.chat.id, format!("```txt\n{}\n```", table))
        .reply_parameters(ReplyParameters::new(msg.id))
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
    return Ok(());
}

async fn format_label(label: &str, args: &[String]) -> String {
    let parts: Vec<&str> = label.split("{}").collect();
    let mut ris = String::new();
    for i in 0..args.len() {
        ris.push_str(parts.get(i).unwrap());
        ris.push_str(args.get(i).unwrap());
    }
    ris.push_str(parts.last().unwrap());
    return ris;
}

async fn double_decimal_format(n: f32) -> String {
    let ris = n.to_string();
    ris.chars()
        .fold((String::new(), -1), |(mut s, mut many), c| {
            if many == 0 {
                (s, many)
            } else {
                many -= 1;
                if c == '.' {
                    many = 3;
                    s.push('\\');
                }
                s.push(c);
                (s, many)
            }
        })
        .0
}

async fn username_or_full(user: &User) -> String {
    if user.username.is_some() {
        format!("@{}", user.username.as_ref().unwrap())
    } else {
        format!("[{}](tg://user?id={})", user.full_name(), user.id.0,)
    }
}

async fn answer_average_with_window(
    conn: Arc<Mutex<Connection>>,
    bot: Bot,
    msg: Message,
    window: u64,
    label: &str,
) -> Result<()> {
    let current = UNIX_EPOCH.elapsed().unwrap().as_secs();
    let starting = current - window;
    match query_shit_session_from(conn, msg.from.as_ref().unwrap(), starting).await {
        Ok(r) => {
            let n = double_decimal_format(r.len() as f32 / (window / DAY) as f32).await;
            let label = format_label(label, &[username_or_full(&msg.from.unwrap()).await, n]).await;
            bot.send_message(msg.chat.id, label)
                .reply_parameters(ReplyParameters::new(msg.id))
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(());
        }
        Err(_) => {
            error!("Couldn't query the shit sessions");
            bot.send_message(msg.chat.id, "Couldn't query the shit sessions")
                .reply_parameters(ReplyParameters::new(msg.id))
                .await?;
            return Ok(());
        }
    }
}

async fn answer_shitting(conn: Arc<Mutex<Connection>>, bot: Bot, msg: Message) -> Result<()> {
    let (_, args) = parse_command(msg.text().unwrap(), BOT_NAME).unwrap();
    let user = msg.from.as_ref().unwrap();

    let new_record = match args.len() {
        0 => Some(
            insert_shitting_session(conn, user, false, false)
                .await
                .unwrap(),
        ),
        1 => {
            let duration = args.first().unwrap().parse();
            match duration {
                Err(_) => None,
                Ok(d) => Some(
                    insert_shitting_session_with_duration(conn, user, d, false, false)
                        .await
                        .unwrap(),
                ),
            }
        }
        2 => {
            let duration = args.first().unwrap().parse();
            match duration {
                Err(_) => None,
                Ok(d) => Some(
                    insert_shitting_session_with_location(
                        conn,
                        user,
                        d,
                        args.get(1).unwrap(),
                        false,
                        false,
                    )
                    .await
                    .unwrap(),
                ),
            }
        }
        3 => {
            let duration = args.first().unwrap().parse();
            let haemorrhoids = args.get(2).unwrap().parse();
            match (duration, haemorrhoids) {
                (Err(_), _) | (_, Err(_)) => None,
                (Ok(d), Ok(h)) => Some(
                    insert_shitting_session_with_location(
                        conn,
                        user,
                        d,
                        args.get(1).unwrap(),
                        h,
                        false,
                    )
                    .await
                    .unwrap(),
                ),
            }
        }
        4 => {
            let duration = args.first().unwrap().parse();
            let haemorrhoids = args.get(2).unwrap().parse();
            let constipated = args.get(3).unwrap().parse();
            match (duration, haemorrhoids, constipated) {
                (Err(_), _, _) | (_, Err(_), _) | (_, _, Err(_)) => None,
                (Ok(d), Ok(h), Ok(c)) => Some(
                    insert_shitting_session_with_location(
                        conn,
                        user,
                        d,
                        args.get(1).unwrap(),
                        h,
                        c,
                    )
                    .await
                    .unwrap(),
                ),
            }
        }
        _ => {
            bot.send_message(msg.chat.id, SHITTING_USAGE)
                .reply_parameters(ReplyParameters::new(msg.id))
                .await?;
            return Ok(());
        }
    };

    if new_record.is_none() {
        bot.send_message(
            msg.chat.id,
            //PLEASE_REPORT,
            SHITTING_USAGE,
        )
        .parse_mode(ParseMode::MarkdownV2)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
        return Ok(());
    }

    let new_record = new_record.unwrap();
    let cur = Duration::new(new_record.timestamp, 0);
    let date: DateTime<Local> = (UNIX_EPOCH + cur).into();
    let username = username_or_full(msg.from.as_ref().unwrap()).await;
    bot.send_message(
        msg.chat.id,
        format!(
            "💩💩💩\n{} added a new shitting session to the database with timestamp `{}`",
            username,
            date.format("%Y\\-%m\\-%d %H:%M")
        ),
    )
    .reply_parameters(ReplyParameters::new(msg.id))
    .reply_markup(InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("❌ Delete record", format!("{}", new_record.id)),
    ]]))
    .parse_mode(ParseMode::MarkdownV2)
    .await?;
    return Ok(());
}

pub async fn delete_shit_callback(
    conn: Arc<Mutex<Connection>>,
    bot: Bot,
    query: CallbackQuery,
) -> Result<()> {
    bot.answer_callback_query(&query.id).await?;
    if query.data.is_none() || !query.mentioned_users().any(|user| user.id == query.from.id) {
        return Ok(());
    }

    let message = query.message.as_ref().unwrap();
    if delete_shit_session(conn, query.data.unwrap().parse().unwrap())
        .await
        .is_err()
    {
        bot.send_message(message.chat().id, "Couldn't delete record")
            .reply_parameters(ReplyParameters::new(message.id()))
            .await?;
        return Ok(());
    }

    bot.send_message(message.chat().id, "Deleted record")
        .reply_parameters(ReplyParameters::new(message.id()))
        .await?;
    bot.edit_message_reply_markup(message.chat().id, message.id())
        .await?;
    return Ok(());
}
