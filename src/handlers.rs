use crate::db::*;
use teloxide::{
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
        Me,
    },
    utils::command::BotCommands,
};
use tokio::sync::Mutex;
use tokio::time;

use std::{collections::HashMap, sync::Arc};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Display this text.")]
    Help,
    #[command(
        description = "Start to log messages in this chat. Privilege is needed for this operation."
    )]
    Start,
    #[command(
        description = "Stop to log messages in this chat. Privilege is needed for this operation."
    )]
    Stop,
}

pub async fn command_handler(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    log::debug!("{}", serde_json::to_string_pretty(&msg).unwrap());
    match cmd {
        Command::Help => {
            log::debug!("got command help");
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Start => {
            log::debug!("got command start");
            if let Some(u) = msg.from() {
                if bot
                    .get_chat_member(msg.chat.id, u.id)
                    .await?
                    .is_privileged()
                {
                    log::debug!("{} is privileged", u.id);
                    Db::new().insert_chat_with_id(msg.chat.id).await;
                }
            }
        }
        Command::Stop => {
            log::debug!("got command stop");
            if let Some(u) = msg.from() {
                if bot
                    .get_chat_member(msg.chat.id, u.id)
                    .await?
                    .is_privileged()
                {
                    log::debug!("{} is privileged", u.id);
                    Db::new().delete_chat_with_id(msg.chat.id).await;
                }
            }
        }
    };

    Ok(())
}

pub async fn message_handler(msg: Message, me: Me) -> ResponseResult<()> {
    log::debug!("{}", serde_json::to_string_pretty(&msg).unwrap());
    if !msg.chat.is_supergroup() {
        return Ok(());
    }
    if let Some(b) = &msg.via_bot {
        if b.id == me.id {
            return Ok(());
        };
    }
    if let Some(u) = msg.from() {
        if u.is_bot {
            return Ok(());
        }
    }
    if msg.text().is_none() && msg.caption().is_none() {
        return Ok(());
    }

    match msg.text() {
        Some(t) => {
            if Command::parse(t, me.username()).is_err() {
                log::debug!("no command got. store it to db");
                normal_message_handler(msg).await?;
            }
        }
        None => {
            log::debug!("caption only. store it to db");
            normal_message_handler(msg).await?;
        }
    }

    Ok(())
}

pub async fn inline_handler(
    bot: Bot,
    q: InlineQuery,
    groups_cache: Arc<Mutex<HashMap<UserId, Vec<crate::types::Chat>>>>,
) -> ResponseResult<()> {
    log::debug!("{}", serde_json::to_string_pretty(&q).unwrap());

    {
        let mut groups_cache_pointer = groups_cache.lock().await;
        if !groups_cache_pointer.contains_key(&q.from.id) {
            log::debug!(
                "{} does not have a permissioned chat list. generate it.",
                q.from.id
            );
            let all_chats = Db::new().get_all_chats().await;
            for c in all_chats {
                match check_chat_member(bot.clone(), c, q.from.id).await {
                    true => {
                        log::debug!("{} have a member of {}", c, q.from.id);
                        groups_cache_pointer
                            .entry(q.from.id)
                            .or_insert(Vec::new())
                            .push(crate::types::Chat::from(c));
                    }
                    false => {
                        log::debug!("{} does not have a member of {}", c, q.from.id);
                        groups_cache_pointer.entry(q.from.id).or_insert(Vec::new());
                    }
                }
            }
            tokio::spawn({
                let expire_groups_cache = groups_cache.clone();
                async move {
                    log::debug!(
                        "permissioned chat list of {} is scheduled to expire in 120 seconds",
                        q.from.id
                    );
                    time::sleep(time::Duration::from_secs(120)).await;
                    expire_groups_cache.lock().await.remove(&q.from.id);
                    log::debug!("permissioned chat list of {} has expired", q.from.id)
                }
            });
        }
    }

    let search_results = Db::new()
        .search_message_with_filter_chats(
            &q.query,
            groups_cache
                .lock()
                .await
                .get(&q.from.id)
                .unwrap_or(&Vec::new()),
        )
        .await;
    bot.answer_inline_query(
        &q.id,
        search_results
            .hits
            .iter()
            .map(|m| (&m.result, m.formatted_result.as_ref().unwrap()))
            .map(|(m, f)| {
                InlineQueryResult::Article(
                    InlineQueryResultArticle::new(
                        &m.key,
                        f["text"].as_str().unwrap(),
                        InputMessageContent::Text(
                            InputMessageContentText::new(format!(
                                r#"「 {} 」 from <a href="{}">{}</a>"#,
                                html_escape::encode_text(&m.text),
                                m.link(),
                                html_escape::encode_text(&m.from),
                            ))
                            .parse_mode(teloxide::types::ParseMode::Html),
                        ),
                    )
                    .description(format!("{}@{}", &m.from, m.format_time())),
                )
            }),
    )
    .send()
    .await?;

    Ok(())
}

pub async fn normal_message_handler(msg: Message) -> ResponseResult<()> {
    if Db::new().filter_chat_with_id(msg.chat.id).await.is_none() {
        log::debug!("{} not a enabled chat", &msg.chat.id);
        return Ok(());
    }

    Db::new()
        .insert_messages(&vec![crate::types::Message::from(&msg)])
        .await;

    Ok(())
}

async fn check_chat_member(bot: Bot, chat_id: ChatId, user_id: UserId) -> bool {
    match bot.get_chat_member(chat_id, user_id).await {
        Ok(m) => m.is_present(),
        Err(_) => false,
    }
}
