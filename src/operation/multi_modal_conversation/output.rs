use std::pin::Pin;

use serde::{Deserialize, Serialize};
use tokio_stream::Stream;

use crate::{error::DashScopeError, operation::common::Usage};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    #[serde(rename = "text")]
    pub text: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    #[serde(rename = "content")]
    pub content: Vec<Content>,

    #[serde(rename = "role")]
    pub role: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Choices {
    #[serde(rename = "finish_reason")]
    pub finish_reason: Option<String>,

    #[serde(rename = "message")]
    pub message: Message,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Output {
    #[serde(rename = "choices")]
    pub choices: Vec<Choices>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MultiModalConversationOutput {
    /// 调用结果信息。
    #[serde(rename = "output")]
    pub output: Output,

    /// 本次调用的唯一标识符。
    #[serde(rename = "request_id")]
    pub request_id: String,

    /// 本次chat请求使用的token信息。
    #[serde(rename = "usage")]
    pub usage: Option<Usage>,
}

pub type MultiModalConversationOutputStream =
    Pin<Box<dyn Stream<Item = Result<MultiModalConversationOutput, DashScopeError>> + Send>>;
