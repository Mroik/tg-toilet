use crate::BOT_NAME;
use teloxide::{
    macros::BotCommands,
    prelude::{Requester, ResponseResult},
    types::Message,
    utils::command::{parse_command, BotCommands as _},
    Bot,
};

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    Cagando,
}

pub async fn answer(bot: Bot, msg: Message) -> ResponseResult<()> {
    if msg.text().is_none() {
        return Ok(());
    }

    let text = msg.text().unwrap();
    let command = Command::parse(text, BOT_NAME);
    if command.is_err() {
        bot.send_message(msg.chat.id, "").await?;
        return Ok(());
    }

    if msg.from.is_none() {
        return Ok(());
    }
    // TODO Figure out how to pass the database Connection
    //create_or_update_user(msg.from.unwrap()).await;

    match command.unwrap() {
        Command::Cagando => todo!(),
    }
}

async fn answer_cagando(text: &str) -> ResponseResult<()> {
    let (_, args) = parse_command(text, BOT_NAME).unwrap();
    match args.len() {
        _ => (), // TODO
    }
    return Ok(());
}
