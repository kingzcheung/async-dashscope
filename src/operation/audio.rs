use crate::client::Client;
use crate::error::DashScopeError;
use secrecy::ExposeSecret;

use super::AUDIO_PATH;

mod tts;
mod asr;

#[cfg(feature = "tts")]
use crate::operation::audio::tts::{TextToSpeechParam, TextToSpeechOutput, TextToSpeechOutputStream};

#[cfg(feature = "asr")]
use crate::operation::audio::asr::{
    param::AutomaticSpeechRecognitionParam,
    output::AutomaticSpeechRecognitionOutputStream,
    client::AsrClient,
};

/// 音频操作
pub struct Audio<'a> {
    client: &'a Client,
}

impl<'a> Audio<'a> {
    /// 创建音频操作实例
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// 文本转语音
    #[cfg(feature = "tts")]
    pub async fn tts(
        &self,
        param: TextToSpeechParam,
    ) -> Result<TextToSpeechOutput, DashScopeError> {
        let response = self
            .client
            .post(&format!("{}/text-to-speech", AUDIO_PATH))
            .json(&param)
            .send()
            .await?;

        let output = response.json().await?;
        Ok(output)
    }

    /// 文本转语音流式输出
    #[cfg(feature = "tts")]
    pub async fn tts_stream(
        &self,
        param: TextToSpeechParam,
    ) -> Result<TextToSpeechOutputStream, DashScopeError> {
        let response = self
            .client
            .post_stream(&format!("{}/text-to-speech", AUDIO_PATH))
            .json(&param)
            .send()
            .await?;

        Ok(response)
    }

    /// 创建 ASR 客户端
    #[cfg(feature = "asr")]
    pub fn asr(&self) -> AsrClient {
        AsrClient::new(self.client.http_client(), self.client.config().api_key().expose_secret().to_string())
    }
}