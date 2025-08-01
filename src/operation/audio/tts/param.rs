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
    // 音色,可选有 Chelsie,Cherry,Ethan,Serena,Dylan,Jada,Sunny
     #[builder(setter(into))]
    voice: String,
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
