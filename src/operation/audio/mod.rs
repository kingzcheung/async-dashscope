use crate::{Client, error::DashScopeError, operation::{audio::tts::output::TextToSpeechOutputStream, ws_client::WsClient}};
use crate::{error::Result, operation::audio::tts::output::TextToSpeechOutput};
pub use tts::param::{
    Input as TextToSpeechInput, InputBuilder as TextToSpeechInputBuilder, TextToSpeechParam,
    TextToSpeechParamBuilder,
};
pub mod tts;
pub mod ws;
// pub mod ws;

const AUDIO_PATH: &str = "/services/aigc/multimodal-generation/generation";

pub struct Audio<'a> {
    client: &'a Client,
}

impl<'a> Audio<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// 执行文本转语音(TTS)转换
    ///
    /// 此异步方法向指定端点发送 POST 请求，将文本转换为语音输出
    ///
    /// # 参数
    /// * `request` - TTS 转换参数配置，包含文本内容、语音模型等设置
    pub async fn tts(&self, request: TextToSpeechParam) -> Result<TextToSpeechOutput> {
        // 检查请求是否明确设置为非流式，如果是，则返回错误。
        if request.stream == Some(true) {
            return Err(DashScopeError::InvalidArgument(
                "When stream is true, use Audio::call_stream".into(),
            ));
        }
        self.client.post(AUDIO_PATH, request).await
    }

    pub async fn tts_stream(&self, request: TextToSpeechParam) -> Result<TextToSpeechOutputStream> {
        // 检查请求是否明确设置为非流式，如果是，则返回错误。
        if request.stream == Some(false) {
            return Err(DashScopeError::InvalidArgument(
                "When stream is false, use Audio::call".into(),
            ));
        }
        self.client.post_stream(AUDIO_PATH, request).await
    }

    pub async fn asr(&self) -> Result<ws::WebsocketInference> {
        let ws = WsClient::into_ws_client(self.client.clone()).await?;
        Ok(ws::WebsocketInference::new(ws))
    }
}