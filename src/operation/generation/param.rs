use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::operation::common::{Parameters, StreamOptions};

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct GenerationParam {
    
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

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct Input {
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct Message {
    #[builder(setter(into))]
    pub role: String,
    #[builder(setter(into))]
    pub content: String,
    #[builder(setter(into, strip_option))]
    #[builder(default=Some(false))]
    pub partial: Option<bool>,
}
