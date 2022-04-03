use rand::prelude::SliceRandom;
use rand::thread_rng;
use serenity::async_trait;
use std::borrow::Cow;

pub enum MessageResponse {
    None,
    Message(Cow<'static, str>),
    MessageWithFile { message: Cow<'static, str>, file_name: Cow<'static, str>, file_content: Box<dyn Send + Sync + AsRef<[u8]>> },
}

#[derive(Debug, Clone, Copy)]
pub enum MessageInput<'a> {
    Help,
    GoodMorning,
    Hello,
    GoodEvening,
    Say(&'a str),
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Character {
    Zundamon,
}

#[async_trait]
pub trait TTSController {
    async fn text_to_speech(&self, character: Character, text: &str) -> anyhow::Result<Box<dyn Send + Sync + AsRef<[u8]>>>;
}

#[async_trait]
pub trait MessageResponder {
    async fn message(&self, msg: MessageInput<'_>) -> MessageResponse;
}

#[async_trait]
impl<'a, T: Send + Sync + MessageResponder> MessageResponder for &'a T {
    async fn message(&self, msg: MessageInput<'_>) -> MessageResponse {
        T::message(self, msg).await
    }
}

pub trait HelpMessage {
    fn message(&self) -> &'static str;
}

pub struct ZundaBotCore<H, T> {
    help: H,
    tts: T,
}

impl<H, T> ZundaBotCore<H, T>
where
    H: HelpMessage,
    T: TTSController,
{
    pub fn new(help_message: H, tts_controller: T) -> Self {
        ZundaBotCore { help: help_message, tts: tts_controller }
    }
}

#[async_trait]
impl<H, T> MessageResponder for ZundaBotCore<H, T>
where
    H: Send + Sync + HelpMessage,
    T: Send + Sync + TTSController,
{
    async fn message(&self, msg: MessageInput<'_>) -> MessageResponse {
        dbg!(&msg);
        match msg {
            MessageInput::Help => MessageResponse::Message(Cow::Borrowed(self.help.message())),
            MessageInput::GoodMorning => {
                const RESPONSES: &[&str] = &["おはようなのだ！", "おはようございますなのだ！", "ぐっどもーにんぐなのだ！", "朝なのだ！僕の朝食を作るのだ！", "今日の朝ごはんはずんだもちなのだ！"];
                MessageResponse::Message(Cow::Borrowed(RESPONSES.choose(&mut thread_rng()).unwrap()))
            }
            MessageInput::Hello => {
                const RESPONSES: &[&str] = &["こんにちはなのだ！", "はろーなのだ！", "今日の昼ごはんはずんだもちなのだ！"];
                MessageResponse::Message(Cow::Borrowed(RESPONSES.choose(&mut thread_rng()).unwrap()))
            }
            MessageInput::GoodEvening => {
                const RESPONSES: &[&str] = &["こんばんわなのだ！"];
                MessageResponse::Message(Cow::Borrowed(RESPONSES.choose(&mut thread_rng()).unwrap()))
            }
            MessageInput::Say(text) => {
                let text = format!("{text} なのだ");
                match self.tts.text_to_speech(Character::Zundamon, &text).await {
                    Ok(wav) => MessageResponse::MessageWithFile {
                        message: Cow::Owned(text),
                        file_name: Cow::Borrowed("zundamon.wav"),
                        file_content: wav,
                    },
                    Err(err) => {
                        println!("error by {}", err);
                        MessageResponse::None
                    }
                }
            }
        }
    }
}
