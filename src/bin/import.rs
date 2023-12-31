use telegram_cjk_search_bot::*;

use clap::Parser;
use db::Db;
use serde::Deserialize;
use serde_json::from_str;
use std::fs;
use std::path::PathBuf;
use teloxide::prelude::*;
use teloxide::types::ChatId;

#[derive(Deserialize)]
struct Content {
    name: String,
    r#type: String,
    id: ChatId,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct Entitiy {
    text: String,
}

#[derive(Deserialize)]
struct Message {
    id: i32,
    r#type: String,
    date_unixtime: String,
    from: Option<String>,
    from_id: Option<String>,
    via_bot: Option<String>,
    text_entities: Vec<Entitiy>,
}

const INSERT_BATCH_LIMIT: usize = 2000;

#[derive(Parser)]
#[command(author, version, long_about = None)]
#[command(about = "Import chat history from a json file to meilisearch db.")]
struct Cli {
    #[arg(default_value = "./history/result.json")]
    file: PathBuf,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let cli = Cli::parse();
    let bot_username = format!("@{}", Bot::from_env().get_me().await.unwrap().username());
    let mut msgs = vec![];
    let mut handles = vec![];

    let content = from_str::<Content>(&fs::read_to_string(cli.file).unwrap()).unwrap();
    assert!(content.r#type.contains("supergroup"));
    log::info!("parse succeed.");

    let mut successful_count: usize = 0;
    for m in content.messages {
        if m.r#type != "message" {
            continue;
        }
        if let Some(u) = m.via_bot {
            if u == bot_username {
                continue;
            }
        }
        if let Some(from_id) = &m.from_id {
            let mut text = String::new();
            m.text_entities.iter().for_each(|ele| {
                text.push_str(&ele.text);
            });
            let text = text;
            if text.is_empty() {
                continue;
            }
            msgs.push(types::Message {
                key: format!("-100{}_{}", &content.id, m.id),
                text,
                from: format!(
                    "{}@{}",
                    match m.from {
                        Some(f) => f,
                        None => format!("已销号{}", from_id),
                    },
                    &content.name
                ),
                id: m.id,
                chat_id: teloxide::types::ChatId(
                    format!("-100{}", content.id).parse::<i64>().unwrap(),
                ),
                date: chrono::DateTime::from_utc(
                    chrono::NaiveDateTime::from_timestamp_opt(
                        m.date_unixtime.parse::<i64>().unwrap(),
                        0,
                    )
                    .unwrap(),
                    chrono::Utc,
                ),
            });
            successful_count += 1;
            if msgs.len() >= INSERT_BATCH_LIMIT {
                handles.push(tokio::spawn({
                    let imsgs = msgs;
                    async move {
                        Db::new().insert_messages(&imsgs).await;
                    }
                }));
                msgs = vec![];
            }
        }
    }
    if !msgs.is_empty() {
        handles.push(tokio::spawn({
            let imsgs = msgs;
            async move {
                Db::new().insert_messages(&imsgs).await;
            }
        }));
    }

    log::info!(
        "insert {} messages. waiting for complete.",
        successful_count
    );
    futures::future::join_all(handles).await;
    log::info!("done.");
}
