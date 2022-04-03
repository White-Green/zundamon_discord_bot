use crate::core::{Character, TTSController};
use reqwest::{Client, IntoUrl, Url};
use serde::Deserialize;
use serenity::async_trait;
use std::collections::HashMap;

pub struct VoicevoxTTS {
    client: Client,
    endpoint: Url,
    speakers: HashMap<Character, i32>,
}

#[derive(Deserialize)]
struct SpeakerStyle {
    name: String,
    id: i32,
}

#[derive(Deserialize)]
struct Speaker {
    name: String,
    // speaker_uuid: String,
    styles: Vec<SpeakerStyle>,
    // version: String,
}

fn find_speaker_id(speakers: &[Speaker], speaker_name: &str, style_name: &str) -> Option<i32> {
    Some(speakers.iter().find(|speaker| speaker.name == speaker_name)?.styles.iter().find(|style| style.name == style_name)?.id)
}

impl VoicevoxTTS {
    pub async fn new(endpoint: impl IntoUrl) -> anyhow::Result<VoicevoxTTS> {
        let client = Client::new();
        let endpoint = endpoint.into_url()?;
        let speakers: Vec<Speaker> = client.get(endpoint.join("speakers")?).header("Accept", "application/json").send().await?.json().await?;
        let speakers = HashMap::from([(Character::Zundamon, find_speaker_id(&speakers, "ずんだもん", "あまあま").ok_or_else(|| anyhow::Error::msg("ずんだもん not found"))?)]);
        Ok(VoicevoxTTS { client, endpoint, speakers })
    }
}

#[async_trait]
impl TTSController for VoicevoxTTS {
    async fn text_to_speech(&self, character: Character, text: &str) -> anyhow::Result<Box<dyn Send + Sync + AsRef<[u8]>>> {
        let speaker_id = *self.speakers.get(&character).unwrap();
        let mut url = self.endpoint.join("audio_query").unwrap();
        url.query_pairs_mut().append_pair("text", text).append_pair("speaker", &speaker_id.to_string());
        let query = self.client.post(url).send().await?.text().await?;
        let mut url = self.endpoint.join("synthesis").unwrap();
        url.query_pairs_mut().append_pair("speaker", &speaker_id.to_string());
        let wav = self.client.post(url).body(query).send().await?.bytes().await?;
        Ok(Box::new(wav))
    }
}
