use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::operation::common::Parameters;


#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct MultiModalConversationParam {
    #[builder(setter(into))]
    pub model: String,
    pub input: Input,

    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub stream:Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub parameters: Option<Parameters>
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct Input {
    messages: Vec<Message>,
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct Message {
    #[builder(setter(into))]
    role: String,
    #[serde(rename = "content")]
    contents: Vec<Value>
}

