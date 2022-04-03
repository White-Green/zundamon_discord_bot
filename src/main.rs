use std::borrow::Cow;
use std::env;

use crate::core::{HelpMessage, MessageResponse, ZundaBotCore};
use crate::discord_bot_connector::{Connector, DiscordBotConnector};
use crate::voicevox_tts::VoicevoxTTS;
use serenity::http::AttachmentType;
use serenity::{async_trait, model::{channel::Message, gateway::Ready}, prelude::*};

mod core;
mod discord_bot_connector;
mod voicevox_tts;

struct Handler<R>(DiscordBotConnector<R>);

impl<R> Handler<R>
    where
        DiscordBotConnector<R>: Connector,
{
    pub fn new(connector: DiscordBotConnector<R>) -> Handler<R> {
        Handler(connector)
    }
}

#[async_trait]
impl<R: Send + Sync> EventHandler for Handler<R>
    where
        DiscordBotConnector<R>: Connector,
{
    async fn message(&self, ctx: Context, msg: Message) {
        match self.0.message(&msg.content).await {
            MessageResponse::None => {}
            MessageResponse::Message(message) => {
                if let Err(err) = msg.channel_id.say(&ctx.http, message).await {
                    println!("error:{err}");
                }
            }
            MessageResponse::MessageWithFile { message, file_name, file_content } => {
                if let Err(err) = msg.channel_id.send_files(&ctx.http, [AttachmentType::Bytes { data: Cow::Borrowed((*file_content).as_ref()), filename: file_name.to_string() }], |m| m.content(message)).await {
                    println!("error:{err}");
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

struct DiscordHelp;

impl HelpMessage for DiscordHelp {
    fn message(&self) -> &'static str {
        r#"> 僕はずんだもんbotなのだ
> 僕に話しかけるには最初に `!ずんだもん` をつけて話しかけてほしいのだ"#
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let handler = Handler::new(DiscordBotConnector::new(ZundaBotCore::new(DiscordHelp, VoicevoxTTS::new("http://localhost:50021").await.unwrap())));
    let mut client = Client::builder(&token).event_handler(handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
