mod api;
mod bot;
mod database;

use anyhow::Result;
use api::start_api;
use bot::{answer, delete_shit_callback};
use database::setup_db;
use log::info;
use std::sync::Arc;
use teloxide::{dispatching::UpdateFilterExt, dptree, prelude::Dispatcher, types::Update, Bot};
use tokio::sync::Mutex;

const DB_NAME: &str = "toilet_db";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let bot = Bot::from_env();
    let conn = Arc::new(Mutex::new(setup_db().await?));
    let mut disp = Dispatcher::builder(
        bot,
        dptree::entry()
            .branch(Update::filter_callback_query().endpoint(delete_shit_callback))
            .branch(Update::filter_message().endpoint(answer)),
    )
    .dependencies(dptree::deps![conn.clone()])
    .build();

    info!("Starting the bot...");
    tokio::join!(start_api(conn.clone()), disp.dispatch());
    return Ok(());
}
