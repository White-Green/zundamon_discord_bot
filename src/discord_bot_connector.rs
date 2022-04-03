use crate::core::{MessageInput, MessageResponder, MessageResponse};
use once_cell::sync::Lazy;
use regex::Regex;
use serenity::async_trait;
use unicode_normalization::UnicodeNormalization;

#[async_trait]
pub trait Connector {
    async fn message(&self, msg: &str) -> MessageResponse;
}

pub struct DiscordBotConnector<R>(pub R);

impl<R> DiscordBotConnector<R>
where
    R: MessageResponder,
{
    pub fn new(responder: R) -> DiscordBotConnector<R> {
        DiscordBotConnector(responder)
    }
}

macro_rules! static_regex {
    ($s:literal) => {{
        static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new($s).unwrap());
        &*REGEX
    }};
}

#[async_trait]
impl<R: Send + Sync + MessageResponder> Connector for DiscordBotConnector<R> {
    async fn message(&self, msg: &str) -> MessageResponse {
        let msg = msg.nfkc().collect::<String>();
        let msg = msg.to_lowercase();
        static PREFIX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^!\s*(?:ずんだ(?:もん)?|ズンダ(?:モン)?|zunda(?:monn?)?)\s+((?:.|\n)*)$").unwrap());
        let captures = if let Some(captures) = PREFIX_REGEX.captures(&msg) {
            captures
        } else {
            return MessageResponse::None;
        };
        assert_eq!(captures.len(), 2);
        let msg = captures.get(1).unwrap().as_str();
        let msg = match msg.trim() {
            text if static_regex!(r"^(?:help|\?|へるぷ|ヘルプ)$").is_match(text) => MessageInput::Help,
            text if static_regex!(r"^(?:おはよう(?:ございます)?!?|ぐ(?:っど?)?もーにんぐ?!?)$").is_match(text) => MessageInput::GoodMorning,
            text if static_regex!(r"^(?:こんにち[はわ]|はろー|ハロー|hello)$").is_match(text) => MessageInput::Hello,
            text if static_regex!(r"^(?:こんばん[はわ])$").is_match(text) => MessageInput::GoodEvening,
            text => {
                if let Some(captures) = static_regex!(r"^(?:say|せい|言って)\s+((?:.|\n)+)$").captures(dbg!(text)) {
                    assert_eq!(captures.len(), 2);
                    let text = captures.get(1).unwrap();
                    MessageInput::Say(text.as_str())
                } else {
                    return MessageResponse::None;
                }
            }
        };
        self.0.message(msg).await
    }
}
