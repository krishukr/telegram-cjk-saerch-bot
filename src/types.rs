use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use teloxide::types::ChatId;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Chat {
    pub id: ChatId,
}

impl std::fmt::Debug for Chat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl From<teloxide::types::ChatId> for Chat {
    fn from(id: teloxide::types::ChatId) -> Self {
        Self { id }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub key: String,
    pub text: String,
    pub from: String,
    pub id: i32,
    pub chat_id: ChatId,
    pub date: DateTime<Utc>,
}

impl From<&teloxide::types::Message> for Message {
    fn from(msg: &teloxide::types::Message) -> Self {
        Self {
            key: format!("{}_{}", msg.chat.id, msg.id),
            text: match msg.text() {
                Some(text) => text.to_string(),
                None => msg.caption().unwrap().to_string(),
            },
            from: match msg.sender_chat() {
                Some(c) => c.title().unwrap().to_string(),
                None => format!(
                    "{}@{}",
                    msg.from().unwrap().full_name(),
                    msg.chat.title().unwrap()
                ),
            },
            id: msg.id.0,
            chat_id: msg.chat.id,
            date: msg.date,
        }
    }
}

impl Message {
    pub fn format_time(&self) -> String {
        self.date
            .with_timezone(chrono::Local::now().offset())
            .format("%Y-%m-%d")
            .to_string()
    }

    pub fn link(&self) -> String {
        let chat_id_string = self.chat_id.to_string();
        format!(
            "https://t.me/c/{}/{}",
            &chat_id_string[4..chat_id_string.len()],
            self.id
        )
    }
}

#[cfg(test)]
mod types_tests {
    use crate::types::*;

    #[test]
    fn message_date_format_test() {
        std::env::set_var("TZ", "Asia/Shanghai");
        let msg = Message::from(
            &serde_json::from_str::<teloxide::types::Message>(
                r#"{
            "message_id": 2,
            "message_thread_id": null,
            "date": 1689699600,
            "chat": {
                "id": -1001,
                "title": "test",
                "type": "supergroup",
                "is_forum": false
            },
            "via_bot": null,
            "from": {
                "id": 1,
                "is_bot": false,
                "first_name": "Fop",
                "last_name": "Bar",
                "username": "Foo_Bar",
                "language_code": "zh-hans"
            },
            "text": "2",
            "entities": [],
            "is_topic_message": false,
            "is_automatic_forward": false,
            "has_protected_content": false
        }"#,
            )
            .unwrap(),
        );
        assert_eq!(msg.format_time(), "2023-07-19");
    }

    #[test]
    fn message_channel_anonymous_test() {
        let msg = Message::from(
            &serde_json::from_str::<teloxide::types::Message>(
                r#"{
            "message_id": 139417,
            "message_thread_id": null,
            "date": 1689781753,
            "chat": {
                "id": -1002,
                "title": "test2",
                "type": "supergroup",
                "is_forum": false
            },
            "via_bot": null,
            "from": {
                "id": 1,
                "is_bot": true,
                "first_name": "Group",
                "username": "GroupAnonymousBot"
            },
            "sender_chat": {
                "id": -1002,
                "title": "test2",
                "type": "supergroup",
                "is_forum": false
            },
            "text": "test",
            "entities": [],
            "is_topic_message": false,
            "is_automatic_forward": false,
            "has_protected_content": false
        }"#,
            )
            .unwrap(),
        );
        assert_eq!(msg.from, "test2");
    }

    #[test]
    fn message_user_test() {
        let msg = Message::from(
            &serde_json::from_str::<teloxide::types::Message>(
                r#"{
            "message_id": 2,
            "message_thread_id": null,
            "date": 1689731458,
            "chat": {
                "id": -1001,
                "title": "test",
                "type": "supergroup",
                "is_forum": false
            },
            "via_bot": null,
            "from": {
                "id": 1,
                "is_bot": false,
                "first_name": "Foo",
                "last_name": "Bar",
                "username": "Foo_Bar",
                "language_code": "zh-hans"
            },
            "text": "2",
            "entities": [],
            "is_topic_message": false,
            "is_automatic_forward": false,
            "has_protected_content": false
            }"#,
            )
            .unwrap(),
        );
        assert_eq!(msg.from, "Foo Bar@test");
    }

    #[test]
    fn message_text_form_text_test() {
        let msg = Message::from(
            &serde_json::from_str::<teloxide::types::Message>(
                r#"{
            "message_id": 2,
            "message_thread_id": null,
            "date": 1689731458,
            "chat": {
                "id": -1001,
                "title": "test",
                "type": "supergroup",
                "is_forum": false
            },
            "via_bot": null,
            "from": {
                "id": 1,
                "is_bot": false,
                "first_name": "Foo",
                "last_name": "Bar",
                "username": "Foo_Bar",
                "language_code": "zh-hans"
            },
            "text": "2",
            "entities": [],
            "is_topic_message": false,
            "is_automatic_forward": false,
            "has_protected_content": false
        }"#,
            )
            .unwrap(),
        );
        assert_eq!(msg.text, "2");
    }

    #[test]
    fn message_text_from_caption_test() {
        let msg = Message::from(
            &serde_json::from_str::<teloxide::types::Message>(
                r#"{
            "message_id": 3,
            "message_thread_id": null,
            "date": 1689731481,
            "chat": {
                "id": -1001,
                "title": "test",
                "type": "supergroup",
                "is_forum": false
            },
            "via_bot": null,
            "from": {
                "id": 1,
                "is_bot": false,
                "first_name": "Foo",
                "last_name": "Bar",
                "username": "Foo_Bar",
                "language_code": "zh-hans"
            },
            "photo": [
                {
                    "file_id": "0-1",
                    "file_unique_id": "2",
                    "file_size": 1224,
                    "width": 90,
                    "height": 81
                },
                {
                    "file_id": "0-1",
                    "file_unique_id": "2",
                    "file_size": 3493,
                    "width": 156,
                    "height": 141
                }
            ],
            "caption": "112",
            "caption_entities": [],
            "is_topic_message": false,
            "is_automatic_forward": false,
            "has_protected_content": false
        }"#,
            )
            .unwrap(),
        );
        assert_eq!(msg.text, "112")
    }

    #[test]
    fn message_link_test() {
        let msg = Message::from(
            &serde_json::from_str::<teloxide::types::Message>(
                r#"{
            "message_id": 3,
            "message_thread_id": null,
            "date": 1689731481,
            "chat": {
                "id": -1001,
                "title": "test",
                "type": "supergroup",
                "is_forum": false
            },
            "via_bot": null,
            "from": {
                "id": 1,
                "is_bot": false,
                "first_name": "Foo",
                "last_name": "Bar",
                "username": "Foo_Bar",
                "language_code": "zh-hans"
            },
            "photo": [
                {
                    "file_id": "0-1",
                    "file_unique_id": "2",
                    "file_size": 1224,
                    "width": 90,
                    "height": 81
                },
                {
                    "file_id": "0-1",
                    "file_unique_id": "2",
                    "file_size": 3493,
                    "width": 156,
                    "height": 141
                }
            ],
            "caption": "112",
            "caption_entities": [],
            "is_topic_message": false,
            "is_automatic_forward": false,
            "has_protected_content": false
        }"#,
            )
            .unwrap(),
        );
        assert_eq!(msg.link(), "https://t.me/c/1/3")
    }
}
