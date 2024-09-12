mod bot;
mod database;

use bot::answer;
use log::info;
use teloxide::{repl, Bot};

const BOT_NAME: &str = "Bot name";
const DB_NAME: &str = "toilet_db";

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting the bot...");
    let bot = Bot::from_env();
    repl(bot, answer).await;
}
