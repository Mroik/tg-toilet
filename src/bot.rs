use crate::BOT_NAME;
use teloxide::{
    macros::BotCommands,
    prelude::{Requester, ResponseResult},
    types::Message,
    utils::command::BotCommands as _,
    Bot,
};

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    Cagando,
    Info,
    Statistiche,
}

pub async fn answer(bot: Bot, msg: Message) -> ResponseResult<()> {
    if msg.text().is_none() {
        return Ok(());
    }

    let text = msg.text().unwrap();
    match Command::parse(text, BOT_NAME) {
        Ok(Command::Cagando) => todo!(),
        Ok(Command::Info) => todo!(),
        Ok(Command::Statistiche) => todo!(),
        Err(_) => {
            bot.send_message(msg.chat.id, "").await?;
            return Ok(());
        }
    }
}
