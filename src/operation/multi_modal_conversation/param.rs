use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::operation::common::Parameters;
use crate::operation::request::RequestTrait;

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct MultiModalConversationParam {
    #[builder(setter(into))]
    pub model: String,
    pub input: Input,

    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub parameters: Option<Parameters>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Input {
    messages: Vec<Message>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Message {
    #[builder(setter(into))]
    role: String,
    #[serde(rename = "content")]
    contents: Vec<Element>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Element {
    Image(String),
    Video(String),
    Audio(String),
    Text(String),
}

impl TryFrom<Value> for Element {
    type Error = crate::error::DashScopeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        // image: {"image": "https://example.com/image.png"}
        // video: {"video": "https://example.com/video.mp4"}
        // audio: {"audio": "https://example.com/audio.mp3"}
        // text: {"text": "Hello, world!"}
        if let Some(image) = value.get("image") {
            if let Some(s) = image.as_str() {
                return Ok(Element::Image(s.to_string()));
            }
        }
        if let Some(video) = value.get("video") {
            if let Some(s) = video.as_str() {
                return Ok(Element::Video(s.to_string()));
            }
        }
        if let Some(audio) = value.get("audio") {
            if let Some(s) = audio.as_str() {
                return Ok(Element::Audio(s.to_string()));
            }
        }
        if let Some(text) = value.get("text") {
            if let Some(s) = text.as_str() {
                // 处理文本字段
                return Ok(Element::Text(s.to_string()));
            }
        }
        Err(crate::error::DashScopeError::ElementError(
            "Invalid element type.".into(),
        ))
    }
}

impl Message {
    /// Creates a new `Message` with a single content item.
    ///
    /// A convenience method for creating a message without the builder pattern.
    pub fn new(role: impl Into<String>, contents: Vec<Element>) -> Self {
        Self {
            role: role.into(),
            contents,
        }
    }

    pub fn push_content(&mut self, content: Element) {
        self.contents.push(content);
    }
}

impl RequestTrait for MultiModalConversationParam {
    fn model(&self) -> &str {
        &self.model
    }

    fn parameters(&self) -> Option<&Parameters> {
        self.parameters.as_ref()
    }
}
