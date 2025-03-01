use crate::database::ToiletDB;
use anyhow::Result;
use chrono::DateTime;
use chrono_tz::Tz;
use lazy_static::lazy_static;
use log::error;
use rand::random;
use std::time::UNIX_EPOCH;
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

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    Shitting,
    Week,
    Month,
    Year,
    Sessions,
    Help,
}

impl TryFrom<&str> for Command {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let ris = match value {
            "shitting" => Self::Shitting,
            "week" => Self::Week,
            "month" => Self::Month,
            "year" => Self::Year,
            "sessions" => Self::Sessions,
            _ => {
                return Err(anyhow::Error::msg(format!(
                    "Couldn't convert {} into a Command",
                    value
                )))
            }
        };
        return Ok(ris);
    }
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

pub struct ShitUser {
    pub id: u64,
    pub username: String,
}

const SHITTING_USAGE: &str = "⚠️⚠️⚠️\nUsage:\n/shitting\n/shitting duration\n/shitting duration location\n/shitting duration location haemorrhoids\n/shitting duration location haemorrhoids constipated\nDuration: in seconds\nLocation: A string without inner whitespaces\nHaemorrhoids and Constipated are either `true` or `false`";
const SESSIONS_USAGE: &str = "⚠️⚠️⚠️\nUsage:\n/sessions\n/sessions @user";
const HELP_USAGE: &str = "/help shitting\n/help sessions\n/help week\n/help month\n/help year";
const PLEASE_REPORT: &str =
    "Coudln't insert your shitting session 😥\nPlease report incident to @Mroik";
const NEVER_USED_BOT: &str = "This user has never used this bot";
const DAY: u64 = 60 * 60 * 24;

lazy_static! {
    static ref DOMAIN_NAME: String = std::env::var("DOMAIN_NAME").unwrap().replace(".", "\\.");
    static ref VIEW_RHASH: String = std::env::var("VIEW_RHASH").unwrap();
    static ref BOT_NAME: String = std::env::var("BOT_NAME").unwrap();
    pub static ref TIMEZONE: Tz = std::env::var("TIMEZONE").unwrap().parse().unwrap();
}

pub async fn answer(conn: ToiletDB, bot: Bot, msg: Message) -> Result<()> {
    if msg.text().is_none() {
        return Ok(());
    }

    let text = msg.text().unwrap();
    let command = Command::parse(text, &BOT_NAME);
    if command.is_err() || msg.from.is_none() || !msg.chat.is_chat() {
        return Ok(());
    }

    if conn
        .create_or_update_user(msg.from.as_ref().unwrap())
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
        Command::Help => answer_help(bot, msg).await,
    }
}

async fn answer_help(bot: Bot, msg: Message) -> Result<()> {
    let (_, args) = parse_command(msg.text().unwrap(), &(*BOT_NAME)).unwrap();
    match args.len() {
        0 => {
            bot.send_message(msg.chat.id, HELP_USAGE)
                .reply_parameters(ReplyParameters::new(msg.id))
                .await?;
        }
        _ => match Command::try_from(*args.first().unwrap()) {
            Ok(Command::Shitting) => {
                bot.send_message(msg.chat.id, SHITTING_USAGE)
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;
            }
            Ok(Command::Week) => {
                bot.send_message(msg.chat.id, "⚠️⚠️⚠️\nUsage:\n/week")
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;
            }
            Ok(Command::Month) => {
                bot.send_message(msg.chat.id, "⚠️⚠️⚠️\nUsage:\n/month")
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;
            }
            Ok(Command::Year) => {
                bot.send_message(msg.chat.id, "⚠️⚠️⚠️\nUsage:\n/year")
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;
            }
            Ok(Command::Sessions) => {
                bot.send_message(msg.chat.id, SESSIONS_USAGE)
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;
            }
            _ => {
                bot.send_message(msg.chat.id, HELP_USAGE)
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;
            }
        },
    }
    return Ok(());
}

