use crate::{
    database::{
        create_or_update_user, insert_shitting_session, insert_shitting_session_with_duration,
        insert_shitting_session_with_location,
    },
    BOT_NAME,
};
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
    prelude::{Requester, ResponseResult},
    types::{Message, ParseMode, ReplyParameters},
    utils::command::{parse_command, BotCommands as _},
    Bot,
};
use tokio::sync::Mutex;

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    Shitting,
}

const SHITTING_USAGE: &str = "⚠️⚠️⚠️\nUsage:\n/shitting\n/shitting duration\n/shitting duration location\n/shitting duration location haemorrhoids\n/shitting duration location haemorrhoids constipated\nDuration: in seconds\nLocation: A string without inner whitespaces\nHaemorrhoids and Constipated are either `true` or `false`";
const PLEASE_REPORT: &str =
    "Coudln't insert your shitting session 😥\nPlease report incident to @Mroik";

pub async fn answer(conn: Arc<Mutex<Connection>>, bot: Bot, msg: Message) -> ResponseResult<()> {
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
    }
}

async fn answer_shitting(
    conn: Arc<Mutex<Connection>>,
    bot: Bot,
    msg: Message,
) -> ResponseResult<()> {
    let (_, args) = parse_command(msg.text().unwrap(), BOT_NAME).unwrap();
    let user = msg.from.as_ref().unwrap();

    let timestamp = match args.len() {
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

    if timestamp.is_none() {
        bot.send_message(
            msg.chat.id,
            //PLEASE_REPORT,
            SHITTING_USAGE,
        )
        .parse_mode(ParseMode::MarkdownV2)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
    } else {
        let cur = Duration::new(timestamp.unwrap(), 0);
        let date: DateTime<Local> = (UNIX_EPOCH + cur).into();
        let username = if msg.from.as_ref().unwrap().username.is_some() {
            format!("@{}", msg.from.unwrap().username.unwrap())
        } else {
            format!(
                "[{}](tg://user?id={})",
                msg.from.as_ref().unwrap().full_name(),
                msg.from.unwrap().id.0,
            )
        };
        bot.send_message(
            msg.chat.id,
            format!(
                "💩💩💩\n{} added a new shitting session to the database with timestamp {}",
                username,
                date.format("%Y\\-%m\\-%d %H:%M")
            ),
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
        bot.delete_message(msg.chat.id, msg.id).await?;
    }
    return Ok(());
}
