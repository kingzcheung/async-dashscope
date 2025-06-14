use std::pin::Pin;

use serde::{Deserialize, Serialize};
use tokio_stream::Stream;

use crate::error::DashScopeError;

use super::Usage;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    /// 输出消息的内容。当使用qwen-vl或qwen-audio系列模型时为array，其余情况为string。
    #[serde(rename = "content")]
    pub content: String,

    /// 输出消息的角色，固定为assistant。
    #[serde(rename = "role")]
    pub role: String,

    // 思考内容
    #[serde(rename = "reasoning_content")]
    pub reasoning_content:Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Choices {
    /// 有四种情况：
    /// - 正在生成时为null；
    /// - 因模型输出自然结束，或触发输入参数中的stop条件而结束时为stop；
    /// - 因生成长度过长而结束为length；
    /// - 因发生工具调用为tool_calls。
    #[serde(rename = "finish_reason")]
    pub finish_reason: Option<String>,

    /// 模型输出的消息对象。
    #[serde(rename = "message")]
    pub message: Message,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Output {
    /// 模型的输出信息。当result_format为message时返回choices参数。
    #[serde(rename = "choices")]
    pub choices: Vec<Choices>,

    /// 模型生成的回复。当设置输入参数result_format为text时将回复内容返回到该字段。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// 当设置输入参数result_format为text时该参数不为空。
    /// 有四种情况：
    /// - 正在生成时为null；
    /// - 因模型输出自然结束，或触发输入参数中的stop条件而结束时为stop；
    /// - 因生成长度过长而结束为length；
    /// - 因发生工具调用为tool_calls。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,

    /// 联网搜索到的信息，在设置search_options参数后会返回该参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_info: Option<SearchInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchInfo {
    #[serde(rename = "search_results")]
    pub search_results: Vec<SearchResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    /// 搜索结果来源的网站名称。
    pub site_name: String,
    /// 来源网站的图标URL，如果没有图标则为空字符串。
    pub icon: Option<String>,

    /// 搜索结果的序号，表示该搜索结果在search_results中的索引。
    pub index: i32,

    /// 搜索结果的标题。
    pub title: Option<String>,

    /// 搜索结果的链接地址。
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenerationOutput {
    /// 调用结果信息。
    #[serde(rename = "output")]
    pub output: Output,

    /// 本次调用的唯一标识符。
    #[serde(rename = "request_id")]
    pub request_id: Option<String>,

    /// 本次chat请求使用的token信息。
    #[serde(rename = "usage")]
    pub usage: Option<Usage>,
}

pub type GenerationOutputStream =
    Pin<Box<dyn Stream<Item = Result<GenerationOutput, DashScopeError>> + Send>>;
