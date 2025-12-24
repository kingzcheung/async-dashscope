use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::operation::{common::Parameters, request::RequestTrait};

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct TextToSpeechParam {
    #[builder(setter(into))]
    pub model: String,
    pub input: Input,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub stream: Option<bool>,
    // 这个参数并不存在，只是为了兼容
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub parameters: Option<Parameters>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Input {
     #[builder(setter(into))]
    text: String,
    /// 音色,可选有 Chelsie,Cherry,Ethan,Serena,Dylan,Jada,Sunny
     #[builder(setter(into))]
    voice: String,

    /// 指定合成音频的语种，默认为 Auto。
    /// 
    /// - Auto：适用无法确定文本的语种或文本包含多种语言的场景，模型会自动为文本中的不同语言片段匹配各自的发音，但无法保证发音完全精准。
    /// - 指定语种：适用于文本为单一语种的场景，此时指定为具体语种，能显著提升合成质量，效果通常优于 Auto。可选值包括：
    ///     - Chinese
    ///     - English
    ///     - German
    ///     - Italian
    ///     - Portuguese
    ///     - Spanish
    ///     - Japanese
    ///     - Korean
    ///     - French
    ///     - Russian
    #[builder(setter(into))]
    language_type:Option<String>,
}

impl RequestTrait for TextToSpeechParam {
    fn model(&self) -> &str {
        &self.model
    }

    fn parameters(&self) -> Option<&Parameters> {
        None
    }
    
    type P = Parameters;
}
