use crate::{database::create_or_update_user, BOT_NAME};
use log::error;
use rusqlite::Connection;
use std::sync::Arc;
use teloxide::{
    macros::BotCommands,
    prelude::ResponseResult,
    types::{Message, User},
    utils::command::{parse_command, BotCommands as _},
    Bot,
};
use tokio::sync::Mutex;

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    Cagando,
}

pub async fn answer(conn: Arc<Mutex<Connection>>, bot: Bot, msg: Message) -> ResponseResult<()> {
    if msg.text().is_none() {
        return Ok(());
    }

    let text = msg.text().unwrap();
    let command = Command::parse(text, BOT_NAME);
    if command.is_err() || msg.from.is_none() {
        return Ok(());
    }

    if create_or_update_user(conn.clone(), msg.from.as_ref().unwrap())
        .await
        .is_err()
    {
        error!("Failed to query number of users");
        return Ok(());
    }

    match command.unwrap() {
        Command::Cagando => answer_cagando(bot, text, msg.from.as_ref().unwrap()).await,
    }
}

async fn answer_cagando(bot: Bot, text: &str, user: &User) -> ResponseResult<()> {
    let (_, args) = parse_command(text, BOT_NAME).unwrap();
    match args.len() {
        _ => (), // TODO
    }
    return Ok(());
}
