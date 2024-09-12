mod database;

use log::info;
use teloxide::{
    macros::BotCommands,
    prelude::{Requester, ResponseResult},
    repl,
    types::Message,
    utils::command::BotCommands as _,
    Bot,
};

const BOT_NAME: &str = "Bot name";
const DB_NAME: &str = "toilet_db";

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    Cagando,
    Info,
    Statistiche,
}

async fn answer(bot: Bot, msg: Message) -> ResponseResult<()> {
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

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting the bot...");
    let bot = Bot::from_env();
    repl(bot, answer).await;
}
