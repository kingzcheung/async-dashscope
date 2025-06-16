use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::operation::common::{Parameters, StreamOptions};
use crate::operation::request::RequestTrait;

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct GenerationParam {
    #[builder(setter(into, strip_option))]
    pub model: String,

    pub input: Input,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub parameters: Option<Parameters>,

    #[builder(setter(into, strip_option))]
    #[builder(default=Some(false))]
    pub stream: Option<bool>,

    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub stream_options: Option<StreamOptions>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Input {
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Message {
    #[builder(setter(into))]
    pub role: String,
    #[builder(setter(into))]
    pub content: String,
    #[builder(setter(into, strip_option))]
    #[builder(default=Some(false))]
    pub partial: Option<bool>,
}

impl Message {
    /// Creates a new `Message`.
    ///
    /// A convenience method for creating a message without the builder pattern.
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
            partial: Some(false),
        }
    }
}

impl RequestTrait for GenerationParam {
    fn model(&self) -> &str {
        &self.model
    }

    fn parameters(&self) -> Option<&Parameters> {
        self.parameters.as_ref()
    }
}
