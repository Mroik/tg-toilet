use crate::{
    database::{
        create_or_update_user, insert_shitting_session, insert_shitting_session_with_duration,
        insert_shitting_session_with_location,
    },
    BOT_NAME,
};
use log::error;
use rusqlite::Connection;
use std::sync::Arc;
use teloxide::{
    macros::BotCommands,
    payloads::SendMessageSetters,
    prelude::{Requester, ResponseResult},
    types::{Message, ReplyParameters},
    utils::command::{parse_command, BotCommands as _},
    Bot,
};
use tokio::sync::Mutex;

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    Shitting,
}

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

    let failed = match args.len() {
        0 => insert_shitting_session(conn, user, false, false)
            .await
            .is_err(),
        1 => insert_shitting_session_with_duration(
            conn,
            user,
            args.first().unwrap().parse().unwrap(),
            false,
            false,
        )
        .await
        .is_err(),
        2 => insert_shitting_session_with_location(
            conn,
            user,
            args.first().unwrap().parse().unwrap(),
            args.get(1).unwrap(),
            false,
            false,
        )
        .await
        .is_err(),
        3 => insert_shitting_session_with_location(
            conn,
            user,
            args.first().unwrap().parse().unwrap(),
            args.get(1).unwrap(),
            args.get(2).unwrap().parse().unwrap(),
            false,
        )
        .await
        .is_err(),
        4 => insert_shitting_session_with_location(
            conn,
            user,
            args.first().unwrap().parse().unwrap(),
            args.get(1).unwrap(),
            args.get(2).unwrap().parse().unwrap(),
            args.get(3).unwrap().parse().unwrap(),
        )
        .await
        .is_err(),
        _ => {
            bot.send_message(msg.chat.id, "⚠️⚠️⚠️\nUsage:\n/shitting\n/shitting duration\n/shitting duration location\n/shitting duration location haemorrhoids\n/shitting duration location haemorrhoids constipated\nDuration: in seconds\nLocation: A string without inner whitespaces\nHaemorrhoids and Constipated are either `true` or `false`").reply_parameters(ReplyParameters::new(msg.id)).await?;
            return Ok(());
        }
    };

    if failed {
        bot.send_message(
            msg.chat.id,
            "Coudln't insert your shitting session 😥\nPlease report incident to @Mroik",
        )
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
    } else {
        bot.delete_message(msg.chat.id, msg.id).await?;
    }
    return Ok(());
}
