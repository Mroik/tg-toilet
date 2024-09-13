mod bot;
mod database;

use anyhow::Result;
use bot::answer;
use database::setup_db;
use log::info;
use std::sync::Arc;
use teloxide::{dispatching::UpdateFilterExt, dptree, prelude::Dispatcher, types::Update, Bot};
use tokio::sync::Mutex;

const BOT_NAME: &str = "Bot name";
const DB_NAME: &str = "toilet_db";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting the bot...");
    let bot = Bot::from_env();
    let conn = Arc::new(Mutex::new(setup_db().await?));
    Dispatcher::builder(bot, Update::filter_message().endpoint(answer))
        .dependencies(dptree::deps![conn])
        .build()
        .dispatch()
        .await;
    return Ok(());
}