async fn answer_sessions(conn: ToiletDB, bot: Bot, msg: Message) -> Result<()> {
    let user = {
        let parsed = parse_command(msg.text().unwrap(), &(*BOT_NAME)).unwrap().1;
        match parsed.len() {
            0 => msg.from.as_ref().unwrap().id.0,
            1 => {
                let user = parsed.first().unwrap();
                if user.starts_with('@') {
                    if let Ok(user) = conn
                        .query_username(user.chars().skip(1).collect::<String>().as_ref())
                        .await
                    {
                        user.id
                    } else {
                        bot.send_message(msg.chat.id, NEVER_USED_BOT)
                            .reply_parameters(ReplyParameters::new(msg.id))
                            .await?;
                        return Ok(());
                    }
                } else {
                    bot.send_message(msg.chat.id, SESSIONS_USAGE)
                        .reply_parameters(ReplyParameters::new(msg.id))
                        .await?;
                    return Ok(());
                }
            }
            _ => {
                bot.send_message(msg.chat.id, SESSIONS_USAGE)
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;
                return Ok(());
            }
        }
    };
    let rand_number: u64 = random();
    bot.send_message(
        msg.chat.id,
        format!(
            "[Here's the data](t\\.me/iv?url\\=https://{}/sessions/{}/{}&rhash\\={})",
            *DOMAIN_NAME, rand_number, user, *VIEW_RHASH
        ),
    )
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
    let initial = n.to_string();
    let mut ris = String::new();
    let mut many = -1;
    initial.chars().for_each(|c| {
        if many == 0 {
            return;
        }
        many -= 1;
        if c == '.' {
            many = 3;
            ris.push('\\');
        }
        ris.push(c);
    });
    return ris;
}

async fn username_or_full(user: &User) -> String {
    if user.username.is_some() {
        format!("@{}", user.username.as_ref().unwrap())
    } else {
        format!("[{}](tg://user?id={})", user.full_name(), user.id.0,)
    }
}

async fn answer_average_with_window(
    conn: ToiletDB,
    bot: Bot,
    msg: Message,
    window: u64,
    label: &str,
) -> Result<()> {
    let current = UNIX_EPOCH.elapsed().unwrap().as_secs();
    let starting = current - window;
    match conn
        .query_shit_session_from(msg.from.as_ref().unwrap(), starting)
        .await
    {
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

async fn answer_shitting(conn: ToiletDB, bot: Bot, msg: Message) -> Result<()> {
    let (_, args) = parse_command(msg.text().unwrap(), &(*BOT_NAME)).unwrap();
    let user = msg.from.as_ref().unwrap();

    let new_record = match args.len() {
        0 => Some(
            conn.insert_shitting_session(user, false, false)
                .await
                .unwrap(),
        ),
        1 => {
            let duration = args.first().unwrap().parse();
            match duration {
                Err(_) => None,
                Ok(d) => Some(
                    conn.insert_shitting_session_with_duration(user, d, false, false)
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
                    conn.insert_shitting_session_with_location(
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
                    conn.insert_shitting_session_with_location(
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
                    conn.insert_shitting_session_with_location(user, d, args.get(1).unwrap(), h, c)
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
    let username = username_or_full(msg.from.as_ref().unwrap()).await;
    let current = DateTime::from_timestamp(new_record.timestamp as i64, 0)
        .unwrap()
        .with_timezone(&(*TIMEZONE));
    bot.send_message(
        msg.chat.id,
        format!(
            "💩💩💩\n{} added a new shitting session to the database with timestamp `{}`",
            username,
            current.format("%Y\\-%m\\-%d %H:%M")
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

pub async fn delete_shit_callback(conn: ToiletDB, bot: Bot, query: CallbackQuery) -> Result<()> {
    bot.answer_callback_query(&query.id).await?;
    let is_sender = !query
        .message
        .as_ref()
        .unwrap()
        .regular_message()
        .unwrap()
        .mentioned_users()
        .any(|user| user.id == query.from.id);

    if query.data.is_none() || is_sender {
        return Ok(());
    }

    let message = query.message.as_ref().unwrap();
    if conn
        .delete_shit_session(query.data.unwrap().parse().unwrap())
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
